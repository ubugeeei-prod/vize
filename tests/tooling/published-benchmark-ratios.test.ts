import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { CROSS_ENGINE_CELL, ENGINE_CLASSES_BY_SURFACE } from "../../tools/benchmarks/scripts/compare-tools-report.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const resultsPath = path.join(root, "tools/benchmarks/results/tool-benchmark-latest.json");

type Surface = {
  id: string;
  label: string;
  baselineId: string;
  vizeMaxId: string;
  primarySpeedup: number | null;
  speedupBaselineId: string | null;
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

// One entry per published table row for a surface that spans engine classes,
// and the snapshot surface it is rendered from. Adding a locale means adding
// it here.
const TYPE_CHECK_ROWS: [file: string, rowLabel: string, surfaceId: string][] = [
  ["README.md", "| Type check ", "check"],
  ["docs/content/architecture/performance-blacksmith.md", "| Large SFC type check ", "large-check"],
  ["docs/content/architecture/performance-blacksmith.md", "| Type check ", "check"],
  [
    "docs/content/ja/architecture/performance-blacksmith.md",
    "| 大型SFCタイプチェック ",
    "large-check",
  ],
  ["docs/content/ja/architecture/performance-blacksmith.md", "| タイプチェック ", "check"],
  ["docs/content/zh-CN/architecture/performance-blacksmith.md", "|大型SFC类型检查", "large-check"],
  ["docs/content/zh-CN/architecture/performance-blacksmith.md", "|类型检查 ", "check"],
  [
    "docs/content/pt-BR/architecture/performance-blacksmith.md",
    "| Verificação grande do tipo SFC ",
    "large-check",
  ],
  ["docs/content/pt-BR/architecture/performance-blacksmith.md", "| Verificação de tipo ", "check"],
  [
    "docs/content/fr/architecture/performance-blacksmith.md",
    "| Contrôle de type grand SFC ",
    "large-check",
  ],
  ["docs/content/fr/architecture/performance-blacksmith.md", "| Contrôle de type  ", "check"],
];

/**
 * The ratio a published cell states, or `null` when the cell is not a ratio.
 * Locales format it as `2.0x`, `2,0x` or `**2.0×**`, so the number is parsed
 * out rather than the whole cell being compared to one rendering.
 */
function statedRatio(cell: string): number | null {
  const match = /^\*{0,2}(\d+[.,]\d+)\s*[x×]\*{0,2}$/u.exec(cell);
  return match == null ? null : Number(match[1].replace(",", "."));
}

function surfaceOf(surfaces: Surface[], id: string): Surface {
  const surface = surfaces.find((candidate) => candidate.id === id);
  assert.ok(surface, `snapshot is missing the ${id} surface`);
  return surface;
}

/**
 * A published table may state a ratio only when the snapshot published one,
 * and it must be that ratio. `n/a (cross-engine)` is not a formatting choice
 * to be edited by hand either: it appears exactly when the run had no
 * incumbent on the Vize lane's own engine to compare against.
 */
test("every published table states the type-check ratio the snapshot states", () => {
  const { surfaces } = readResults();

  assert.deepEqual(
    TYPE_CHECK_ROWS.map(([file, rowLabel]) => {
      const cell = speedupCell(file, rowLabel);
      return [file, rowLabel, statedRatio(cell) ?? cell];
    }),
    TYPE_CHECK_ROWS.map(([file, rowLabel, surfaceId]) => {
      const surface = surfaceOf(surfaces, surfaceId);
      const ratio = surface.primarySpeedup;
      return [file, rowLabel, ratio == null ? CROSS_ENGINE_CELL : Number(ratio.toFixed(1))];
    }),
  );
});

/**
 * `docs/content/architecture/performance.md` and its translations are NOT
 * rendered from `tools/benchmarks/results/tool-benchmark-latest.json` — they are
 * hand-maintained from a separate local bench (`tools/benchmarks/scripts/check.ts`). That is why
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

test("the committed snapshot never rates one engine class against another", () => {
  const { surfaces } = readResults();
  const classified = surfaces.filter((surface) => surface.engineClasses != null);

  assert.deepEqual(
    classified.map((surface) => surface.id),
    ["large-check", "check"],
  );

  for (const surface of classified) {
    const classes = surface.engineClasses ?? {};
    if (surface.speedupBaselineId == null) {
      // Nothing on the Vize lane's engine ran, so nothing may be published.
      assert.deepEqual(
        [surface.id, surface.speedupStatus, surface.primarySpeedup],
        [surface.id, "cross-engine", null],
      );
      continue;
    }
    assert.deepEqual(
      [surface.id, surface.speedupStatus, classes[surface.speedupBaselineId]],
      [surface.id, "in-class", classes[surface.vizeMaxId]],
    );
  }

  // The classification the snapshot carries is the one the generator uses, so
  // a re-render cannot silently reclassify a row.
  assert.deepEqual(
    classified.map((surface) => surface.engineClasses),
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
