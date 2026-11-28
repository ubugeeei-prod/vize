import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync, type SpawnSyncOptionsWithStringEncoding } from "node:child_process";
import type {
  ProfileName,
  CommandOptions,
  CargoLockPackage,
  CargoLockField,
  DependencyIdentity,
} from "./types.ts";

function isCargoLockField(value: string): value is CargoLockField {
  return value === "name" || value === "version" || value === "source" || value === "checksum";
}

export function command(
  executable: string,
  args: readonly string[],
  options: CommandOptions = {},
): string {
  const launchOptions: SpawnSyncOptionsWithStringEncoding = {
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
    ...options,
  };
  const result = spawnSync(executable, args, launchOptions);
  assert.equal(
    result.status,
    0,
    `${executable} ${args.join(" ")} failed: ${result.error ?? result.stderr}`,
  );
  return result.stdout;
}
export function sha(file: string): string {
  return createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}
export function json(file: string, data: unknown): void {
  fs.writeFileSync(file, `${JSON.stringify(data, null, 2)}\n`);
}
export function profile(repo: string, name: ProfileName): string {
  const text = fs.readFileSync(path.join(repo, "Cargo.toml"), "utf8");
  const found = text.match(
    new RegExp(`^\\[profile\\.${name}\\]\\n[\\s\\S]*?(?=^\\[|$(?![\\s\\S]))`, "m"),
  );
  assert.ok(found, `missing ${name} profile`);
  return found[0].trimEnd();
}
function lockPackages(file: string): CargoLockPackage[] {
  const text = fs.readFileSync(file, "utf8");
  return text
    .split(/^\[\[package\]\]\s*$/m)
    .slice(1)
    .map((block) => {
      const row: Partial<CargoLockPackage> = {};
      for (const match of block.matchAll(/^(name|version|source|checksum) = ("[^\n]*")$/gm)) {
        const field = match[1];
        const encoded = match[2];
        assert.ok(field && isCargoLockField(field), "invalid Cargo lock identity field");
        assert.ok(encoded, "missing Cargo lock identity value");
        assert.ok(!Object.hasOwn(row, field), "duplicate Cargo lock identity field");
        const value: unknown = JSON.parse(encoded);
        assert.ok(typeof value === "string", "Cargo lock identity must decode to a string");
        row[field] = value;
      }
      const { name, version, source, checksum } = row;
      assert.ok(name && version, "missing Cargo lock package identity");
      return {
        name,
        version,
        ...(source === undefined ? {} : { source }),
        ...(checksum === undefined ? {} : { checksum }),
      };
    });
}
export function dependencyIdentity(originalFile: string, resolvedFile: string): DependencyIdentity {
  const key = (row: CargoLockPackage): string =>
    JSON.stringify([row.name, row.version, row.source ?? ""]);
  const allowed = new Map<string, CargoLockPackage>(
    lockPackages(originalFile).map((row): [string, CargoLockPackage] => [key(row), row]),
  );
  const resolved = lockPackages(resolvedFile);
  for (const row of resolved) {
    if (row.name === "text_rebuild_probe") continue;
    const previous = allowed.get(key(row));
    assert.ok(previous, `dependency identity changed: ${key(row)}`);
    assert.equal(row.checksum, previous.checksum, `dependency checksum changed: ${key(row)}`);
  }
  return { packages: resolved.length, identities: "all-existing-packages-match-source-lock-v1" };
}
