export type Issue = 11252 | 11253;
export type Phase = "before" | "after";
export type Role = "runtime-prop-null-11252" | "imports-vue-ts-11253";
export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export interface Image {
  path: string;
  bytes: number;
  sha256: string;
}
export interface SourceFile {
  path: string;
  base64: string;
  sha256: string;
}
export interface Fixture {
  issue: Issue;
  receiptSha256: string;
  files: SourceFile[];
}
export interface MergeDescriptor {
  issue: Issue;
  pull: number;
  originalHead: string;
  merge: string;
  paths: string[];
}
export interface ReleaseRequest {
  issue: Issue;
  phase: Phase;
  version: string;
  releaseRun: string;
  capsuleBlob: string;
  fixes: MergeDescriptor[];
}
export interface Recipe {
  issue: Issue;
  role: Role;
  platform: "darwin" | "linux";
  arch: "arm64" | "x64";
  node: string;
  pnpm: string;
  vue: "3.5.44";
  typescript: string;
  vueTsc: string | null;
  tsrs: string | null;
  originalArgv: readonly (readonly string[])[];
}
export type TimingField = {
  label: "total" | "collect" | "imports" | "gen" | "corsa";
  maxNanoseconds: number;
};
export type StreamOracle =
  | { kind: "bytes"; base64: string }
  | { kind: "json"; value: Json }
  | { kind: "rendered-timings"; literals: string[]; fields: TimingField[] };
export interface Oracle {
  id: string;
  status: number;
  stdout: StreamOracle;
  stderr: { kind: "bytes"; base64: string };
  sourceOwnedBy: { commit: string; path: string; sha256: string };
}
export type CheckerEvidence =
  | { state: "UNKNOWN"; reason: string }
  | {
      state: "LAUNCHER_OBSERVED";
      packageName: string;
      version: string;
      executable: Image;
      versionObservation: string;
      backend: "UNKNOWN";
    };
export interface CommandPlan {
  id: string;
  consumer: "original" | "tsrs-supplement";
  executable: string;
  argv: string[];
  sourceFiles: SourceFile[];
  purpose: "original" | "supplement" | "control";
  adaptation: string | null;
  checker: CheckerEvidence;
  requiresNativeLoad: boolean;
  oracle: Oracle | null;
}
export interface Request {
  schema: "vize.humanReports11252_11253.request.v1";
  role: Role;
  mode: "CAPTURE_BEFORE" | "ACCEPT_AFTER" | "UNQUALIFIED_AFTER";
  reviewedController: {
    head: string;
    files: Array<{ path: string; bytes: number; sha256: string; gitBlob: string }>;
  };
  fixture: Fixture;
  release: ReleaseRequest;
  commands: CommandPlan[];
  sourceAdmission: {
    commit: string;
    run: string;
    receiptSha256: string;
    receiptBlob: string;
  } | null;
  oracleOwnerReceipt: Image | null;
  publicBefore?: {
    controllerRun: string;
    controllerHead: string;
    artifactId: number;
    archiveDigest: string;
    bundleMemberSha256: string;
    oracles: Array<Omit<Oracle, "sourceOwnedBy">>;
    reviewedController: Request["reviewedController"];
    judgmentBlob: string;
  } | null;
}
export interface NativeEvent {
  commandId: string;
  event: "initialized" | "attempt" | "returned" | "failed" | "exit";
  pid: number;
  parentPid: number;
  argv: string[];
  nativePath: string;
  sha256: string;
  ancestry?: Array<{ pid: number; parentPid: number; source: "process.ppid" | "ps" }>;
}
export interface Observation {
  id: string;
  consumer: "original" | "tsrs-supplement";
  pid: number;
  executable: string;
  argv: string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: string | null;
  stdoutBase64: string;
  stderrBase64: string;
  sourceFiles: SourceFile[];
  environment: Record<string, string>;
  nativeEvents: NativeEvent[];
  checker: CommandPlan["checker"];
  instrumentation?: "none" | "load-observed-companion";
  companionOf?: string;
  backend?: "UNKNOWN";
}
export interface PackageBase {
  name: string;
  version: string;
  directory: string;
  resolved: string;
  integrity: string;
  archiveSha512: string;
  files: Image[];
}
export type PackageReceipt = PackageBase &
  (
    | { allInstalledBytesEqualArchive: true; binContext?: never; sourceConformance?: never }
    | {
        allInstalledBytesEqualArchive: false;
        binContext: import("./dependency-bin-context.ts").BinContext;
        sourceConformance: ReturnType<
          typeof import("./dependency-bin-partition.ts").humanDependencyBinPartition
        >;
      }
  );
/** Facts needed to bind a command, without claiming complete file-set qualification. */
export interface InstallationFacts {
  kind: "original" | "tsrs-supplement";
  root: string;
  packageManifest: Json;
  packageRawBase64: string;
  lockRawBase64: string;
  lockSha256: string;
  node: Image;
  nodeVersion: string;
  pnpm: Image;
  pnpmVersion: string;
  platform: string;
  arch: string;
  kernel: string;
  host: import("./optional-residency.ts").HostFacts;
  managerFiles: import("./optional-residency.ts").ManagerFile[];
  declaredOptionalNotInstalled: import("./optional-residency.ts").OptionalAbsence[];
  packageManagerEnvironment: Record<string, string>;
  binPolicy: import("./dependency-bin-policy.ts").BinPolicy;
  packages: PackageBase[];
  cli: Image;
  sdk: Image[];
  mapperExec: string[];
  native: Image & { packageName: string; token: string; identityProbeOnly: true };
  nativeLoad: import("./native-loader-prerequisite.ts").NativeLoadPrerequisite;
}
export interface InstallReceipt extends InstallationFacts {
  schema: "vize.humanReports11252_11253.install.v1";
  packages: PackageReceipt[];
}
export interface PublicPayloadReceipt extends PackageBase {
  publicPayloadBytesEqualArchive: true;
  completeInstalledFileSetQualified: false;
  manifestRawBase64: string;
}
/** Collection can retain raw outcomes while final qualification is still pending.
 * This schema is deliberately not an InstallReceipt and cannot enter receive(). */
export interface CaptureInstallReceipt extends InstallationFacts {
  schema: "vize.humanReports11252_11253.capturePrerequisites.v1";
  packages: PublicPayloadReceipt[];
  mandatoryExecutableChainsAuthenticated: true;
  executableChains: import("./command-chains.ts").ExecutableChains;
  binConfiguration: import("./bin-configuration.ts").BinConfiguration;
  completeInstalledFileSetQualified: false;
  accepted: false;
  productQualified: false;
  afterQualified: false;
  issueQualified: false;
}
export interface ReleaseAuthority {
  version: string;
  candidate: string;
  source: string;
  workflowHead: string;
  run: string;
  capsule: Record<string, unknown>;
  sdk: Array<{ path: string; bytes: number; sha256: string }>;
  observations: Array<{ endpoint: string; value: unknown }>;
  produced: unknown;
}
export interface Bundle {
  request: Request;
  release: ReleaseAuthority;
  installs: InstallReceipt[];
  observations: Observation[];
  companions?: Observation[];
  controller: { head: string; run: string; job: string; files: Image[] };
}
