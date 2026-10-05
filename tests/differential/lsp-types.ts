export type JsonRpcMessage = {
  jsonrpc: string;
  id?: number;
  method?: string;
  params?: Record<string, unknown>;
  result?: unknown;
  error?: unknown;
};

export type FileReference = { source: string; sha256: string; runtimePath: string };
export type FixtureData = {
  version: number;
  id: string;
  provenance: { fixCommit: string; witness?: { path: string; sha256: string } };
  documentVersion: number;
  initializationOptions?: Record<string, boolean>;
  hierarchicalDocumentSymbols?: true;
  files?: FileReference[];
  input?: FileReference;
  entry?: string;
  expected: { source: string; sha256: string };
  method: string;
  options?: { tabSize: number; insertSpaces: boolean };
};
export type PlannedLspFixture = {
  id: string;
  state: string;
  targets: string[];
  case: { path: string; sha256: string };
  inputs: { root: string; files: { path: string; sha256: string }[] };
  requestCount: number;
  initializationOptions: Record<string, boolean>;
};
export type LspFixture = PlannedLspFixture & {
  data: FixtureData;
  files: { runtimePath: string; bytes: Buffer }[];
  entry: string;
  requests: { method?: string; params: Record<string, unknown>; result: unknown }[];
};
export type LoadedLspManifest = {
  manifest: {
    schema: string;
    version: number;
    product: string;
    baseRevision: string;
    cases: PlannedLspFixture[];
  };
  manifestSha256: string;
  cases: LspFixture[];
};
export type WireObservation = {
  clientWireBase64: string;
  serverWireBase64: string;
  clientWireSha256?: string;
  serverWireSha256?: string;
  workspaceUri?: string;
  stderrBase64: string;
  exitStatus: number | null;
  signal: string | null;
  processError: string | null;
};
export type ResponseComparison = { requestId: number; state: string };
export type LspRow = {
  id: string;
  legacy: {
    state: string;
    verdict: string;
    comparator: string;
    observation?: WireObservation;
    responses?: ResponseComparison[];
    error?: string;
  };
  native: { state: string; reason: string };
  comparison: { state: string; reason: string };
};
export type BuildIdentity = {
  sourceRevision: string;
  binaryPath?: string;
  binarySha256?: string;
  cliVersion?: string;
};
export type BuildReceipt = BuildIdentity & {
  schema: string;
  version: number;
  recipe: string;
};
export type LspSummary = {
  plannedCases: number;
  legacyMatches: number;
  legacyFailures: number;
  baselineDrift: number;
  nativeUnsupported: number;
  pairedComparisons: number;
  nativeHandled: number;
  nativeEquivalent: number;
};
export type LspReport = {
  schema: string;
  version: number;
  product: string;
  sourceRevision: string;
  manifestSha256: string;
  argv: string[];
  buildReceipt: BuildReceipt | null;
  binary: { path: string; sha256: string; version: string } | null;
  binaryProbe?: {
    exitStatus: number | null;
    signal: string | null;
    stdoutBase64: string;
    stderrBase64: string;
    processError: string | null;
  };
  rows: LspRow[];
  summary: LspSummary;
};

export function hasCapabilities(message: JsonRpcMessage): boolean {
  if (message.error || typeof message.result !== "object" || message.result === null) return false;
  const result = message.result as Record<string, unknown>;
  return (
    typeof result.capabilities === "object" &&
    result.capabilities !== null &&
    !Array.isArray(result.capabilities)
  );
}

export const errorText = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);
