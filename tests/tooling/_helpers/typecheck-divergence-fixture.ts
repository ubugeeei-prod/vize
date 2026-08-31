import os from "node:os";
import path from "node:path";

import { compareTypecheckDiagnostics } from "../../../legacy-tools/fixtures/typecheck-divergence.mjs";

/** Workspace root the comparator normalizes fixture paths against. */
export const cwd = path.join(os.tmpdir(), "vize-divergence-workspace");

export function compare(
  files: Array<{ file: string; diagnostics: string[] }>,
  vueTscOutput: string,
  documentedDifferences?: unknown[],
) {
  return compareTypecheckDiagnostics({
    projectId: "fixture",
    cwd,
    vizeReport: { files },
    vueTscOutput,
    documentedDifferences,
  });
}

/**
 * A reviewable ledger entry for the spelling-suggestion divergence of #3358:
 * vize and vue-tsc report different codes at the same span, so it exercises
 * cancellation against both the false-positive and the false-negative bucket.
 */
export const suggestionDifference = {
  project: "fixture",
  file: "src/App.vue",
  severity: "error",
  line: 5,
  column: 16,
  vize: { code: 2552, message: "Cannot find name 'useRouter'. Did you mean 'router'?" },
  baseline: { code: 2304, message: "Cannot find name 'useRouter'." },
  issue: 3358,
  reason: "vue-tsc exhausts its per-program spelling-suggestion budget before this span.",
};

export const suggestionFiles = [
  {
    file: "src/App.vue",
    diagnostics: ["error:5:16 [TS2552] Cannot find name 'useRouter'. Did you mean 'router'?"],
  },
];

export const suggestionBaseline =
  "src/App.vue(5,16): error TS2304: Cannot find name 'useRouter'.\n";
