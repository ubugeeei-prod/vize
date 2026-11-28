import { existsSync } from "node:fs";
import path from "node:path";

/**
 * Green CI evidence bound to Content Mapper v1 obligations.
 *
 * A binding is not qualification. It records that every listed receipt passed
 * at one exact candidate source, so an obligation moves from "unbound" to
 * "bound". "complete" still needs the five-case receipts for the release
 * candidate. `tools/commands/release/verify-content-mapper-bindings.ts`
 * re-checks each receipt against the GitHub run that produced it.
 */
export interface ContentMapperV1Receipt {
  /** Workflow file that produced the run. */
  workflow: string;
  runId: number;
  runAttempt: number;
  /** Exact job name in that run attempt; it must have succeeded. */
  job: string;
  /** Steps of that job that must have succeeded. */
  steps: readonly string[];
  /** Artifact uploaded by the run, with its GitHub artifact digest. */
  artifact: string;
  digest: string;
  /** Test cases that must be listed as "passed" in the artifact's test-cases JSON. */
  cases: readonly string[];
}

export interface ContentMapperV1Binding {
  obligations: readonly string[];
  candidateSource: string;
  receipts: readonly ContentMapperV1Receipt[];
}

const PROTOCOL_TEST = "vize_canon batch::virtual_project::content_mapper::tests::protocol::";

export const CONTENT_MAPPER_V1_BINDINGS: readonly ContentMapperV1Binding[] = [
  {
    // Every protocol feature: the Rust feature constants and masks match the
    // pinned upstream protocol shape.
    obligations: [
      "Hover",
      "SignatureHelp",
      "Completion",
      "Definition",
      "TypeDefinition",
      "Implementation",
      "References",
      "DocumentHighlights",
      "Rename",
      "CallHierarchy",
      "CodeActions",
      "Formatting",
      "InlayHints",
      "SemanticTokens",
      "FoldingRanges",
      "SelectionRanges",
      "LinkedEditing",
      "AutoInsert",
      "DocumentSymbols",
      "CodeLens",
    ],
    candidateSource: "9be6799ade47abf2a2e43de2dd540c8b44f8f92d",
    receipts: [
      {
        workflow: ".github/workflows/check.yml",
        runId: 46124973647,
        runAttempt: 1,
        job: "PR source checks / PR Rust workspace products / Rust tests (Type checker)",
        steps: [],
        artifact: "test-cases-rust-typechecker-1",
        digest: "sha256:a1b99ab89796745e8f55229f1d2a2b8462c88820560e4b5d6254feffdeb8b16c",
        cases: [
          `${PROTOCOL_TEST}protocol_v1_feature_constants_match_the_pinned_upstream_shape`,
          `${PROTOCOL_TEST}feature_masks_have_positive_and_negative_bit_oracles`,
          `${PROTOCOL_TEST}emits_protocol_v1_spans_with_honest_features_and_without_forbidden_overlaps`,
          `${PROTOCOL_TEST}transform_declares_its_virtual_extension_on_every_response`,
        ],
      },
    ],
  },
];

const SOURCE_SHA = /^[0-9a-f]{40}$/u;
const ARTIFACT_DIGEST = /^sha256:[0-9a-f]{64}$/u;

export function validateContentMapperV1Bindings(
  root: string,
  bindings: readonly ContentMapperV1Binding[],
  obligationIds: ReadonlySet<string>,
): Map<string, ContentMapperV1Binding> {
  const byObligation = new Map<string, ContentMapperV1Binding>();
  for (const binding of bindings) {
    if (!SOURCE_SHA.test(binding.candidateSource))
      throw new Error(`binding source ${binding.candidateSource} is not an exact commit`);
    if (binding.obligations.length === 0) throw new Error("binding names no obligation");
    if (binding.receipts.length === 0)
      throw new Error(`binding at ${binding.candidateSource} has no receipt`);
    for (const id of binding.obligations) {
      if (!obligationIds.has(id)) throw new Error(`binding names unknown obligation ${id}`);
      if (byObligation.has(id)) throw new Error(`obligation ${id} is bound twice`);
      byObligation.set(id, binding);
    }
    for (const receipt of binding.receipts) {
      const label = `${receipt.workflow}#${receipt.runId}`;
      if (!existsSync(path.join(root, receipt.workflow)))
        throw new Error(`${label}: workflow file does not exist`);
      if (!Number.isSafeInteger(receipt.runId) || receipt.runId <= 0)
        throw new Error(`${label}: run ID must be a positive integer`);
      if (!Number.isSafeInteger(receipt.runAttempt) || receipt.runAttempt <= 0)
        throw new Error(`${label}: run attempt must be a positive integer`);
      if (receipt.job.length === 0) throw new Error(`${label}: receipt names no job`);
      if (receipt.artifact.length === 0) throw new Error(`${label}: receipt names no artifact`);
      if (!ARTIFACT_DIGEST.test(receipt.digest))
        throw new Error(`${label}: artifact digest must be sha256:<64 hex>`);
      if (new Set(receipt.cases).size !== receipt.cases.length)
        throw new Error(`${label}: duplicate test case`);
      if (new Set(receipt.steps).size !== receipt.steps.length)
        throw new Error(`${label}: duplicate step`);
    }
  }
  return byObligation;
}
