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

test("all published summary tables retain the measured Nuxt ratio", () => {
  const latest = JSON.parse(
    readFileSync(
      new URL("../../tools/benchmarks/results/tool-benchmark-latest.json", import.meta.url),
      "utf8",
    ),
  );
  const nuxt = latest.surfaces.find(({ id }: { id: string }) => id === "nuxt");
  const ratio = formatSpeedup(nuxt.primarySpeedup);
  const cell = (glyph: string) =>
    new RegExp(`(?<![\\d.])${ratio.slice(0, -1).replace(".", "\\.")}${glyph}(?!\\d)`);
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
        .some((line) => line.startsWith("|") && /Nuxt/.test(line) && cell("x").test(line)),
    );
  }
  const readme = readFileSync(new URL("../../README.md", import.meta.url), "utf8");
  assert.ok(
    readme.split("\n").some((line) => line.startsWith("| Nuxt build") && cell("×").test(line)),
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
