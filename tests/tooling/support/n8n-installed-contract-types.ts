/** Preserve whole JSON packets, including unknown provider fields. These types do not project output. */
export interface CliMessage {
  ruleId: string;
  severity: number;
  [field: string]: unknown;
}
export interface CliPacket {
  file: string;
  messages: CliMessage[];
  errorCount: number;
  warningCount: number;
  [field: string]: unknown;
}
export interface SourceCase {
  id: string;
  source: string;
  sourceSha256: string;
  scriptless: boolean;
  ruleOptions: Record<string, unknown>;
  cliExpectations: CliPacket[];
}
export interface SemanticFixture {
  schema: string;
  cases: SourceCase[];
  qualification: string;
}
export interface OracleCapture {
  caseId: string;
  configuration: string;
  packets: CliPacket[];
}
export interface SemanticOracle {
  caseId: string;
  observations: Array<{ recorded: { captures: OracleCapture[] } }>;
}
export interface Corpus {
  revision: string;
  fixture: string;
  packages: Record<string, number>;
  files: Array<{ file: string; sha256: string; scriptless: boolean }>;
  licenses: Array<{ file: string; sha256: string }>;
}
export interface AuthoredFixture {
  cases: Array<{ id: string; source: string }>;
  cliExpectations: OracleCapture[];
}
export interface AuthoredOracle {
  recorded: {
    captures: OracleCapture[];
    configurations: Array<{ id: string; ruleOptions: Record<string, unknown> }>;
  };
}
export interface Projection {
  fixtureRevision: string;
  packageVueFileCounts: Record<string, number>;
  linter: { rules: Record<string, string>; ruleOptions: Record<string, unknown> };
  scopes: Record<
    string,
    { entries: Array<{ files: string[]; linter: { rules: Record<string, string> } }> }
  >;
  oracleRules: Record<string, string>;
}
export interface SemanticCliCapture {
  caseId: string;
  baseline: CliPacket[];
  repeat: CliPacket[];
  outside: CliPacket[];
}
export type AuthoredCapture = (options: {
  onRecorded: (packet: {
    phase?: string;
    configuration: string;
    caseId: string;
    [field: string]: unknown;
  }) => void;
}) => Promise<AuthoredOracle>;
export type SemanticCapture = (options: {
  onRecorded: (packet: { caseId: string; iteration: number; packet: unknown }) => void;
}) => Promise<SemanticOracle[]>;
