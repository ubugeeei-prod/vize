import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse as parseToml } from "@iarna/toml";

type Target = { name: string; kind: string[]; src_path: string; "required-features"?: string[] };
type Package = { id: string; name: string; manifest_path: string; targets: Target[] };
type Metadata = { workspace_members: string[]; packages: Package[] };
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readMetadata(cwd: string): Metadata {
  const result = spawnSync(
    "cargo",
    ["metadata", "--format-version", "1", "--no-deps", "--locked", "--offline"],
    { cwd, encoding: "utf8", maxBuffer: 8 * 1024 * 1024 },
  );
  assert.equal(result.status, 0, result.stderr || result.error?.message);
  return JSON.parse(result.stdout) as Metadata;
}

function invalidTargets(metadata: Metadata): string[] {
  const members = new Set(metadata.workspace_members);
  const invalid: string[] = [];
  for (const pkg of metadata.packages) {
    if (!members.has(pkg.id)) continue;
    for (const target of pkg.targets) {
      if (!fs.existsSync(target.src_path) || !fs.statSync(target.src_path).isFile()) {
        invalid.push(`${pkg.name}:${target.name}: missing source ${target.src_path}`);
      }
    }
    const manifest = parseToml(fs.readFileSync(pkg.manifest_path, "utf8"));
    for (const kind of ["bin", "test", "bench", "example"]) {
      const declarations = manifest[kind] as { name: string }[] | undefined;
      for (const declaration of declarations ?? []) {
        if (
          !pkg.targets.some(
            (target) => target.name === declaration.name && target.kind.includes(kind),
          )
        ) {
          invalid.push(`${pkg.name}:${declaration.name}: unregistered explicit ${kind} target`);
        }
      }
    }
  }
  return invalid;
}

const metadata = readMetadata(repoRoot);

test("every workspace Cargo target has a source and explicit targets are registered", () => {
  assert.deepEqual(invalidTargets(metadata), []);
});

test("the renamed DOM filter target retains its legacy oracle feature gate", () => {
  const pkg = metadata.packages.find((pkg) => pkg.name === "vize_atelier_dom");
  assert.ok(pkg);
  const target = pkg.targets.find((target) => target.name === "l2_filters");
  assert.ok(target);
  assert.equal(path.basename(target.src_path), "l2_filters.rs");
  assert.deepEqual(target["required-features"], ["legacy"]);
  assert.ok(!pkg.targets.some((target) => target.name === "davinci_l2_filters"));
});

test("real Cargo metadata cannot conceal a dangling explicit target from the guard", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-target-registration-"));
  try {
    fs.mkdirSync(path.join(cwd, "src"));
    fs.writeFileSync(path.join(cwd, "src/lib.rs"), "pub fn witness() {}\n");
    fs.writeFileSync(
      path.join(cwd, "Cargo.toml"),
      '[package]\nname="target_witness"\nversion="0.1.0"\nedition="2024"\n' +
        '[[test]]\nname="missing_witness"\n',
    );
    fs.writeFileSync(
      path.join(cwd, "Cargo.lock"),
      'version = 4\n[[package]]\nname = "target_witness"\nversion = "0.1.0"\n',
    );
    const invalid = invalidTargets(readMetadata(cwd));
    assert.ok(
      invalid.some((message) => message.includes("missing_witness")),
      invalid.join("\n"),
    );
  } finally {
    fs.rmSync(cwd, { recursive: true, force: true });
  }
});
