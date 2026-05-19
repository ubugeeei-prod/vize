import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

test("fuzz workspace declares libfuzzer-sys and an isolated [workspace]", () => {
  const manifest = readRepoFile("fuzz/Cargo.toml");

  // The fuzz crate must be its own workspace so the root workspace stable
  // toolchain is not pinned to libfuzzer-sys's nightly requirement.
  assert.match(manifest, /^\[workspace\]\s*$/m);

  assert.match(manifest, /libfuzzer-sys\s*=\s*"0\.4"/);
  assert.match(manifest, /cargo-fuzz\s*=\s*true/);

  // Every declared bin target must have a corresponding harness file so
  // `cargo fuzz run <target>` resolves cleanly on CI.
  const binMatches = [...manifest.matchAll(/\[\[bin\]\]\s+name = "([^"]+)"\s+path = "([^"]+)"/g)];
  assert.ok(binMatches.length > 0, "fuzz/Cargo.toml must declare at least one [[bin]] target");
  for (const [, , relativePath] of binMatches) {
    const fullPath = path.join(repoRoot, "fuzz", relativePath);
    assert.ok(
      fs.existsSync(fullPath),
      `fuzz target file ${relativePath} declared in Cargo.toml is missing`,
    );
  }
});

test("fuzz CI workflow gates short PR fuzz and schedules long nightly fuzz", () => {
  const workflow = readRepoFile(".github/workflows/fuzz.yml");

  assert.match(workflow, /name:\s*Fuzz/);
  assert.match(workflow, /schedule:[\s\S]*?-\s*cron:/);
  assert.match(workflow, /pull_request:[\s\S]*paths:/);

  // The matrix must drive each fuzz_target declared in fuzz/Cargo.toml.
  const manifest = readRepoFile("fuzz/Cargo.toml");
  const targets = [...manifest.matchAll(/\[\[bin\]\]\s+name = "([^"]+)"/g)].map(([, name]) => name);
  for (const target of targets) {
    assert.match(
      workflow,
      new RegExp(`target:\\s*\\[[^\\]]*${target}[^\\]]*\\]`),
      `fuzz workflow matrix missing ${target}`,
    );
  }

  assert.match(workflow, /cargo \+nightly fuzz run/);
  assert.match(workflow, /-max_total_time=/);

  // Reproducers on failure must be uploaded so triage does not have to
  // re-run the fuzzer to recover the failing input.
  assert.match(workflow, /upload-artifact[\s\S]*fuzz\/artifacts\//);
});

test("seed_corpus.mjs writes seeds for every declared fuzz target", () => {
  const script = readRepoFile("tools/fuzz/seed_corpus.mjs");
  const manifest = readRepoFile("fuzz/Cargo.toml");
  const targets = [...manifest.matchAll(/\[\[bin\]\]\s+name = "([^"]+)"/g)].map(([, name]) => name);

  for (const target of targets) {
    assert.match(
      script,
      new RegExp(`resetCorpus\\("${target}"\\)`),
      `seed_corpus.mjs must seed corpus/${target}/`,
    );
  }
});
