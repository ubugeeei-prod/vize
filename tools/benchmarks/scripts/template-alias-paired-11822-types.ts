export interface FileEvidence {
  path: string;
  bytes: number;
  sha256: string;
}
export interface InventoryEntry {
  path: string;
  bytes?: number;
  sha256?: string;
  symlink?: string;
  resolved?: string;
}
export interface ClosureSnapshot {
  directories: { root: string; files: InventoryEntry[] }[];
  files: FileEvidence[];
}
export interface PackageManifest {
  [key: string]: unknown;
  name?: string;
  version?: string;
  files?: string[];
  types?: string;
  typings?: string;
  exports?: unknown;
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
  optionalDependencies?: Record<string, string>;
  peerDependencies?: Record<string, string>;
  typescript?: { contentMapper: { exec: string[]; [key: string]: unknown } };
}
export interface Location {
  file: string;
  line: number;
  column: number;
  code: number;
}
export interface Diagnostic extends Location {
  message: string;
}
export interface DiagnosticPacket {
  status: number;
  diagnostics: Diagnostic[];
}
export interface DirectivePacket {
  directives: number[][];
  unusedExpectDirectiveDiagnostics: { code: number; messageText: string }[];
}
export interface TransformPacket {
  text: string;
  extension?: string;
  diagnosticDirectives?: DirectivePacket | null;
  [key: string]: unknown;
}
export interface ProtocolMessage {
  id?: string | number | null;
  method?: string;
  params?: { fileName: string; [key: string]: unknown };
  result?: TransformPacket;
  error?: unknown;
}
export interface Resources {
  wall_s: number;
  user_s: number;
  system_s: number;
  max_rss_kib: number;
  status: number;
}
export interface Metrics {
  mapperWaitSeconds: number;
  checkSeconds: number;
  types: number;
  instantiations: number;
  checkerMemoryKiB: number;
}
export interface Observation {
  command: string;
  args: string[];
  cwd: string;
  explicitEnvironment: NodeJS.ProcessEnv;
  started: string;
  ended: string;
  status: number | null;
  signal: NodeJS.Signals | null;
  timedOut: boolean;
  error?: string;
  stdout: FileEvidence;
  stderr: FileEvidence;
  time: FileEvidence | null;
  usage: Resources;
  text: string;
}
export interface GitIdentity {
  sha: string;
  tree: string;
}
export type VariantId = "baseline" | "candidate" | "vue";
export type Variant = { id: "vue" } | { id: Exclude<VariantId, "vue">; package: string };
export interface MapperConfig {
  compilerOptions?: Record<string, unknown>;
  contentMappers: { package: string; extensions?: string[]; [key: string]: unknown }[];
  [key: string]: unknown;
}
export interface WorkspaceClosure {
  configFile: string;
  packageRoot: string;
  originalConfig: MapperConfig;
  config: MapperConfig;
  changedFields: string[];
  package: InventoryEntry[];
  authoredInputs: InventoryEntry[];
}
export interface CargoArtifact {
  reason: string;
  target: { name: string; kind: string[]; src_path?: string };
  filenames: string[];
  features: string[];
  profile: Record<string, unknown>;
}
export interface SourceReceipt {
  source: GitIdentity;
  endpoint: string;
  token: string;
  rustc: string;
  artifact: CargoArtifact;
  cliPackage: string;
  nativePackage: string;
  image: FileEvidence;
  originalImage: FileEvidence;
  generatedSdk: FileEvidence[];
  capsule: ClosureSnapshot;
  [key: string]: unknown;
}
export interface NativeReceipt {
  packageName: string;
  version: string;
  kind: string;
  integrity: string;
  integrityMeaning: string;
  manifest: FileEvidence;
  binary: FileEvidence;
  elfHeader: string;
}
export interface ProbeIdentity {
  images: { path: string; sha256: string }[];
  sharedObjects: string[];
  token: string | null;
  nativeEntrypoint: string;
}
export interface ProbeRecord {
  stdout: FileEvidence;
  stderr: FileEvidence;
  command: string;
  identity: ProbeIdentity | null;
  nativeDeclarations:
    | import("./template-alias-paired-11822-declarations.ts").DeclarationClosure
    | null;
}
