// The process plan shared by the canonical shard runner and its recorder:
// which observer runs which test binaries, how each process is invoked, and
// the exact segment lines its combined log carries.
import assert from "node:assert/strict";
import { resolve } from "node:path";

export const artifactDirectory = "real-project-davinci-dom-corpus";
export const corpusRelative = "tests/_fixtures/_git";
export const shardEnv = "VIZE_DAVINCI_DIFFERENTIAL_CORPUS_SHARD";
export const shardVectorEnv = "VIZE_DAVINCI_DIFFERENTIAL_CORPUS_SHARD_VECTOR";

export type SuiteKind = "reach" | "ssr" | "pug";
export type BuildTarget = {
  readonly package: string;
  readonly features: string;
  readonly test: string;
};
export type Suite = BuildTarget & {
  readonly kind: SuiteKind;
  /** The canonical sweep test that is sharded; null runs the binary once. */
  readonly sweep: string | null;
  readonly shards: number;
  /** The complete binary may never report fewer passing tests. */
  readonly minimumTests: number;
};

const reach: Suite = {
  kind: "reach",
  package: "vize_atelier_sfc",
  features: "legacy-dom-differential",
  test: "davinci_production_reach",
  sweep: "production_compiles_report_and_hold_their_davinci_reach",
  shards: 16,
  minimumTests: 17,
};
const ssr: Suite = {
  kind: "ssr",
  package: "vize_atelier_ssr",
  features: "legacy-differential",
  test: "davinci_ssr_corpus",
  sweep: "ssr_lanes_agree_on_sfc_templates",
  shards: 12,
  minimumTests: 3,
};
const pug: Suite = {
  kind: "pug",
  package: "vize_l1_to_l2",
  features: "legacy-differential",
  test: "davinci_pug_corpus",
  sweep: null,
  shards: 1,
  minimumTests: 1,
};
const dom: BuildTarget = {
  package: "vize_l1_to_l2",
  features: "legacy-differential",
  test: "davinci_dom_corpus",
};

/** Observers whose evidence is produced by this process runner. */
export const shardedObservers: Readonly<Record<string, readonly Suite[]>> = {
  "ssr-pug": [ssr, pug],
  reach: [reach],
};
/** Every observer's test binaries, for the background prebuild. */
export const prebuildTargets: Readonly<Record<string, readonly BuildTarget[]>> = {
  ...shardedObservers,
  dom: [dom],
};

export type Role = "base" | "shard" | "full";
export type ObserverProcess = {
  readonly suite: Suite;
  readonly role: Role;
  readonly index: number | null;
  readonly vector: string | null;
};

export function observerProcesses(observer: string): ObserverProcess[] {
  const suites = shardedObservers[observer];
  assert(suites, `Observer ${observer} is not sharded`);
  return suites.flatMap((suite): ObserverProcess[] => {
    if (suite.sweep === null) return [{ suite, role: "full", index: null, vector: null }];
    assert(Number.isSafeInteger(suite.shards) && suite.shards > 0, "Invalid shard count");
    const width = String(suite.shards - 1).length;
    return [
      { suite, role: "base", index: null, vector: null },
      ...Array.from({ length: suite.shards }, (_, index) => ({
        suite,
        role: "shard" as const,
        index,
        vector: `${observer}-${suite.test}-shard-${String(index).padStart(width, "0")}-of-${suite.shards}.nul`,
      })),
    ];
  });
}

export function processHeader(entry: ObserverProcess): string {
  const shard =
    entry.role === "shard"
      ? ` index=${entry.index} count=${entry.suite.shards} vector=${entry.vector}`
      : "";
  return `canonical observer process: test=${entry.suite.test} role=${entry.role}${shard}`;
}

export function processArguments(entry: ObserverProcess): string[] {
  const { sweep } = entry.suite;
  if (entry.role === "full") return ["--nocapture"];
  assert(sweep, "A sharded suite needs its sweep test");
  return entry.role === "base"
    ? ["--exact", "--skip", sweep, "--nocapture"]
    : ["--exact", sweep, "--nocapture"];
}

export function cargoArguments(target: BuildTarget, extra: readonly string[]): string[] {
  return [
    "test",
    "-p",
    target.package,
    "--features",
    target.features,
    "--test",
    target.test,
    ...extra,
  ];
}

/**
 * The environment `cargo test` gave the binary (the corpus root, resolved
 * against the workspace, and the package directory), plus this process's
 * shard and vector path. Inherited shard variables never leak into a
 * complete or base process.
 */
export function processEnvironment(
  entry: ObserverProcess,
  inherited: Readonly<Record<string, string | undefined>>,
  manifestDirectory: string,
  cwd: string,
): Record<string, string | undefined> {
  const env: Record<string, string | undefined> = {
    ...inherited,
    CARGO_MANIFEST_DIR: manifestDirectory,
    VIZE_DAVINCI_DIFFERENTIAL_CORPUS: corpusRelative,
  };
  delete env[shardEnv];
  delete env[shardVectorEnv];
  if (entry.role === "shard") {
    env[shardEnv] = `${entry.index}/${entry.suite.shards}`;
    env[shardVectorEnv] = resolve(cwd, artifactDirectory, entry.vector as string);
  }
  return env;
}

export type Finished = {
  readonly status: number | null;
  readonly signal: string | null;
  readonly elapsed: number;
};

export function exitLine(entry: ObserverProcess, finished: Finished): string {
  const index = entry.role === "shard" ? ` index=${entry.index}` : "";
  return `canonical observer process exit: test=${entry.suite.test} role=${entry.role}${index} status=${finished.status} signal=${finished.signal ?? "none"} elapsed_ms=${finished.elapsed}`;
}
