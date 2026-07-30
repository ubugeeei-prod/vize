import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { CROSS_ENGINE_CELL, ENGINE_CLASSES_BY_SURFACE } from "../../bench/compare-tools-report.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const resultsPath = path.join(root, "bench/results/tool-benchmark-latest.json");

type Surface = {
  id: string;
  label: string;
  primarySpeedup: number | null;
  speedupStatus: string;
  engineClasses?: Record<string, string>;
  variants: { id: string }[];
};

function readResults(): { surfaces: Surface[] } {
  return JSON.parse(fs.readFileSync(resultsPath, "utf8"));
}

function tableCells(line: string): string[] {
  return line
    .split("|")
    .slice(1, -1)
    .map((cell) => cell.trim());
}

/**
 * Every published table has the speedup as the LAST cell of a row. Rows are
 * located by the label the translation uses, so the test does not depend on
 * column widths or on how a locale words the rest of the row.
 */
function speedupCell(file: string, rowLabel: string): string {
  const line = fs
    .readFileSync(path.join(root, file), "utf8")
    .split("\n")
    .find((candidate) => candidate.startsWith("|") && candidate.includes(rowLabel));
  assert.ok(line != null, `${file} has no table row labelled ${rowLabel}`);
  const cells = tableCells(line);
  return cells[cells.length - 1];
}

// One entry per published table row that compares a JS TypeScript engine
// against the native one. Adding a locale means adding it here.
const CROSS_ENGINE_ROWS: [file: string, rowLabel: string][] = [
  ["README.md", "| Type check "],
  ["docs/content/architecture/performance-blacksmith.md", "| Large SFC type check "],
  ["docs/content/architecture/performance-blacksmith.md", "| Type check "],
  ["docs/content/ja/architecture/performance-blacksmith.md", "| 大型SFCタイプチェック "],
  ["docs/content/ja/architecture/performance-blacksmith.md", "| タイプチェック "],
  ["docs/content/zh-CN/architecture/performance-blacksmith.md", "|大型SFC类型检查"],
  ["docs/content/zh-CN/architecture/performance-blacksmith.md", "|类型检查 "],
  [
    "docs/content/pt-BR/architecture/performance-blacksmith.md",
    "| Verificação grande do tipo SFC ",
  ],
  ["docs/content/pt-BR/architecture/performance-blacksmith.md", "| Verificação de tipo "],
  ["docs/content/fr/architecture/performance-blacksmith.md", "| Contrôle de type grand SFC "],
  ["docs/content/fr/architecture/performance-blacksmith.md", "| Contrôle de type  "],
];

test("no published table states a cross-engine type-check speedup", () => {
  assert.deepEqual(
    CROSS_ENGINE_ROWS.map(([file, rowLabel]) => [file, rowLabel, speedupCell(file, rowLabel)]),
    CROSS_ENGINE_ROWS.map(([file, rowLabel]) => [file, rowLabel, CROSS_ENGINE_CELL]),
  );
});

/**
 * `docs/content/architecture/performance.md` and its translations are NOT
 * rendered from `bench/results/tool-benchmark-latest.json` — they are
 * hand-maintained from a separate local bench (`bench/check.ts`). That is why
 * #3431's retraction did not reach them and they were left as the only surface
 * in the repo still publishing `8.6x` / `8.9x` for the same cross-engine
 * comparison README and the snapshot had already withdrawn. Each entry is a
 * page and the header cell that identifies its vue-tsc comparison table.
 */
const HAND_MAINTAINED_TYPE_CHECK_TABLES: [file: string, headerCell: string][] = [
  ["docs/content/architecture/performance.md", "vue-tsc (ST)"],
  ["docs/content/fr/architecture/performance.md", "vue-tsc (ST)"],
  ["docs/content/ja/architecture/performance.md", "vue-tsc (ST)"],
  ["docs/content/pt-BR/architecture/performance.md", "vue-tsc (ST)"],
  ["docs/content/zh-CN/architecture/performance.md", "vue-tsc （ST）"],
];

/**
 * The ratio cells of a hand-maintained type-check table.
 *
 * That table has three ratio columns, not one, so the last-cell rule above does
 * not cover it. The ratio columns are identified structurally instead of
 * positionally: they are the columns the `Rate` row leaves blank, which holds in
 * every translation regardless of how the headers are worded.
 */
function typeCheckRatioCells(file: string, headerCell: string): string[] {
  const lines = fs.readFileSync(path.join(root, file), "utf8").split("\n");
  const headerIndex = lines.findIndex(
    (line) => line.startsWith("|") && tableCells(line).includes(headerCell),
  );
  assert.notEqual(headerIndex, -1, `${file} has no ${headerCell} comparison table`);

  const timeRow = tableCells(lines[headerIndex + 2]);
  const rateRow = tableCells(lines[headerIndex + 3]);
  assert.equal(timeRow.length, rateRow.length, `${file}'s type-check table is ragged`);
  return timeRow.filter((_, index) => index > 0 && rateRow[index] === "");
}

test("no hand-maintained page states a cross-engine type-check speedup either", () => {
  assert.deepEqual(
    HAND_MAINTAINED_TYPE_CHECK_TABLES.map(([file, headerCell]) => [
      file,
      typeCheckRatioCells(file, headerCell),
    ]),
    HAND_MAINTAINED_TYPE_CHECK_TABLES.map(([file]) => [
      file,
      [CROSS_ENGINE_CELL, CROSS_ENGINE_CELL, CROSS_ENGINE_CELL],
    ]),
  );
});

test("the committed snapshot marks exactly the cross-engine surfaces as unranked", () => {
  const { surfaces } = readResults();

  assert.deepEqual(
    surfaces
      .filter((surface) => surface.engineClasses != null)
      .map((surface) => [surface.id, surface.speedupStatus, surface.primarySpeedup]),
    [
      ["large-check", "cross-engine", null],
      ["check", "cross-engine", null],
    ],
  );

  // The classification the snapshot carries is the one the generator uses, so
  // a re-render cannot silently reclassify a row.
  assert.deepEqual(
    surfaces
      .filter((surface) => surface.engineClasses != null)
      .map((surface) => surface.engineClasses),
    [ENGINE_CLASSES_BY_SURFACE["large-check"], ENGINE_CLASSES_BY_SURFACE.check],
  );
});

test("every surface the generator classifies is classified in the snapshot", () => {
  const { surfaces } = readResults();

  assert.deepEqual(
    surfaces
      .filter((surface) => ENGINE_CLASSES_BY_SURFACE[surface.id] != null)
      .map((surface) => surface.id)
      .sort(),
    Object.keys(ENGINE_CLASSES_BY_SURFACE).sort(),
  );
});

test("same-engine surfaces keep publishing a ranked speedup", () => {
  const { surfaces } = readResults();

  assert.deepEqual(
    surfaces
      .filter((surface) => surface.engineClasses == null)
      .map((surface) => [surface.id, surface.speedupStatus, typeof surface.primarySpeedup]),
    [
      ["compile", "ranked", "number"],
      ["large-compile", "ranked", "number"],
      ["lint", "ranked", "number"],
      ["fmt", "ranked", "number"],
      ["vite", "ranked", "number"],
      ["nuxt", "ranked", "number"],
    ],
  );
});
