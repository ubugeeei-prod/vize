import os from "node:os";
import path from "node:path";

import { compareLintFindings } from "../../../legacy-tools/fixtures/lint-divergence.mjs";

/** Workspace root the comparator normalizes fixture paths against. */
export const cwd = path.join(os.tmpdir(), "vize-patina-divergence-workspace");

export const repoRoot = path.resolve(import.meta.dirname, "..", "..", "..");

export const ledgerPath = path.join(
  repoRoot,
  "tests",
  "_fixtures",
  "patina-lint-documented-divergences.json",
);

/**
 * A minimal stand-in for the checked-in rule map: one entry per status, so the
 * routing of every bucket is exercised without depending on which upstream
 * rules happen to be implemented today.
 */
export const ruleMap = {
  upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
  entries: {
    "vue/no-v-html": { status: "mapped", patinaRule: "vue/no-v-html" },
    "vue/require-v-for-key": { status: "mapped", patinaRule: "vue/require-v-for-key" },
    "vue/no-unused-components": { status: "mapped", patinaRule: "vue/no-unused-components" },
    "vue/attributes-order": { status: "mapped", patinaRule: "vue/attribute-order" },
    "vue/no-undef-properties": { status: "unimplemented", issue: 3223 },
    "vue/max-attributes-per-line": {
      status: "intentional-divergence",
      reason: "formatting-category rule owned by glyph",
    },
  },
};

export function span(line: number, column: number, endLine: number, endColumn: number) {
  return { line, column, endLine, endColumn };
}

export function eslintResult(file: string, messages: unknown[]) {
  return { filePath: path.join(cwd, file), messages };
}

export function compare(patinaFindings: unknown[], eslintResults: unknown[], ledger?: unknown[]) {
  return compareLintFindings({
    projectId: "fixture",
    cwd,
    ruleMap,
    patinaFindings,
    eslintResults,
    documentedDivergences: ledger,
  });
}
