// P4-3c: real CLI recall on both original and independent class-b mutation
// trees. The seed tool compares complete diagnostic multisets, including
// baseline shifts; a different rule at the right span cannot satisfy recall.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  defuseSuppressions,
  intersectSuppressions,
  loadRuleMap,
  scanSuppressions,
} from "../../tools/support/compat/davinci/lib/fpfn-suppress-scan.ts";
import {
  flattenLintJson,
  resolveVizeCli,
  runVizeLintJson,
} from "../../tools/support/compat/davinci/lib/fpfn-shared.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const script = path.join(root, "tools/commands/davinci/seed-defects.rs");
const shard = ["splitpanes", "layoutit-grid", "cssgridgenerator"];

for (const [mode, expected] of [
  ["matrix", 90],
  ["corpus-shard", 130],
] as const) {
  test(`unused setup bindings recall every ${mode} seed by identity`, (t) => {
    if (
      mode === "corpus-shard" &&
      shard.some((name) => {
        const dir = path.join(root, "tests/_fixtures/_git", name);
        return !fs.existsSync(dir) || fs.readdirSync(dir).length === 0;
      })
    ) {
      t.skip(
        "P0-13 shard is not hydrated; Davinci unused bindings Actions hydrates all three projects",
      );
      return;
    }
    const out = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-unused-"));
    try {
      const result = spawnSync(
        "rust-script",
        [
          script,
          `--${mode}`,
          "--out",
          out,
          "--assert",
          "--report",
          path.join(out, "assert-report.json"),
        ],
        {
          cwd: root,
          encoding: "utf8",
          timeout: 120_000,
          maxBuffer: 64 * 1024 * 1024,
        },
      );
      if (result.error) throw result.error;
      const report = JSON.parse(fs.readFileSync(path.join(out, "assert-report.json"), "utf8"));
      assert.equal(report.scope.filesCopied, expected, result.stdout);
      assert.equal(report.classB.expected, expected, result.stdout);
      assert.equal(report.classB.detected, expected, result.stdout);
      assert.deepEqual(report.classB.misses, [], result.stdout);
      assert.deepEqual(report.baselineShift.misses, [], result.stdout);
      assert.deepEqual(report.baselineShift.unmappable, [], result.stdout);
      assert.deepEqual(report.unexpected, [], result.stdout);
      t.diagnostic(
        `class-b=${report.classB.detected}/${report.classB.expected} baseline-mapped=${report.baselineShift.mapped} misses=0 unmappable=0 unexpected=0`,
      );
      if (mode === "corpus-shard") {
        const manifest = JSON.parse(fs.readFileSync(path.join(out, "manifest.json"), "utf8"));
        const files = manifest.files.map((file: { path: string }) => file.path);
        const suppressions = new Map();
        const defused = path.join(out, "defused");
        for (const file of files) {
          const source = fs.readFileSync(path.join(out, "original", file), "utf8");
          suppressions.set(file, scanSuppressions(source));
          const target = path.join(defused, file);
          fs.mkdirSync(path.dirname(target), { recursive: true });
          fs.writeFileSync(target, defuseSuppressions(source).defused);
        }
        const rows = flattenLintJson(
          runVizeLintJson(
            resolveVizeCli(),
            defused,
            files,
            path.join(out, "class-b/original/vite.config.mjs"),
          ),
        );
        const diagnostics = new Map();
        for (const row of rows) {
          const group = diagnostics.get(row.path) ?? [];
          group.push(row);
          diagnostics.set(row.path, group);
        }
        assert.deepEqual(
          intersectSuppressions(diagnostics, suppressions, loadRuleMap()).candidates,
          [],
          "TS-38: opt-in consumer adds no corpus suppression candidate",
        );
      }
      // FN-1 remains a measured default-preset gap. Its misses do not weaken
      // class-(b)'s identity gate or add default-preset enablement in this change.
      assert.equal(
        result.status,
        report.classA.misses.length ? 1 : 0,
        `${result.stdout}\n${result.stderr}`,
      );
    } finally {
      fs.rmSync(out, { recursive: true, force: true });
    }
  });
}
