import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const command = path.join(repoRoot, "tools/commands/davinci/consumer-migration-surfaces.rs");
const legacyFiles = [
  "tools/support/compat/davinci/consumer-migration-surfaces.mjs",
  "tools/support/compat/davinci/lib/artifact-set.mjs",
  "tools/support/compat/davinci/lib/consumer-migration-render.mjs",
  "tools/support/compat/davinci/lib/consumer-migration-scan.mjs",
  "tools/support/compat/davinci/lib/consumer-migration-summary.mjs",
  "tools/support/compat/davinci/lib/markdown.mjs",
  "tools/support/compat/davinci/lib/ordering.mjs",
  "tools/support/compat/davinci/lib/paths.mjs",
  "tools/support/compat/davinci/lib/rust-source.mjs",
];
const scannedCrates = [
  "vize_atelier_core",
  "vize_atelier_dom",
  "vize_atelier_sfc",
  "vize_atelier_ssr",
  "vize_atelier_vapor",
  "vize_atelier_jsx",
  "vize_patina",
  "vize_canon",
  "vize_glyph",
  "vize_maestro",
];
const shardDir = "docs/davinci/plan/consumer-migration-surfaces";
const coreShard = `${shardDir}/compiler/vize_atelier_core.tsv`;

function writeFile(root, relPath, content) {
  const target = path.join(root, relPath);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, content);
}

function copyFile(root, relPath) {
  const target = path.join(root, relPath);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.copyFileSync(path.join(repoRoot, relPath), target);
}

function createScratchRepo() {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-consumer-surfaces-"));
  writeFile(scratch, "Cargo.toml", "[workspace]\n");
  writeFile(scratch, "pnpm-workspace.yaml", "packages: []\n");
  for (const relPath of legacyFiles) copyFile(scratch, relPath);
  for (const crate of scannedCrates) {
    writeFile(
      scratch,
      `crates/${crate}/Cargo.toml`,
      `[package]\nname = "${crate}"\nversion = "0.0.0"\nedition = "2024"\n`,
    );
  }
  writeFile(scratch, "crates/vize_atelier_core/src/lib.rs", "use vize_s0::Root;\n");
  return scratch;
}

function runSurfaceCommand(root, mode) {
  return spawnSync("rust-script", [command, mode], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, VIZE_REPO_ROOT: root },
  });
}

void test("consumer migration surface check fails on an injected stale artifact", () => {
  const scratch = createScratchRepo();
  try {
    const write = runSurfaceCommand(scratch, "--write");
    assert.equal(write.status, 0, `${write.stdout}${write.stderr}`.trim());

    const clean = runSurfaceCommand(scratch, "--check");
    assert.equal(clean.status, 0, `${clean.stdout}${clean.stderr}`.trim());

    // The scanned `use vize_s0::Root;` lands in its (consumer, crate) shard
    // and nowhere else.
    const shard = fs.readFileSync(path.join(scratch, coreShard), "utf8").split("\n");
    assert.deepEqual(shard.slice(1), [
      "compiler\tCompiler\tsource\tcrates/vize_atelier_core/src/lib.rs\t1\ts0\tS0\tstage\tvize_s0\tpreferred\t1",
      "",
    ]);

    fs.appendFileSync(path.join(scratch, coreShard), "injected\tedit\n");
    const stale = runSurfaceCommand(scratch, "--check");

    assert.equal(stale.status, 1, `--check accepted stale artifacts:\n${stale.stdout}`);
    assert.match(
      stale.stderr,
      /stale: docs\/davinci\/plan\/consumer-migration-surfaces\/compiler\/vize_atelier_core\.tsv drifted/,
    );
    assert.match(
      stale.stderr,
      /Regenerate with: rust-script tools\/commands\/davinci\/consumer-migration-surfaces\.rs --write/,
    );

    // A shard the generator no longer produces is stale too, and --write
    // deletes it: the shard set is gated as strictly as the shard bytes.
    assert.equal(runSurfaceCommand(scratch, "--write").status, 0);
    const leftover = path.join(scratch, shardDir, "compiler/vize_removed_crate.tsv");
    fs.writeFileSync(leftover, "consumer_id\n");
    const stray = runSurfaceCommand(scratch, "--check");
    assert.equal(stray.status, 1, `--check accepted a leftover shard:\n${stray.stdout}`);
    assert.match(
      stray.stderr,
      /stale: docs\/davinci\/plan\/consumer-migration-surfaces\/compiler\/vize_removed_crate\.tsv is not produced by the generator/,
    );
    assert.equal(runSurfaceCommand(scratch, "--write").status, 0);
    assert.equal(fs.existsSync(leftover), false);
    assert.equal(runSurfaceCommand(scratch, "--check").status, 0);

    // Cross-file totals are computed on demand, never committed.
    const summary = runSurfaceCommand(scratch, "--summary");
    assert.equal(summary.status, 0, `${summary.stdout}${summary.stderr}`.trim());
    assert.match(summary.stdout, /\| Compiler +\| +1 \| +1 \| +0 \|/);
    const index = fs.readFileSync(
      path.join(scratch, "docs/davinci/plan/consumer-migration-surfaces.md"),
      "utf8",
    );
    assert.equal(index.includes("## Consumer summary"), false);
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
});
