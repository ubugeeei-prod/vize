import assert from "node:assert/strict";
import { test } from "node:test";
import { readFileSync } from "node:fs";
import { formatSpeedup } from "../../tools/benchmarks/scripts/compare-tools-report.mjs";
import { stagedSnapshotFiles } from "./support/benchmark-staging.ts";

test("ratio formatting never rounds a regression to parity or zero", () => {
  for (const [value, expected] of [
    [0.9546, "0.95x"],
    [0.9999, "0.9999x"],
    [0.000001, "0.000001x"],
    [1, "1.0x"],
    [1.514, "1.5x"],
    [NaN, "n/a"],
  ] as const)
    assert.equal(formatSpeedup(value), expected);
});

test("all published summary tables retain the measured Nuxt slowdown", () => {
  for (const locale of ["", "ja/", "zh-CN/", "fr/", "pt-BR/"]) {
    const page = readFileSync(
      new URL(
        `../../docs/content/${locale}architecture/performance-blacksmith.md`,
        import.meta.url,
      ),
      "utf8",
    );
    assert.ok(
      page
        .split("\n")
        .some((line) => line.startsWith("|") && /Nuxt/.test(line) && /0\.95x/.test(line)),
    );
  }
  const readme = readFileSync(new URL("../../README.md", import.meta.url), "utf8");
  assert.ok(
    readme.split("\n").some((line) => line.startsWith("| Nuxt build") && /0\.95×/.test(line)),
  );
});

test("staging evidence accepts equivalent shell spellings and notices omitted files", () => {
  const files = ["README.md", "docs/performance.md", "docs/performance-blacksmith.md"].sort();
  for (const command of [
    "git add README.md docs/performance{,-blacksmith}.md",
    "git add docs/performance.md README.md docs/performance-blacksmith.md",
  ])
    assert.deepEqual(stagedSnapshotFiles(command, files), files);
  assert.deepEqual(stagedSnapshotFiles("git add README.md", files), ["README.md"]);
  assert.throws(() => stagedSnapshotFiles("echo git add README.md", files), /one git add/);
});
