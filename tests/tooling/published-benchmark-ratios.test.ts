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
  const cells = line
    .split("|")
    .slice(1, -1)
    .map((cell) => cell.trim());
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
