import type { SfcDialect } from "./sfc-baseline-routes.ts";

export type BaselineFailureStage =
  | "adapter-load"
  | "comparison-harness"
  | "sfc-parse"
  | "template-compile"
  | "semantic-normalize";

export type BaselineFailure = {
  side: "original" | "formatted" | "harness";
  stage: BaselineFailureStage;
  message: string;
};

export type SfcBaselineProvenance = {
  id: string;
  dialect: SfcDialect;
  package: string | null;
  version: string | null;
  entrySha256: string | null;
  normalization: string;
  options: Record<string, unknown>;
};

export type SfcBaselineComparison = {
  verdict: "equivalent" | "semantic-diff" | "baseline-unusable";
  reasonCode: string | null;
  differences: string[];
  failure: BaselineFailure | null;
  beforeSemanticSha256: string | null;
  afterSemanticSha256: string | null;
  baseline: SfcBaselineProvenance;
};
