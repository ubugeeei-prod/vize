// Process-level shards for the canonical production-reach and SSR observers.
//
// Both sweeps clear and read the process-wide profiler around every compile,
// so they cannot use threads. Each observer instead builds its test binary
// once and runs several copies concurrently: one "base" process runs every
// test except the canonical sweep, and each "shard" process runs only the
// sweep over the files whose position in the collected corpus vector is
// congruent to its index. Each shard writes the logical paths it swept to a
// NUL-terminated vector. The recorder below checks every process segment,
// sums the counters, and proves the vectors are an exact disjoint cover of the
// canonical files.json manifest, so the evidence has the same meaning as one
// complete single-process sweep.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import {
  closeSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

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

const headerPattern =
  /^canonical observer process: test=([a-z0-9_]+) role=(base|shard|full)(?: index=(\d+) count=(\d+) vector=([A-Za-z0-9._-]+))?$/;
const exitPattern =
  /^canonical observer process exit: test=([a-z0-9_]+) role=(base|shard|full)(?: index=(\d+))? status=(-?\d+|null) signal=([A-Z0-9]+|none) elapsed_ms=(\d+)$/;

export function hasProcessSegments(lines: readonly string[]): boolean {
  return lines.some((line) => line.startsWith("canonical observer process"));
}

type Segment = { readonly header: string; readonly lines: readonly string[] };

export function splitProcessSegments(lines: readonly string[]): Segment[] {
  const segments: Segment[] = [];
  let current: { header: string; lines: string[] } | null = null;
  for (const line of lines) {
    if (current === null) {
      if (headerPattern.test(line)) {
        current = { header: line, lines: [] };
        continue;
      }
      // Build output and blank separators only; never counters or results.
      assert(
        !/^(?:canonical observer process|test result:|davinci[ -]|test .* \.\.\. )/.test(line),
        "Observer evidence outside a process segment",
      );
      continue;
    }
    if (line.startsWith("canonical observer process")) {
      const exit = exitPattern.exec(line);
      const header = headerPattern.exec(current.header);
      assert(exit && header, "Malformed observer process boundary");
      assert(
        exit[1] === header[1] && exit[2] === header[2] && (exit[3] ?? null) === (header[3] ?? null),
        "Observer process exit belongs to another process",
      );
      assert.equal(exit[4], "0", `Observer process failed: ${current.header}`);
      assert.equal(exit[5], "none", `Observer process was signalled: ${current.header}`);
      segments.push(current);
      current = null;
      continue;
    }
    current.lines.push(line);
  }
  assert.equal(current, null, "Unterminated observer process segment");
  return segments;
}

function single(lines: readonly string[], prefix: string): string {
  const found = lines.filter((line) => line.startsWith(prefix));
  assert.equal(found.length, 1, `Missing or duplicate observer counter: ${prefix}`);
  return found[0] as string;
}

function counter(line: string, name: string): number {
  const match = new RegExp(`(?:^| )${name}=(\\d+)(?: |$)`).exec(line);
  assert(match, `Missing observer counter ${name}`);
  return Number(match[1]);
}

const closurePrefix = "davinci-differential corpus scope:";
const shardPrefix = "davinci-differential corpus shard:";
const resultPattern =
  /^test result: ok\. (\d+) passed; 0 failed; 0 ignored; 0 measured; (\d+) filtered out;/;

function harness(lines: readonly string[]): { passed: number; filtered: number } {
  const results = lines.filter((line) => line.startsWith("test result:"));
  assert.equal(results.length, 1, "Incomplete observer harness");
  const match = resultPattern.exec(results[0] as string);
  assert(match, "Observer tests did not complete without skips");
  return { passed: Number(match[1]), filtered: Number(match[2]) };
}

function closure(lines: readonly string[], gitlinks: number): void {
  const found = lines.filter((line) => line.startsWith(closurePrefix));
  assert.equal(found.length, 1, "Incomplete canonical scope");
  const line = found[0] as string;
  assert(line.includes("scope=canonical closure_evidence=true"), "Observer used a foreign corpus");
  assert.equal(counter(line, "submodules"), gitlinks, "Observer omitted committed gitlinks");
}

export function parseShardVector(bytes: Buffer): string[] {
  if (bytes.length === 0) return [];
  assert.equal(bytes.at(-1), 0, "Incomplete shard path vector");
  const paths = new TextDecoder("utf-8", { fatal: true }).decode(bytes).slice(0, -1).split("\0");
  for (const path of paths)
    assert(
      path &&
        !path.startsWith("/") &&
        !path.split("/").some((part) => ["", ".", ".."].includes(part)),
      "Foreign shard corpus path",
    );
  return paths;
}

type Totals = Record<string, number>;
const add = (totals: Totals, key: string, value: number) => {
  totals[key] = (totals[key] ?? 0) + value;
};
const shapes = ["dom_inline", "dom_module", "ssr", "vapor"];

/** One shard's canonical counters; returns the number of files it swept. */
function validateShardCounters(kind: SuiteKind, lines: readonly string[], totals: Totals): number {
  if (kind === "reach") {
    const summary = single(lines, "davinci production reach: scope=canonical ");
    const files = counter(summary, "files");
    assert.equal(counter(summary, "unreadable"), 0, "Unreadable reach inputs");
    assert.equal(counter(summary, "parse_errors"), 0, "Reach parse failures");
    const canonical = lines.slice(lines.indexOf(summary));
    for (const shape of shapes) {
      const reachLine = single(canonical, `davinci production reach: shape=${shape} `);
      assert.equal(counter(reachLine, "rejected"), 0, "Production reach rejected a plan");
      assert.equal(counter(reachLine, "divergences"), 0, "Production reach diverged");
      const native = single(
        canonical,
        `davinci native-only acceptance: scope=canonical product=compiler target=${shape} `,
      );
      assert.equal(counter(native, "planned"), files, "Native reach inventory changed");
      assert.equal(
        counter(native, "native_only") +
          counter(native, "legacy_backed") +
          counter(native, "unverified"),
        files,
        "Native reach counters are incomplete",
      );
      add(totals, `native.${shape}`, counter(native, "planned"));
    }
    add(totals, "files", files);
    return files;
  }
  assert.equal(kind, "ssr", "Only reach and SSR sweeps are sharded");
  const emitter = single(lines, "davinci SSR corpus sweep: scope=canonical ");
  const files = counter(emitter, "files");
  assert.equal(counter(emitter, "parsed"), files, "SSR parse inventory changed");
  const production = single(lines, "davinci SSR production sweep: scope=canonical ");
  for (const [label, line] of [
    ["emitter", emitter],
    ["production", production],
  ] as const) {
    assert.equal(counter(line, "rejected"), 0, "SSR rejected a plan");
    assert.equal(counter(line, "divergences"), 0, "SSR diverged");
    for (const name of ["templates", "compared", "s4"])
      add(totals, `${label}.${name}`, counter(line, name));
  }
  add(totals, "files", files);
  add(totals, "parsed", counter(emitter, "parsed"));
  return files;
}

export type ShardEvidence = {
  readonly files: readonly (readonly unknown[])[];
  readonly readVector: (name: string) => Buffer;
};
export type ShardIdentity = { readonly files: number; readonly gitlinks: readonly unknown[] };

/**
 * Validate a sharded observer log against the canonical manifest. Returns the
 * vector file names whose bytes the receipt must pin.
 */
export function validateShardedObserver(
  observer: string,
  lines: readonly string[],
  identity: ShardIdentity,
  evidence: ShardEvidence,
): string[] {
  const expected = observerProcesses(observer);
  const segments = splitProcessSegments(lines);
  assert.deepEqual(
    segments.map((segment) => segment.header),
    expected.map(processHeader),
    "Observer processes are incomplete or foreign",
  );
  const gitlinks = identity.gitlinks.length;
  const manifest = evidence.files.map((row) => row[0]);
  assert(
    manifest.length === identity.files && manifest.every((path) => typeof path === "string"),
    "Missing canonical file manifest",
  );
  const vectors: string[] = [];
  for (const suite of shardedObservers[observer] ?? []) {
    const own = expected
      .map((entry, position) => ({ entry, segment: segments[position] as Segment }))
      .filter(({ entry }) => entry.suite === suite);
    if (suite.sweep === null) {
      assert.equal(own.length, 1, "Missing observer process");
      const { segment } = own[0] as { segment: Segment };
      const result = harness(segment.lines);
      assert.equal(result.filtered, 0, "Observer tests were filtered");
      assert(result.passed >= suite.minimumTests, "Observer tests shrank");
      closure(segment.lines, gitlinks);
      assert.equal(suite.kind, "pug", "Unknown unsharded observer suite");
      const pugLine = single(segment.lines, "davinci pug corpus sweep: ");
      assert.equal(counter(pugLine, "files"), identity.files, "Pug corpus shrank");
      assert(
        counter(pugLine, "pug") > 0 && counter(pugLine, "compiled_lanes") > 0,
        "Pug proved no compilation",
      );
      assert.equal(counter(pugLine, "refused"), 0, "Pug refused original inputs");
      continue;
    }
    const [base, ...shards] = own;
    assert(base && base.entry.role === "base", "Missing observer base process");
    const baseResult = harness(base.segment.lines);
    assert.equal(baseResult.filtered, 1, "Observer base process skipped other tests");
    assert(baseResult.passed + 1 >= suite.minimumTests, "Observer tests shrank");
    assert(
      !base.segment.lines.some(
        (line) =>
          line.startsWith(closurePrefix) ||
          line.startsWith(shardPrefix) ||
          line.startsWith(`test ${suite.sweep} `),
      ),
      "Observer base process ran the canonical sweep",
    );
    const totals: Totals = {};
    const seen = new Set<string>();
    for (const { entry, segment } of shards) {
      const result = harness(segment.lines);
      assert.equal(result.passed, 1, "Observer shard ran other tests");
      assert.equal(result.filtered, baseResult.passed, "Observer shard binary differs from base");
      assert.equal(
        segment.lines.filter((line) => line === `test ${suite.sweep} ... ok`).length,
        1,
        "Observer shard did not pass its canonical sweep",
      );
      closure(segment.lines, gitlinks);
      const shardLine = single(segment.lines, shardPrefix);
      assert(
        shardLine.startsWith(
          `${shardPrefix} scope=canonical index=${entry.index} count=${suite.shards} `,
        ),
        "Observer shard selected a foreign slice",
      );
      assert.equal(counter(shardLine, "planned"), identity.files, "Observer shard corpus shrank");
      const vector = entry.vector as string;
      const paths = parseShardVector(evidence.readVector(vector));
      assert.equal(counter(shardLine, "selected"), paths.length, "Shard vector is incomplete");
      assert.equal(
        validateShardCounters(suite.kind, segment.lines, totals),
        paths.length,
        "Shard counters differ from its vector",
      );
      for (const path of paths) {
        assert(!seen.has(path), `Shards overlap at ${path}`);
        seen.add(path);
      }
      vectors.push(vector);
    }
    assert.equal(seen.size, manifest.length, "Shards do not cover the canonical manifest");
    for (const path of manifest)
      assert(seen.has(path as string), `Shards omitted canonical file ${String(path)}`);
    assert.equal(totals.files, identity.files, "Sharded corpus shrank");
    if (suite.kind === "reach") {
      for (const shape of shapes)
        assert.equal(totals[`native.${shape}`], identity.files, "Native reach inventory changed");
    } else {
      assert.equal(totals.parsed, identity.files, "SSR parse inventory changed");
      for (const label of ["emitter", "production"])
        assert(
          ["templates", "compared", "s4"].every((name) => (totals[`${label}.${name}`] ?? 0) > 0),
          "SSR observer proved no selected production comparisons",
        );
    }
  }
  return vectors;
}

type Built = { readonly executable: string; readonly manifestDirectory: string };

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

/** Build one suite's test binary and return its executable. */
export function buildSuite(suite: Suite, cwd: string, output: (bytes: Buffer) => void): Built {
  const result = spawnSync(
    "cargo",
    cargoArguments(suite, ["--no-run", "--message-format=json-render-diagnostics"]),
    { cwd, maxBuffer: 1024 * 1024 * 1024, stdio: ["ignore", "pipe", "pipe"] },
  );
  output(result.stderr ?? Buffer.alloc(0));
  assert(!result.error && result.status === 0, `cargo test --no-run failed for ${suite.test}`);
  const executables = (result.stdout ?? Buffer.alloc(0))
    .toString("utf8")
    .split("\n")
    .filter((line) => line.startsWith("{"))
    .map((line) => JSON.parse(line) as Record<string, unknown>)
    .filter((message) => {
      const target = message.target as { name?: unknown; kind?: unknown } | undefined;
      const profile = message.profile as { test?: unknown } | undefined;
      return (
        message.reason === "compiler-artifact" &&
        target?.name === suite.test &&
        Array.isArray(target.kind) &&
        target.kind.includes("test") &&
        profile?.test === true &&
        typeof message.executable === "string"
      );
    });
  assert.equal(executables.length, 1, `Missing unique ${suite.test} test executable`);
  const message = executables[0] as { executable: string; manifest_path: string };
  return { executable: message.executable, manifestDirectory: dirname(message.manifest_path) };
}

type Finished = { readonly status: number | null; readonly signal: string | null; elapsed: number };

function runProcess(
  entry: ObserverProcess,
  built: Built,
  cwd: string,
  outputPath: string,
): Promise<Finished> {
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    CARGO_MANIFEST_DIR: built.manifestDirectory,
    VIZE_DAVINCI_DIFFERENTIAL_CORPUS: corpusRelative,
  };
  delete env[shardEnv];
  delete env[shardVectorEnv];
  if (entry.role === "shard") {
    env[shardEnv] = `${entry.index}/${entry.suite.shards}`;
    env[shardVectorEnv] = resolve(cwd, artifactDirectory, entry.vector as string);
  }
  const descriptor = openSync(outputPath, "w");
  const started = Date.now();
  return new Promise((resolvePromise) => {
    const child = spawn(built.executable, processArguments(entry), {
      cwd: built.manifestDirectory,
      env,
      stdio: ["ignore", descriptor, descriptor],
    });
    let settled = false;
    const finish = (status: number | null, signal: string | null) => {
      if (settled) return;
      settled = true;
      closeSync(descriptor);
      resolvePromise({ status, signal, elapsed: Date.now() - started });
    };
    child.on("error", () => finish(null, "SPAWN"));
    child.on("close", (status, signal) => finish(status, signal));
  });
}

export function exitLine(entry: ObserverProcess, finished: Finished): string {
  const index = entry.role === "shard" ? ` index=${entry.index}` : "";
  return `canonical observer process exit: test=${entry.suite.test} role=${entry.role}${index} status=${finished.status} signal=${finished.signal ?? "none"} elapsed_ms=${finished.elapsed}`;
}

/** Build once, run every observer process concurrently, and write one log. */
export async function runObserver(
  observer: string,
  cwd: string,
  logName: string,
): Promise<boolean> {
  const directory = resolve(cwd, artifactDirectory);
  mkdirSync(directory, { recursive: true });
  const chunks: Buffer[] = [];
  const write = (bytes: Buffer) => {
    chunks.push(bytes);
    process.stdout.write(bytes);
  };
  const processes = observerProcesses(observer);
  const built = new Map<Suite, Built>();
  try {
    for (const suite of shardedObservers[observer] ?? [])
      built.set(suite, buildSuite(suite, cwd, write));
  } catch (error) {
    writeFileSync(join(directory, logName), Buffer.concat(chunks));
    throw error;
  }
  const scratch = mkdtempSync(join(tmpdir(), `canonical-${observer}-`));
  try {
    const finished = await Promise.all(
      processes.map(async (entry, position) => {
        const outputPath = join(scratch, `${position}.log`);
        const result = await runProcess(entry, built.get(entry.suite) as Built, cwd, outputPath);
        process.stdout.write(`${exitLine(entry, result)}\n`);
        return { entry, result, outputPath };
      }),
    );
    for (const { entry, result, outputPath } of finished) {
      const output = readFileSync(outputPath);
      const segment = Buffer.concat([
        Buffer.from(`${processHeader(entry)}\n`),
        output,
        Buffer.from(output.length && output.at(-1) !== 10 ? "\n" : ""),
        Buffer.from(`${exitLine(entry, result)}\n`),
      ]);
      chunks.push(segment);
      process.stdout.write(segment);
    }
    writeFileSync(join(directory, logName), Buffer.concat(chunks));
    return finished.every(({ result }) => result.status === 0 && result.signal === null);
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

/** Compile every test binary an observer will run, without running it. */
export function prebuild(observer: string, cwd: string): void {
  const targets = prebuildTargets[observer];
  assert(targets, `Unknown canonical observer ${observer}`);
  for (const target of targets) {
    const result = spawnSync("cargo", cargoArguments(target, ["--no-run"]), {
      cwd,
      stdio: "inherit",
    });
    assert(!result.error && result.status === 0, `cargo test --no-run failed for ${target.test}`);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, observer, logName] = process.argv.slice(2);
  const cwd = process.cwd();
  if (mode === "prebuild" && observer) prebuild(observer, cwd);
  else if (mode === "run" && observer && logName) {
    if (!(await runObserver(observer, cwd, logName))) process.exitCode = 1;
  } else throw new Error("Expected canonical shards prebuild OBSERVER | run OBSERVER LOG");
}
