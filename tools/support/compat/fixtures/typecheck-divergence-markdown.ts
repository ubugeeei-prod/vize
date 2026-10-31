import { describeClassification, type BudgetArtifact } from "./typecheck-divergence-budget.ts";

type MarkdownSummary = BudgetArtifact["divergence"]["summary"] & {
  readonly vizeExcludedNonVueCount: number;
  readonly baselineExcludedNonVueCount: number;
  readonly baselineExcludedSupportVueCount: number;
  readonly baselineExcludedProjectCount: number;
  readonly baselineExcludedExternalCount: number;
};

interface MarkdownCoverage {
  readonly vizeVueFileCount: number;
  readonly baselineVueFileCount: number;
  readonly sharedVueFileCount: number;
  readonly missingVueFiles: readonly unknown[];
  readonly unexpectedVueFiles: readonly unknown[];
  readonly ignoredDependencyVueFileCount: number;
  readonly ignoredSupportVueFileCount: number;
}

interface MarkdownAmbient {
  readonly verdict: string;
  readonly unusableReason?: string | null;
  readonly vueRuntime: readonly {
    readonly name: string;
    readonly copies: readonly unknown[];
  }[];
}

type MarkdownMutation =
  | {
      readonly passed: true;
      readonly verdict: string;
      readonly file: string;
      readonly span: { readonly line: number; readonly column: number };
    }
  | { readonly passed?: false | null; readonly unusableReason?: string | null };

export interface TypecheckMarkdownArtifact {
  readonly project: string;
  readonly evidence: { readonly commitSha: string };
  readonly divergence: { readonly summary: MarkdownSummary; readonly sha256: string };
  readonly baseline: {
    readonly coverage: MarkdownCoverage;
    readonly configuration: { readonly errorCount: number };
    readonly ambient?: MarkdownAmbient | null;
  };
  readonly mutationOracle?: MarkdownMutation | null;
  readonly budget: BudgetArtifact["budget"];
}

/**
 * The human-readable half of a divergence run, split out of
 * `typecheck-divergence-report.mjs` so the report script stays inside the
 * per-file line budget.
 *
 * This is what a reviewer reads in the shard's step summary, so every field the
 * verdict depends on is printed: the diagnostic counts, both exclusion lenses,
 * the Vue-file coverage, and — since #3513 — how many configuration diagnostics
 * the baseline emitted. A shard that measured nothing has to say so here, not
 * only in the JSON nobody opens.
 *
 * Since #3738 it also prints the classification, because the numbers alone do
 * not carry it: 568 false positives measured against a baseline that never
 * loaded and 39 measured against one that did read identically in this table.
 */
export function renderMarkdown(artifact: TypecheckMarkdownArtifact): string {
  const summary = artifact.divergence.summary;
  const coverage = artifact.baseline.coverage;
  return [
    `## ${artifact.project} typecheck divergence`,
    "",
    `Commit: ${artifact.evidence.commitSha}`,
    `Vize diagnostics: ${summary.vizeDiagnosticCount}`,
    `vue-tsc diagnostics: ${summary.baselineDiagnosticCount}`,
    `Shared: ${summary.sharedCount}`,
    `Message mismatches: ${summary.messageMismatchCount}`,
    `Documented differences: ${summary.documentedDifferenceCount}`,
    `False positives: ${summary.falsePositiveCount} (${summary.falsePositiveRatio})`,
    `False negatives: ${summary.falseNegativeCount} (${summary.falseNegativeRatio})`,
    `Vize excluded non-Vue: ${summary.vizeExcludedNonVueCount}`,
    `vue-tsc excluded non-Vue: ${summary.baselineExcludedNonVueCount}`,
    `vue-tsc excluded support Vue: ${summary.baselineExcludedSupportVueCount}`,
    `vue-tsc excluded project-level: ${summary.baselineExcludedProjectCount}`,
    `vue-tsc excluded external: ${summary.baselineExcludedExternalCount}`,
    `vue-tsc configuration errors: ${artifact.baseline.configuration.errorCount}`,
    `vue-tsc ambient environment: ${describeAmbient(artifact.baseline.ambient)}`,
    `Vize Vue files: ${coverage.vizeVueFileCount}`,
    `vue-tsc Vue files: ${coverage.baselineVueFileCount}`,
    `Shared Vue files: ${coverage.sharedVueFileCount}`,
    `Missing Vue files: ${coverage.missingVueFiles.length}`,
    `Unexpected Vue files: ${coverage.unexpectedVueFiles.length}`,
    `Ignored dependency Vue files: ${coverage.ignoredDependencyVueFileCount}`,
    `Ignored support Vue files: ${coverage.ignoredSupportVueFileCount}`,
    `Seeded mutation oracle: ${describeMutationOracle(artifact.mutationOracle)}`,
    `Budget verdict: ${describeVerdict(artifact.budget)}`,
    `Classification: ${describeClassification(artifact)}`,
    `Budget passed: ${artifact.budget.passed}`,
    `Digest: ${artifact.divergence.sha256}`,
    "",
  ].join("\n");
}

/**
 * Printed as Vue-runtime copies rather than as a bare verdict, because that is
 * the number a reviewer of run 31979524200 needed and could not get: two copies
 * of `vue` in the program is why 894 "false negatives" were not Vize's.
 */
function describeAmbient(ambient: MarkdownAmbient | null | undefined): string {
  if (ambient == null) return "missing";
  const copies = ambient.vueRuntime
    .map((entry) => `${entry.name} ×${entry.copies.length}`)
    .join(", ");
  const detail = copies === "" ? "no Vue runtime in program" : copies;
  return ambient.unusableReason == null
    ? `${ambient.verdict} (${detail})`
    : `${ambient.verdict} (${ambient.unusableReason})`;
}

function describeVerdict(budget: BudgetArtifact["budget"]): string {
  return budget.unusableReason == null ? budget.verdict : `unusable (${budget.unusableReason})`;
}

function describeMutationOracle(mutationOracle: MarkdownMutation | null | undefined): string {
  if (mutationOracle?.passed === true) {
    return `${mutationOracle.verdict} (${mutationOracle.file}:${mutationOracle.span.line}:${mutationOracle.span.column})`;
  }
  return `unusable (${mutationOracle?.unusableReason ?? "missing"})`;
}
