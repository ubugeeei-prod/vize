import type { buildProductObserver } from "./observer-build.ts";

export type CompilerTarget = "dom" | "ssr" | "vapor";
export type JsonValue = null | boolean | number | string | JsonValue[] | JsonObject;
export type JsonObject = { [key: string]: JsonValue };
export type CompilerBuildReceipt = ReturnType<typeof buildProductObserver>["receipt"];

type Artifact = { path: string; sha256: string; role?: string };
export type CompilerFixture = {
  id: string;
  state: string;
  targets: CompilerTarget[];
  comparison: { requiredFacets: string[]; contract: string };
  adapters: { legacy: string; native: null; reasons: { native: string } };
  inputs: { files: Artifact[] };
  reference: Artifact;
  historicalObservation: Artifact;
  options: Artifact;
};

export type PinnedSfcOptions = {
  descriptorParseOptions: JsonValue;
  compileOptions: JsonValue;
  effectiveAdapterInputs: {
    templateSyntax: JsonValue;
    customElements: { construction: JsonValue };
    codegen: JsonValue;
    scriptOutput: JsonValue;
    experimental: JsonValue;
  };
  authoredWitness: { inputSha256: string };
  entrypoint: string;
};

export type HistoricalCompilerPacket = {
  profile?: string;
  result?: JsonObject;
  options?: JsonObject;
  cases?: { id: string; options: JsonObject; output: JsonObject }[];
};
export type CompilerReference = { options: JsonObject; output: JsonObject };
export type CompilerObservation = {
  id: string;
  source: string;
  entrypoint: string;
  options: JsonValue;
  result: JsonObject;
};
export type CompilerObservationPacket = {
  schema: string;
  version: number;
  cases: CompilerObservation[];
};

export type CompilerAttempt = {
  argv: string[];
  exitStatus: number | null;
  signal: string | null;
  processError: string | null;
  stdoutBase64: string;
  stderrBase64: string;
  stdoutSha256: string;
};

export type CompilerRow = {
  id: string;
  target: CompilerTarget;
  legacy: {
    state: "failed" | "completed";
    verdict: "failed" | "matched-reference" | "baseline-drift";
    inputSha256: string;
    result: JsonObject | null;
    error?: string;
  };
  native: { state: "unsupported"; reason: string };
  comparison: { state: "not-compared"; reason: string };
};

export type CompilerSummary = {
  plannedCases: number;
  legacyMatches: number;
  baselineDrift: number;
  legacyFailures: number;
  nativeUnsupported: number;
  nativeHandled: number;
  nativeEquivalent: number;
  pairedComparisons: number;
};

export type CompilerApiReport = {
  schema: string;
  version: number;
  product: string;
  sourceRevision: string;
  manifestSha256: string;
  buildReceipt: CompilerBuildReceipt | null;
  attempts: CompilerAttempt[];
  failure: string | null;
  rows: CompilerRow[];
  summary: CompilerSummary;
};

export function compilerFailure(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
