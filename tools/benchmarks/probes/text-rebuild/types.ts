import type { SpawnSyncOptionsWithStringEncoding } from "node:child_process";

export const ids = [
  "text.rebuild.inline12",
  "text.rebuild.inline24",
  "text.rebuild.heap43",
  "text.rebuild.heap103",
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
  target: { name: "text_rebuild_probe" };
}

export interface RebuildOracle {
  oracle: "l2-text-rebuild-v1";
  cases: 1040;
  outputs: string[];
  lengths: number[];
  heap: boolean[];
  capacities: number[];
}

export interface BuildMeasurement {
  source_commit: string;
  binary_sha256: string;
  binary: string;
  dependency_identity: DependencyIdentity;
  oracle: RebuildOracle;
  runs: InstructionCounts[];
  dump_receipts: Array<{
    bench_id: BenchId;
    file: string;
    sha256: string;
    bytes: number;
    instructions: number;
  }>;
}

export interface Delta {
  base: number;
  head: number;
  delta: number;
}

export interface Qualification {
  instruction_nonregression: boolean;
  no_functional_difference_observed: true;
  caveat: string;
}

export interface MeasurementReport {
  schema_version: 1;
  supplemental_probe: "l2-text-rebuild-v1";
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
    direct_function: "vize_l2::analysis::text_facts::rebuild_source";
    linked_context: "standalone-public-api-client-built-from-actual-product-source";
  };
  builds: Partial<Record<BuildLabel, BuildMeasurement>>;
  comparison?: Record<BenchId, Delta>;
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
