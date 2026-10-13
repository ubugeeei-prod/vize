// Validation of process-sharded canonical observer logs. The recorder checks
// every process segment, sums the counters, and proves the shard vectors are
// an exact disjoint cover of the canonical files.json manifest, so the
// evidence has the same meaning as one complete single-process sweep.
import assert from "node:assert/strict";
import {
  observerProcesses,
  processHeader,
  shardedObservers,
} from "./canonical-corpus-shard-plan.ts";
import type { SuiteKind } from "./canonical-corpus-shard-plan.ts";

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
