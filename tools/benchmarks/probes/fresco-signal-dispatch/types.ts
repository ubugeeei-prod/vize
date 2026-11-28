import type { SpawnSyncOptionsWithStringEncoding } from "node:child_process";

export const ids = [
  "fresco.signal.install",
  "fresco.signal.dispatch.simple",
  "fresco.signal.dispatch.siginfo",
  "fresco.signal.dispatch.ignore",
] as const;
export type BenchId = (typeof ids)[number];
export type BuildLabel = "base-a" | "head-a" | "base-b" | "head-b";
export type SourceFile =
  | "src/main.rs"
  | "src/instruction.rs"
  | "Cargo.toml.in"
  | "driver.ts"
  | "types.ts"
  | "support.ts";
export type ProfileName = "release" | "ci-opt";
export type InstructionCounts = Record<BenchId, number>;
export type CommandOptions = Omit<SpawnSyncOptionsWithStringEncoding, "encoding">;

export interface CallgrindRow {
  bench_id: string;
  instructions: number;
}

export interface InstructionIdentity {
  bench_id: string;
  fixture: string;
  window: string;
}

// These are the contracts of the exact repository modules loaded below.
// Their imports deliberately follow DRIVER_REPO rather than a mutable cwd.
export interface InstructionLibrary {
  readonly GUEST_ENVIRONMENT: Readonly<Record<string, string>>;
  readonly BUILD_SOURCE_DATE_EPOCH: string;
  readonly LIBC_DISPATCH: string;
  parseCallgrind(text: string): CallgrindRow | null;
  parseIdentities(stderr: string): ReadonlyMap<string, InstructionIdentity>;
  validateAllocatorSetup(stderr: string): void;
}

export interface MeasurementToolchainLibrary {
  measurementToolchain(root: string): string;
}

export interface CargoLockPackage {
  name: string;
  version: string;
  source?: string;
  checksum?: string;
}

export type CargoLockField = keyof CargoLockPackage;

export interface DependencyIdentity {
  packages: number;
  identities: "all-existing-packages-match-source-lock-v1";
}

export interface ProbeCompilerArtifact {
  reason: "compiler-artifact";
  executable: string;
  fresh: boolean;
  target: { name: "fresco_signal_dispatch_probe" };
}

export interface NmSymbol {
  raw: string;
  address: string;
  bytes: number;
  kind: string;
  name: string;
}

export interface SignalOracle {
  oracle: "fresco-public-signal-dispatch-v1";
  simple_calls: number;
  simple_signal: number;
  simple_entry_errno: number;
  simple_exit_errno: number;
  siginfo_calls: number;
  siginfo_signal: number;
  siginfo_entry_errno: number;
  siginfo_exit_errno: number;
  siginfo_has_information: number;
  ignored_exit_errno: number;
  mask_contains_sigusr1: true;
  terminal_backend_active: false;
}

export interface BuildMeasurement {
  source_commit: string;
  binary_sha256: string;
  binary: string;
  dependency_identity: DependencyIdentity;
  saved_action_static_bytes: number;
  saved_action_symbols: string[];
  default_oracle: "terminated-by-SIGHUP";
  oracle: SignalOracle;
  runs: InstructionCounts[];
}

export interface Delta {
  base: number;
  head: number;
  delta: number;
}

export interface Qualification {
  instruction_nonregression: boolean;
  saved_action_static_storage_nonregression: boolean;
  no_functional_difference_observed: true;
  caveat: string;
}

export interface MeasurementReport {
  schema_version: 1;
  supplemental_probe: "fresco-public-signal-dispatch-v1";
  source_digests: Record<SourceFile, string>;
  driver_commit: string;
  base_commit: string;
  head_commit: string;
  expected_base_commit: string;
  expected_head_commit: string;
  methodology: {
    rustc: string;
    profile: "release";
    target: "x86_64-unknown-linux-gnu";
    flags: "target-cpu=x86-64;instr-atstart=no;cache-sim=no;branch-sim=no";
    window_protocol: "callgrind-client-call-boundary-v1";
    guest_environment: Readonly<Record<string, string>>;
    libc_dispatch: string;
    source_date_epoch: string;
    valgrind: "valgrind-3.22.0";
    libc: string;
  };
  scope: {
    terminal_backend_active: false;
    retry_injection: false;
    nesting_injection: false;
    default_disposition: "native-separate-process-correctness-only";
    linked_context: "standalone-public-api-client-built-from-actual-product-source";
  };
  builds: Partial<Record<BuildLabel, BuildMeasurement>>;
  comparison?: Record<BenchId, Delta>;
  static_saved_action_bytes?: Delta;
  qualification?: Qualification;
  error?: string;
}

export interface EnvironmentReceipt {
  cpuinfo: string;
  uname: string;
  dpkg: string;
  run_id: string | undefined;
  run_attempt: string | undefined;
}
