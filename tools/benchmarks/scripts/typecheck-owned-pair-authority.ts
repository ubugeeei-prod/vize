/** Source and loaded-image custody, outside timed samples. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync, realpathSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { sha256, type NativeAuthority } from "./typecheck-owned-pair-evidence.ts";

export function sourceRevision(directory: string) {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: directory, encoding: "utf8" });
  assert.equal(result.status, 0);
  assert.match(result.stdout.trim(), /^[0-9a-f]{40}$/u);
  return result.stdout.trim();
}
export function ownedNativeAuthority(directory: string, source: string): NativeAuthority {
  assert.equal(sourceRevision(directory), source);
  const clean = spawnSync("git", ["diff", "HEAD", "--exit-code"], { cwd: directory });
  assert.equal(clean.status, 0, "source checkout changed");
  const probe = spawnSync(
    process.execPath,
    [
      "-e",
      `
    const fs=require('node:fs'), path=require('node:path');
    const binding=require(process.argv[1]);
    const images=Object.entries(require.cache).filter(([file,module])=>file.endsWith('.node')&&module.exports===binding);
    if(images.length!==1) throw Error('exact native image unavailable');
    console.log(JSON.stringify({image:fs.realpathSync(images[0][0]),token:binding.getNativeBuildToken()}));
  `,
      join(directory, "npm/native"),
    ],
    { encoding: "utf8" },
  );
  assert.equal(probe.status, 0);
  assert.equal(probe.stderr, "");
  const observed = JSON.parse(probe.stdout);
  const image = realpathSync(observed.image);
  assert.ok(
    image.startsWith(`${realpathSync(join(directory, "npm/native"))}/`),
    "native image resolved outside this source role",
  );
  const built = join(directory, "npm/native/.artifacts/native", image.split("/").at(-1)!);
  assert.ok(
    readFileSync(built).equals(readFileSync(image)),
    "loader image differs from this invocation's N-API build output",
  );
  assert.match(observed.token, /^[0-9a-f]{32}$/u);
  assert.ok(readFileSync(image).includes(Buffer.from(observed.token)));
  return { source, image, sha256: sha256(readFileSync(image)), token: observed.token };
}

/** Hash the complete transitive file closure, following symlinks once per real directory. */
export function providerClosure(roots: string[]) {
  const visited = new Set<string>(),
    rows: Array<{ path: string; bytes: number; sha256: string }> = [];
  const walk = (input: string) => {
    const absolute = realpathSync(input);
    if (visited.has(absolute)) return;
    visited.add(absolute);
    const entry = statSync(absolute);
    if (entry.isDirectory())
      for (const name of readdirSync(absolute).sort()) walk(join(absolute, name));
    else if (entry.isFile())
      rows.push({ path: absolute, bytes: entry.size, sha256: sha256(readFileSync(absolute)) });
    else throw new Error(`unexpected provider entry: ${absolute}`);
  };
  for (const input of roots) walk(input);
  return rows.sort((a, b) => a.path.localeCompare(b.path));
}
