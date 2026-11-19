import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const generator = "tools/support/compat/davinci/consumer-migration-surfaces.ts";
const legacyFiles = [
  generator,
  "tools/support/compat/davinci/lib/artifact-set.ts",
  "tools/support/compat/davinci/lib/consumer-migration-render.ts",
  "tools/support/compat/davinci/lib/consumer-migration-scan.ts",
  "tools/support/compat/davinci/lib/consumer-migration-summary.ts",
  "tools/support/compat/davinci/lib/markdown.ts",
  "tools/support/compat/davinci/lib/ordering.ts",
  "tools/support/compat/davinci/lib/paths.ts",
  "tools/support/compat/davinci/lib/rust-source.ts",
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
const runShard =
  "artifacts/davinci-ledgers/consumer-migration-surfaces/compiler/vize_atelier_core.tsv";
const committedShard = `${shardDir}/compiler/vize_atelier_core.tsv`;

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
  writeFile(scratch, "crates/vize_atelier_core/src/lib.rs", "use vize_l0::Root;\n");
  return scratch;
}

function runSurfaceCommand(root, mode) {
  // The scratch copy resolves its repository root from its own location.
  return spawnSync(process.execPath, [path.join(root, generator), mode], {
    cwd: root,
    encoding: "utf8",
  });
}

void test("consumer migration surfaces are run artifacts and a committed shard is stale", () => {
  const scratch = createScratchRepo();
  try {
    const write = runSurfaceCommand(scratch, "--write");
    assert.equal(write.status, 0, `${write.stdout}${write.stderr}`.trim());
    const clean = runSurfaceCommand(scratch, "--check");
    assert.equal(clean.status, 0, `${clean.stdout}${clean.stderr}`.trim());
    // The scanned `use vize_l0::Root;` lands in its (consumer, crate) shard,
    // which every run writes under the uploaded ledger directory.
    const shard = fs.readFileSync(path.join(scratch, runShard), "utf8").split("\n");
    assert.deepEqual(shard.slice(1), [
      "compiler\tCompiler\tsource\tcrates/vize_atelier_core/src/lib.rs\t1\tl0\tL0\tstage\tvize_l0\tpreferred\t1",
      "",
    ]);
    assert.equal(fs.existsSync(path.join(scratch, committedShard)), false);

    // The committed index is still byte-compared.
    const indexPath = path.join(scratch, "docs/davinci/plan/consumer-migration-surfaces.md");
    fs.appendFileSync(indexPath, "injected edit\n");
    const stale = runSurfaceCommand(scratch, "--check");
    assert.equal(stale.status, 1, `--check accepted a stale index:\n${stale.stdout}`);
    assert.match(
      stale.stderr,
      /stale: docs\/davinci\/plan\/consumer-migration-surfaces\.md drifted/,
    );
    assert.match(
      stale.stderr,
      /Regenerate with: node tools\/support\/compat\/davinci\/consumer-migration-surfaces\.ts --write/,
    );

    // A shard committed under the plan directory is stale, and --write deletes it.
    assert.equal(runSurfaceCommand(scratch, "--write").status, 0);
    const leftover = path.join(scratch, committedShard);
    fs.mkdirSync(path.dirname(leftover), { recursive: true });
    fs.writeFileSync(leftover, "consumer_id\n");
    const stray = runSurfaceCommand(scratch, "--check");
    assert.equal(stray.status, 1, `--check accepted a committed shard:\n${stray.stdout}`);
    assert.match(
      stray.stderr,
      /stale: docs\/davinci\/plan\/consumer-migration-surfaces\/compiler\/vize_atelier_core\.tsv is not produced by the generator/,
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
