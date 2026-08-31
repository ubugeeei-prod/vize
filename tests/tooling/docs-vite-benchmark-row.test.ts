/**
 * `docs/content/architecture/performance.md` published a third, independent Vite
 * figure -- `957ms / 479ms / 2.0x` -- sourced from `tools/benchmarks/scripts/vite.ts`. #3392 showed
 * that harness timed Vize with a warm persistent pre-compile cache inherited from
 * its own warmup while `@vitejs/plugin-vue` compiled from scratch, so the ratio was
 * not apples-to-apples; #3392 also split the harness output into separate cold and
 * warm rows, so the single quoted figure stopped being reproducible even in form.
 * README.md and `performance-blacksmith.md` had already been reconciled against
 * `tools/benchmarks/results/tool-benchmark-latest.json` (#3422, #3431) and pinned by
 * `readme-benchmark-rows.test.ts`; this surface, and its four localized copies,
 * were missed.
 *
 * This pins all five copies to the same artifact README is pinned to, so the Vite
 * number cannot drift between the README, the Blacksmith snapshot page, and the
 * performance page again.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

/** Every locale copy of the performance page, relative to the repo root. */
const PERFORMANCE_PAGES = [
  "docs/content/architecture/performance.md",
  "docs/content/fr/architecture/performance.md",
  "docs/content/ja/architecture/performance.md",
  "docs/content/pt-BR/architecture/performance.md",
  "docs/content/zh-CN/architecture/performance.md",
];

type Variant = { id: string; medianMs: number };
type Surface = { id: string; files: number; primarySpeedup: number; variants: Variant[] };

function viteSurface(): Surface {
  const surfaces = JSON.parse(
    fs.readFileSync(
      path.join(repoRoot, "tools/benchmarks/results/tool-benchmark-latest.json"),
      "utf8",
    ),
  ).surfaces as Surface[];
  const surface = surfaces.find((candidate) => candidate.id === "vite");
  assert.ok(surface, "snapshot is missing the vite surface");
  return surface;
}

function medianMs(surface: Surface, variantId: string): number {
  const variant = surface.variants.find((candidate) => candidate.id === variantId);
  assert.ok(variant, `snapshot is missing the ${variantId} variant`);
  return variant.medianMs;
}

/** The README's own formatting: `1.66s` above a second, `732.5ms` below it. */
function formatMs(ms: number): string {
  return ms >= 1000
    ? `${(ms / 1000).toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}s`
    : `${ms.toLocaleString("en-US", { minimumFractionDigits: 1, maximumFractionDigits: 1 })}ms`;
}

function splitRow(line: string): string[] {
  return line
    .slice(1, -1)
    .split("|")
    .map((cell) => cell.trim());
}

/**
 * The three measured cells of the page's Vite comparison table.
 *
 * The row label is translated per locale, so the table is located by its header
 * -- which names the two plugins and is identical in every copy -- and the data
 * row is the first non-separator row after it.
 */
function readViteRowMeasurements(page: string): string[] {
  const lines = fs.readFileSync(path.join(repoRoot, page), "utf8").split("\n");
  const headerIndex = lines.findIndex((line) => {
    if (!line.startsWith("| ") || !line.endsWith(" |")) {
      return false;
    }
    const cells = splitRow(line);
    return cells.includes("@vitejs/plugin-vue") && cells.includes("@vizejs/vite-plugin");
  });
  assert.notEqual(
    headerIndex,
    -1,
    `${page} has no @vitejs/plugin-vue vs @vizejs/vite-plugin table`,
  );

  const dataRow = lines[headerIndex + 2];
  assert.ok(dataRow?.startsWith("| "), `${page}'s Vite table has no data row`);
  return splitRow(dataRow).slice(1);
}

test("every locale's performance page publishes the snapshot's Vite row", () => {
  const surface = viteSurface();
  const expected = [
    formatMs(medianMs(surface, "vite-plugin-vue")),
    formatMs(medianMs(surface, "vize-vite-plugin")),
    `**${surface.primarySpeedup.toFixed(1)}x**`,
  ];

  assert.deepEqual(
    PERFORMANCE_PAGES.map((page) => [page, readViteRowMeasurements(page)]),
    PERFORMANCE_PAGES.map((page) => [page, expected]),
  );
});

test("no locale's performance page still quotes the pre-#3392 tools/benchmarks/scripts/vite.ts figure", () => {
  // `957ms`/`479ms` were the harness's numbers, not the artifact's. They may only
  // appear inside the paragraph that explains why they were retracted, which
  // quotes them in backticks; a bare occurrence means a table row came back.
  assert.deepEqual(
    PERFORMANCE_PAGES.flatMap((page) => {
      const lines = fs.readFileSync(path.join(repoRoot, page), "utf8").split("\n");
      return lines
        .filter((line) => line.startsWith("| ") && /(?<!\d)(?:957|479)(?!\d)/.test(line))
        .map((line) => [page, line]);
    }),
    [],
  );
});
