// Synthetic process-sharded observer evidence in the exact shape the shard
// runner writes: one log with a segment per process plus one NUL-terminated
// path vector per shard. Counters are synthetic; parsers and validators are
// the real ones.
import {
  exitLine,
  observerProcesses,
  processHeader,
} from "../../../tools/support/compat/github/canonical-corpus-shard-plan.ts";
import type { ObserverProcess } from "../../../tools/support/compat/github/canonical-corpus-shard-plan.ts";

const shapes = ["dom_inline", "dom_module", "ssr", "vapor"];

function segmentLines(
  entry: ObserverProcess,
  paths: readonly string[],
  selected: readonly string[],
  gitlinks: number,
): string[] {
  const { suite } = entry;
  const total = suite.minimumTests;
  const closure = `davinci-differential corpus scope: root=/repo/tests/davinci_test_support/../../tests/_fixtures/_git scope=canonical closure_evidence=true submodules=${gitlinks}`;
  if (entry.role === "base")
    return [
      `running ${total - 1} tests`,
      "test synthetic_control ... ok",
      `test result: ok. ${total - 1} passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.10s`,
    ];
  if (entry.role === "full")
    return [
      "running 1 test",
      closure,
      `davinci pug corpus sweep: files=${paths.length} pug=3 refused=0 compiled_lanes=9`,
      `test ${suite.test} ... ok`,
      "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s",
    ];
  const files = selected.length;
  const counters =
    suite.kind === "reach"
      ? [
          "davinci production reach: scope=committed files=551 unreadable=0 parse_errors=0 without_template=91",
          ...shapes.map(
            (shape) =>
              `davinci production reach: shape=${shape} stage="L2" templates=458 accepted=442 permille=965 ready=0 legacy={} rejected=0 routed_vapor=0 unrecorded=0 sfc_errors={} compared=442 divergences=0`,
          ),
          `davinci production reach: scope=canonical files=${files} unreadable=0 parse_errors=0 without_template=0`,
          ...shapes.flatMap((shape) => [
            `davinci production reach: shape=${shape} stage="L2" templates=${files} accepted=${files} permille=1000 ready=0 legacy={} rejected=0 routed_vapor=0 unrecorded=0 sfc_errors={} compared=${files} divergences=0`,
            `davinci native-only acceptance: scope=canonical product=compiler target=${shape} planned=${files} native_only=0 legacy_backed=${files} unverified=0 permille=0 contribution=sfc_parse:legacy:legacy`,
          ]),
        ]
      : [
          `davinci SSR production sweep: scope=canonical templates=${files} compared=${files} s4=${files} error_skips=0 rejected=0 divergences=0`,
          `davinci SSR production lanes: {"s4": ${files}}`,
          `davinci SSR corpus sweep: scope=canonical files=${files} parsed=${files} templates=${files} compared=${files} s4=${files} legacy_error_skips=0 rejected=0 divergences=0`,
          `davinci SSR corpus lanes: {"s4": ${files}}`,
        ];
  return [
    "running 1 test",
    closure,
    `davinci-differential corpus shard: scope=canonical index=${entry.index} count=${suite.shards} planned=${paths.length} selected=${files}`,
    ...counters,
    `test ${suite.sweep} ... ok`,
    "",
    `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; ${total - 1} filtered out; finished in 1.00s`,
  ];
}

export type ShardedEvidence = { log: Buffer; vectors: Map<string, Buffer> };

export function shardedObserverEvidence(
  observer: string,
  paths: readonly string[],
  gitlinks: number,
): ShardedEvidence {
  const vectors = new Map<string, Buffer>();
  const lines = ["   Compiling davinci_test_support v0.0.0", "    Finished `test` profile"];
  for (const entry of observerProcesses(observer)) {
    const selected =
      entry.role === "shard"
        ? paths.filter((_, position) => position % entry.suite.shards === entry.index)
        : [];
    if (entry.vector)
      vectors.set(
        entry.vector,
        Buffer.from(selected.length ? `${selected.join("\0")}\0` : "", "utf8"),
      );
    lines.push(
      processHeader(entry),
      ...segmentLines(entry, paths, selected, gitlinks),
      exitLine(entry, { status: 0, signal: null, elapsed: 1 }),
    );
  }
  return { log: Buffer.from(`${lines.join("\n")}\n`), vectors };
}
