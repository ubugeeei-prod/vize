// Per-observer pieces of the real-project divergence automation: how a
// canonical observer log reports legacy-vs-native differences, how a sample
// is shown in the pickup issue, which reducer oracle keeps the difference,
// and where a minimized witness is promoted. Reduction, promotion and issue
// filing are otherwise shared, so every observer gets the same custody rules.
import assert from "node:assert/strict";
import { dirname, join } from "node:path";

export type DomSample = {
  sourcePath: string;
  oldLength: number;
  newLength: number;
  firstDifference: number;
  oldWindow: string;
  newWindow: string;
};
export type SsrSample = { sourcePath: string; lanes: string[]; selectedFailed: boolean };
export type Divergences<Sample> = { divergenceCount: number; samples: Sample[] };
export type DivergenceObserver = {
  readonly id: "dom" | "ssr";
  readonly label: string;
  /** Observer log inside the evidence artifact. */
  readonly log: string;
  /** Evidence artifact prefix, before `-<run>-<attempt>-<sha>`. */
  readonly evidenceArtifact: string;
  readonly parse: (log: string) => Divergences<{ sourcePath: string }>;
  readonly describe: (sample: never) => string;
  readonly reducerArguments: (sourcePath: string, vizeBin: string) => string[];
  readonly fixtureRoot: string;
  /** The merge-queue test that runs every promoted witness. */
  readonly promotedTest: string;
};

const ansiSequence = new RegExp(`${String.fromCharCode(27)}\\[[0-?]*[ -/]*[@-~]`, "g");
const corpusPrefix = "tests/_fixtures/_git/";
const sampleLimit = 20;

function stripAnsi(text: string): string {
  return text.replace(ansiSequence, "");
}

export function compareCodepoints(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function code(value: unknown): string {
  return `\`${String(value).replaceAll("`", "\\`")}\``;
}

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function count(text: string, label: string): number {
  const value = Number(text);
  assert(Number.isSafeInteger(value), `Invalid ${label} divergence count`);
  return value;
}

export function parseDomDivergences(log: string): Divergences<DomSample> {
  const counters = [
    ...stripAnsi(log).matchAll(/davinci DOM corpus sweep: [^\r\n]* divergences=(\d+)/g),
  ];
  assert.equal(counters.length, 1, "Expected one canonical DOM corpus counter");
  const divergenceCount = count(counters[0][1], "canonical DOM");
  const samples: DomSample[] = [];
  for (const rawLine of stripAnsi(log).split(/\r?\n/)) {
    const start = rawLine.indexOf(corpusPrefix);
    if (start === -1) continue;
    const line = rawLine.slice(start);
    const match =
      /^(tests\/_fixtures\/_git\/[^/]+\/.+?\.vue): old_len=(\d+) new_len=(\d+) first_diff=(\d+) old_window=(.*) new_window=(.*)$/.exec(
        line,
      );
    if (!match) continue;
    samples.push({
      sourcePath: match[1],
      oldLength: Number(match[2]),
      newLength: Number(match[3]),
      firstDifference: Number(match[4]),
      oldWindow: match[5],
      newWindow: match[6],
    });
  }
  assert.equal(
    samples.length,
    Math.min(divergenceCount, sampleLimit),
    "Canonical DOM divergence samples are incomplete",
  );
  return { divergenceCount, samples };
}

const ssrShardHeader =
  /^canonical observer process: test=davinci_ssr_corpus role=shard index=(\d+) count=(\d+) /;
const ssrShardExit = /^canonical observer process exit: test=davinci_ssr_corpus role=shard /;
const ssrCounter = /^davinci SSR production sweep: scope=\S+ [^\r\n]* divergences=(\d+)$/;
// `{name} {verdicts:?}` from the production sweep: the corpus path under any
// root spelling, then the lane labels of that SFC's production compiles.
const ssrSample =
  /^[^\s"'`]*?(tests\/_fixtures\/_git\/[^/\s]+\/.+?\.vue) \[((?:"[a-z0-9_.]+"(?:, "[a-z0-9_.]+")*)?)\](: selected lane failed: .*)?$/;

type SsrShard = { index: number; total: number; counters: number[]; samples: SsrSample[] };

// The SSR observer runs the production sweep in process shards. Each shard
// prints one production counter and, when it differs, every divergent path;
// a shard without its counter is incomplete evidence, never a clean shard.
export function parseSsrDivergences(log: string): Divergences<SsrSample> {
  const shards = new Map<number, SsrShard>();
  let shard: SsrShard | null = null;
  for (const line of stripAnsi(log).split(/\r?\n/)) {
    const header = ssrShardHeader.exec(line);
    if (header) {
      const index = Number(header[1]);
      assert(!shards.has(index), `Duplicate canonical SSR shard ${index}`);
      shard = { index, total: Number(header[2]), counters: [], samples: [] };
      shards.set(index, shard);
      continue;
    }
    if (shard === null) continue;
    if (ssrShardExit.test(line)) {
      shard = null;
      continue;
    }
    const counter = ssrCounter.exec(line);
    if (counter) {
      shard.counters.push(count(counter[1], "canonical SSR"));
      continue;
    }
    const sample = ssrSample.exec(line);
    if (sample) {
      shard.samples.push({
        sourcePath: sample[1],
        lanes: sample[2] === "" ? [] : sample[2].split(", ").map((lane) => lane.slice(1, -1)),
        selectedFailed: sample[3] !== undefined,
      });
    }
  }
  assert(shards.size > 0, "Expected canonical SSR shard evidence");
  const totals = new Set([...shards.values()].map(({ total }) => total));
  assert.equal(totals.size, 1, "Canonical SSR shards disagree on their count");
  assert.equal(shards.size, [...totals][0], "Canonical SSR shard evidence is incomplete");
  let divergenceCount = 0;
  const samples: SsrSample[] = [];
  for (const { index, counters, samples: found } of shards.values()) {
    assert.equal(
      counters.length,
      1,
      `Expected one canonical SSR production counter in shard ${index}`,
    );
    assert.equal(
      found.length,
      counters[0],
      `Canonical SSR divergence samples are incomplete in shard ${index}`,
    );
    divergenceCount += counters[0];
    samples.push(...found);
  }
  samples.sort((left, right) => compareCodepoints(left.sourcePath, right.sourcePath));
  return { divergenceCount, samples: samples.slice(0, sampleLimit) };
}

const dom: DivergenceObserver = {
  id: "dom",
  label: "DOM",
  log: "dom-corpus.log",
  evidenceArtifact: "canonical-corpus-dom",
  parse: parseDomDivergences,
  describe: (sample: DomSample) =>
    `- ${code(sample.sourcePath)}: old=${sample.oldLength} bytes, Davinci=${sample.newLength} bytes, first difference=${sample.firstDifference}; old window=${code(sample.oldWindow)}; Davinci window=${code(sample.newWindow)}`,
  reducerArguments: () => ["--dom-divergence"],
  fixtureRoot: "tests/_fixtures/differential/compiler/real-project-dom-reductions",
  promotedTest: "davinci_dom_corpus::promoted_real_project_dom_fixtures_agree",
};

const ssr: DivergenceObserver = {
  id: "ssr",
  label: "SSR",
  log: "ssr-pug.log",
  evidenceArtifact: "canonical-corpus-ssr-pug",
  parse: parseSsrDivergences,
  describe: (sample: SsrSample) =>
    `- ${code(sample.sourcePath)}: production lanes=${code(sample.lanes.join(", ") || "none")}${sample.selectedFailed ? "; the selected lane failed to compile" : ""}`,
  // The `vize_atelier_ssr` example built beside the reducer compiles each
  // candidate under the original corpus path as its bundler id.
  reducerArguments: (sourcePath, vizeBin) => [
    "--script",
    `${shellQuote(join(dirname(vizeBin), "examples", "ssr_lane_divergence"))} --name ${shellQuote(sourcePath)}`,
  ],
  fixtureRoot: "tests/_fixtures/differential/compiler/real-project-ssr-reductions",
  promotedTest: "davinci_ssr_corpus::promoted_real_project_ssr_fixtures_agree",
};

export const divergenceObservers: Readonly<Record<DivergenceObserver["id"], DivergenceObserver>> =
  Object.freeze({ dom, ssr });

export function divergenceObserver(id: string): DivergenceObserver {
  assert(Object.hasOwn(divergenceObservers, id), `Unknown divergence observer: ${id}`);
  return divergenceObservers[id as DivergenceObserver["id"]];
}

export function reducedFileName(observer: DivergenceObserver): string {
  return `canonical-${observer.id}-divergence.reduced.vue`;
}
