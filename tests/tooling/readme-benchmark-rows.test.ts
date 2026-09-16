/**
 * The README benchmark table and the generated Blacksmith snapshot are two
 * renderings of the same measurements, and they had drifted: the README
 * published the Vite row as `1.70s / 1.52s / 1.1x` and the Nuxt row as
 * `6.68s / 7.35s / 0.9x` while `tools/benchmarks/results/tool-benchmark-latest.json` --
 * the artifact `docs/content/architecture/performance-blacksmith.md` is
 * rendered from -- held `1.66s / 732.5ms / 2.3x` and `6.79s / 6.42s / 1.1x`.
 * A reader got a different answer depending on which page they opened.
 *
 * Every row is now published from the same validated artifact, including the
 * same-native-engine type-check comparison against verter-tsc.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

type Surface = {
  id: string;
  files: number;
  primarySpeedup: number;
  variants: { id: string; medianMs: number }[];
};

function readSnapshot(): Surface[] {
  return JSON.parse(
    fs.readFileSync(
      path.join(repoRoot, "tools/benchmarks/results/tool-benchmark-latest.json"),
      "utf8",
    ),
  ).surfaces as Surface[];
}

/** README table rows, keyed by their first cell. */
function readReadmeRows(): Map<string, string[]> {
  const readme = fs.readFileSync(path.join(repoRoot, "README.md"), "utf8");
  const rows = new Map<string, string[]>();
  for (const line of readme.split("\n")) {
    if (!line.startsWith("| ") || !line.endsWith(" |")) {
      continue;
    }
    const cells = line
      .slice(1, -1)
      .split("|")
      .map((cell) => cell.trim());
    if (cells.length === 6 && !cells[0].startsWith("-")) {
      rows.set(cells[0], cells);
    }
  }
  return rows;
}

/** The README's own formatting: `1.66s` above a second, `732.5ms` below it. */
function formatMs(ms: number): string {
  return ms >= 1000
    ? `${(ms / 1000).toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}s`
    : `${ms.toLocaleString("en-US", { minimumFractionDigits: 1, maximumFractionDigits: 1 })}ms`;
}

function expectedRow(
  surfaces: Surface[],
  surfaceId: string,
  label: string,
  tool: string,
  baselineId: string,
  vizeId: string,
  speedupSuffix = "",
): string[] {
  const surface = surfaces.find((candidate) => candidate.id === surfaceId);
  assert.ok(surface, `snapshot is missing the ${surfaceId} surface`);
  const variant = (id: string) => {
    const found = surface.variants.find((candidate) => candidate.id === id);
    assert.ok(found, `snapshot is missing the ${id} variant`);
    return found;
  };
  return [
    label,
    surface.files.toLocaleString("en-US"),
    tool,
    formatMs(variant(baselineId).medianMs),
    formatMs(variant(vizeId).medianMs),
    `**${surface.primarySpeedup.toFixed(1)}×**${speedupSuffix}`,
  ];
}

test("every README row matches the committed benchmark snapshot", () => {
  const surfaces = readSnapshot();
  const rows = readReadmeRows();

  assert.deepEqual(
    ["SFC compile", "Lint", "Format", "Type check", "Vite build", "Nuxt build"].map((label) =>
      rows.get(label),
    ),
    [
      expectedRow(
        surfaces,
        "compile",
        "SFC compile",
        "@vue/compiler-sfc",
        "vue-compiler-sfc-1t",
        "vize-native-max",
      ),
      expectedRow(
        surfaces,
        "lint",
        "Lint",
        "eslint-plugin-vue",
        "eslint-plugin-vue-1t",
        "vize-lint-max",
      ),
      expectedRow(surfaces, "fmt", "Format", "Prettier", "prettier-cli", "vize-fmt-max"),
      expectedRow(surfaces, "check", "Type check", "verter-tsc", "verter-tsc", "vize-check-max"),
      expectedRow(
        surfaces,
        "vite",
        "Vite build",
        "@vitejs/plugin-vue",
        "vite-plugin-vue",
        "vize-vite-plugin",
      ),
      expectedRow(surfaces, "nuxt", "Nuxt build", "Nuxt compiler", "nuxt-default", "vize-nuxt"),
    ],
  );
});

// The header used to assert "15,000 generated Vue SFCs" for the whole table
// while the type-check, Vite, and Nuxt rows ran at 500, 1,000, and 500 files.
// Those are exactly the three rows that look weakest, so the overstatement made
// them look worse than they are on a corpus they never used.
test("the README states a per-row corpus size instead of one blanket figure", () => {
  const readme = fs.readFileSync(path.join(repoRoot, "README.md"), "utf8");
  const benchmarks = readme.slice(readme.indexOf("## Benchmarks"));
  const heading = benchmarks.slice(0, benchmarks.indexOf("| Surface"));

  assert.equal(
    /\b15,000 generated Vue SFCs\b/.test(heading),
    false,
    "the heading must not claim a single corpus size for rows measured at different sizes",
  );

  const rows = readReadmeRows();
  const surfaces = readSnapshot();
  const filesFor = (id: string) => {
    const surface = surfaces.find((candidate) => candidate.id === id);
    assert.ok(surface, `snapshot is missing the ${id} surface`);
    return surface.files.toLocaleString("en-US");
  };

  assert.deepEqual(
    [
      rows.get("SFC compile")?.[1],
      rows.get("Lint")?.[1],
      rows.get("Format")?.[1],
      rows.get("Type check")?.[1],
      rows.get("Vite build")?.[1],
      rows.get("Nuxt build")?.[1],
    ],
    [
      filesFor("compile"),
      filesFor("lint"),
      filesFor("fmt"),
      filesFor("check"),
      filesFor("vite"),
      filesFor("nuxt"),
    ],
  );
});
