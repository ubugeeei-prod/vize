import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import {
  cargoArguments,
  observerProcesses,
  prebuildTargets,
  processArguments,
  shardedObservers,
} from "../../tools/support/compat/github/canonical-corpus-shard-plan.ts";
import {
  parseShardVector,
  validateShardedObserver,
} from "../../tools/support/compat/github/canonical-corpus-shard-evidence.ts";
import { validateObserverLog } from "../../tools/support/compat/github/canonical-corpus-observer.mjs";
import { shardedObserverEvidence } from "./support/canonical-corpus-shard-logs.ts";
import type { ShardedEvidence } from "./support/canonical-corpus-shard-logs.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const paths = Array.from(
  { length: 61 },
  (_, index) => `project-${String(index % 7).padStart(2, "0")}/src/Original-${index}.vue`,
);
const files = paths.map((path) => [path, "a".repeat(64), 1, "b".repeat(40)]);
const identity = { files: paths.length, gitlinks: Array.from({ length: 7 }, () => ({})) };
const lines = (log: Buffer) => log.toString("utf8").split("\n");
const evidence = (shards: ShardedEvidence, rows: unknown[][] = files) => ({
  files: rows,
  readVector: (name: string) => {
    const vector = shards.vectors.get(name);
    assert(vector, `missing vector ${name}`);
    return vector;
  },
});
const validate = (observer: string, shards: ShardedEvidence, rows: unknown[][] = files) =>
  validateShardedObserver(observer, lines(shards.log), identity, evidence(shards, rows));
const replaceLog = (shards: ShardedEvidence, from: string | RegExp, to: string) => ({
  ...shards,
  log: Buffer.from(shards.log.toString("utf8").replace(from, to)),
});

await test("one build and one binary per suite: a base process plus disjoint sweep shards", () => {
  assert.deepEqual(Object.keys(shardedObservers).sort(), ["reach", "ssr-pug"]);
  const reach = observerProcesses("reach");
  assert.deepEqual(
    reach.map((entry) => entry.role),
    ["base", ...Array.from({ length: 16 }, () => "shard")],
  );
  assert.deepEqual(processArguments(reach[0]!), [
    "--exact",
    "--skip",
    "production_compiles_report_and_hold_their_davinci_reach",
    "--nocapture",
  ]);
  assert.deepEqual(processArguments(reach[3]!), [
    "--exact",
    "production_compiles_report_and_hold_their_davinci_reach",
    "--nocapture",
  ]);
  assert.equal(reach[3]!.vector, "reach-davinci_production_reach-shard-02-of-16.nul");
  const ssrPug = observerProcesses("ssr-pug");
  assert.deepEqual(
    ssrPug.map((entry) => `${entry.suite.test}:${entry.role}`),
    [
      "davinci_ssr_corpus:base",
      ...Array.from({ length: 12 }, () => "davinci_ssr_corpus:shard"),
      "davinci_pug_corpus:full",
    ],
  );
  assert.deepEqual(processArguments(ssrPug.at(-1)!), ["--nocapture"]);
  // The original commands: same packages, features and test targets.
  assert.deepEqual(
    Object.fromEntries(
      Object.entries(prebuildTargets).map(([observer, targets]) => [
        observer,
        targets.map((target) => cargoArguments(target, []).join(" ")),
      ]),
    ),
    {
      dom: ["test -p vize_l1_to_l2 --features legacy-differential --test davinci_dom_corpus"],
      "ssr-pug": [
        "test -p vize_atelier_ssr --features legacy-differential --test davinci_ssr_corpus",
        "test -p vize_l1_to_l2 --features legacy-differential --test davinci_pug_corpus",
      ],
      reach: [
        "test -p vize_atelier_sfc --features legacy-dom-differential --test davinci_production_reach",
      ],
    },
  );
  assert.throws(() => observerProcesses("dom"));
});

await test("complete shard vectors cover files.json exactly and sum to the whole corpus", () => {
  for (const observer of ["reach", "ssr-pug"]) {
    const shards = shardedObserverEvidence(observer, paths, 7);
    const vectors = validate(observer, shards);
    assert.equal(vectors.length, observer === "reach" ? 16 : 12);
    const counters: string[] = validateObserverLog(
      observer,
      shards.log,
      identity,
      evidence(shards),
    );
    assert(counters.some((line) => line.startsWith("davinci-differential corpus shard:")));
    assert.throws(
      () => validateObserverLog(observer, shards.log, identity),
      /requires the canonical manifest/,
    );
  }
});

await test("an omitted, duplicated, foreign or miscounted shard path fails closed", () => {
  const shards = shardedObserverEvidence("reach", paths, 7);
  const name = "reach-davinci_production_reach-shard-03-of-16.nul";
  const original = shards.vectors.get(name)!;
  const withVector = (bytes: Buffer) => ({
    ...shards,
    vectors: new Map([...shards.vectors, [name, bytes]]),
  });
  const vector = parseShardVector(original);
  // Dropping a file from both vector and counters still leaves files.json uncovered.
  const shorter = withVector(Buffer.from(`${vector.slice(1).join("\0")}\0`));
  assert.throws(() => validate("reach", shorter), /Shard vector is incomplete/);
  const dropped = replaceLog(shorter, /(index=3 count=16 planned=61 selected=)4/, "$13");
  assert.throws(() => validate("reach", dropped), /Shard counters differ|Shards do not cover/);
  const overlap = withVector(Buffer.from(`${[paths[0], ...vector.slice(1)].join("\0")}\0`));
  assert.throws(() => validate("reach", overlap), /Shards overlap/);
  const foreign = withVector(
    Buffer.from(`${["foreign/App.vue", ...vector.slice(1)].join("\0")}\0`),
  );
  assert.throws(() => validate("reach", foreign), /Shards omitted canonical file/);
  for (const bytes of [
    Buffer.from("a.vue"),
    Buffer.from("/abs.vue\0"),
    Buffer.from("a/../b.vue\0"),
  ])
    assert.throws(() => validate("reach", withVector(bytes)));
  const bigger = [...files, ["project-99/Added.vue", "a".repeat(64), 1, "b".repeat(40)]];
  assert.throws(() =>
    validateShardedObserver(
      "reach",
      lines(shards.log),
      { ...identity, files: 62 },
      evidence(shards, bigger),
    ),
  );
});

await test("every process must be present, clean and from the same binary", () => {
  const reach = shardedObserverEvidence("reach", paths, 7);
  const ssr = shardedObserverEvidence("ssr-pug", paths, 7);
  for (const [observer, shards, from, to] of [
    ["reach", reach, "role=shard index=15 count=16", "role=shard index=14 count=16"],
    ["reach", reach, /role=shard index=7 status=0/, "role=shard index=7 status=101"],
    ["reach", reach, /role=base status=0 signal=none/, "role=base status=0 signal=SIGKILL"],
    ["reach", reach, "0 measured; 1 filtered out", "0 measured; 2 filtered out"],
    [
      "reach",
      reach,
      /1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered/,
      "2 passed; 0 failed; 0 ignored; 0 measured; 16 filtered",
    ],
    [
      "reach",
      reach,
      /1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered/,
      "1 passed; 0 failed; 1 ignored; 0 measured; 16 filtered",
    ],
    ["reach", reach, "submodules=7", "submodules=6"],
    ["reach", reach, "closure_evidence=true", "closure_evidence=false"],
    ["reach", reach, /planned=61 selected/, "planned=60 selected"],
    [
      "reach",
      reach,
      /test production_compiles_report_and_hold_their_davinci_reach \.\.\. ok/,
      "test other ... ok",
    ],
    [
      "reach",
      reach,
      /shape=ssr stage="L2" templates=4 accepted=4 permille=1000 ready=0 legacy=\{\} rejected=0/,
      'shape=ssr stage="L2" templates=4 accepted=4 permille=1000 ready=0 legacy={} rejected=1',
    ],
    ["reach", reach, /compared=4 divergences=0/, "compared=4 divergences=1"],
    [
      "reach",
      reach,
      /scope=canonical files=4 unreadable=0/,
      "scope=canonical files=4 unreadable=1",
    ],
    [
      "reach",
      reach,
      /scope=canonical files=4 unreadable=0 parse_errors=0/,
      "scope=canonical files=4 unreadable=0 parse_errors=2",
    ],
    [
      "reach",
      reach,
      /target=vapor planned=4 native_only=0 legacy_backed=4/,
      "target=vapor planned=4 native_only=0 legacy_backed=3",
    ],
    [
      "reach",
      reach,
      "   Compiling",
      "davinci production reach: scope=canonical files=61\n   Compiling",
    ],
    [
      "reach",
      reach,
      /\ncanonical observer process exit: test=davinci_production_reach role=shard index=15[^\n]*\n$/,
      "\n",
    ],
    ["ssr-pug", ssr, /scope=canonical files=6 parsed=6/, "scope=canonical files=6 parsed=5"],
    ["ssr-pug", ssr, /s4=6 error_skips=0 rejected=0/, "s4=6 error_skips=0 rejected=1"],
    [
      "ssr-pug",
      ssr,
      /legacy_error_skips=0 rejected=0 divergences=0/,
      "legacy_error_skips=0 rejected=0 divergences=3",
    ],
    ["ssr-pug", ssr, "pug=3 refused=0", "pug=3 refused=1"],
    ["ssr-pug", ssr, "davinci pug corpus sweep: files=61", "davinci pug corpus sweep: files=60"],
    ["ssr-pug", ssr, "pug=3 refused=0 compiled_lanes=9", "pug=0 refused=0 compiled_lanes=0"],
    ["ssr-pug", ssr, "role=full\n", "role=base\n"],
  ] as const) {
    const changed = replaceLog(shards, from, to);
    assert.notDeepEqual(changed.log, shards.log, String(from));
    assert.throws(() => validate(observer, changed), String(from));
  }
});

await test("the observer workflow builds in the background and runs each sharded observer", () => {
  type Step = { name?: string; run?: string; uses?: string; with?: Record<string, unknown> };
  const workflow = parse(readRepoFile(".github", "workflows", "davinci-canonical-corpus.yml"));
  const steps = workflow.jobs["canonical-observers"].steps as Step[];
  const names = steps.map((step) => step.name);
  const step = (name: string) => steps.find((candidate) => candidate.name === name) as Step;
  const position = (name: string) => names.indexOf(name);
  assert.equal(
    step("Start observer test build").run,
    'nohup node tools/support/compat/github/canonical-corpus-shards.ts prebuild "$OBSERVER" > "$RUNNER_TEMP/canonical-prebuild.log" 2>&1 &',
  );
  assert(position("Setup Node") < position("Start observer test build"));
  assert(
    position("Start observer test build") < position("Select and hydrate full fixture corpus"),
  );
  assert(
    steps.findIndex((candidate) => candidate.uses === "./.github/actions/setup-rust-sticky-cache") <
      position("Start observer test build"),
  );
  assert.equal(
    step("Run L4 SSR and pug L1 differential corpora").run,
    "node tools/support/compat/github/canonical-corpus-shards.ts run ssr-pug ssr-pug.log",
  );
  assert.equal(
    step("Measure Davinci production reach on the hydrated corpus").run,
    "node tools/support/compat/github/canonical-corpus-shards.ts run reach reach.log",
  );
  for (const name of [
    "Restore immutable fixture objects",
    "Cache immutable trusted fixture objects",
  ])
    assert.deepEqual(step(name).with, {
      path: ".git/modules/tests/_fixtures/_git",
      key: "${{ steps.corpus-plan.outputs.cache-key }}",
      "restore-keys": "${{ steps.corpus-plan.outputs.cache-restore-key }}",
    });
  const finalizer = workflow.jobs["davinci-dom-corpus"].steps as Step[];
  assert(
    finalizer.findIndex((candidate) => candidate.name === "Setup Node") <
      finalizer.findIndex((candidate) => candidate.name === "Select complete canonical workers"),
  );
});

await test("only trusted main seeds the fixture objects that PR and queue observers restore", () => {
  type Step = { name?: string; run?: string; uses?: string; if?: string; with?: unknown };
  const workflow = parse(readRepoFile(".github", "workflows", "canonical-corpus-objects.yml"));
  assert.deepEqual(workflow.on.push, {
    branches: ["main"],
    paths: [
      ".gitmodules",
      "tests/_fixtures/_git/**",
      ".github/workflows/canonical-corpus-objects.yml",
    ],
  });
  assert.equal(workflow.on.pull_request, undefined);
  assert.equal(workflow.on.merge_group, undefined);
  assert.deepEqual(workflow.permissions, { contents: "read" });
  const job = workflow.jobs.seed;
  assert.equal(job.if, "${{ github.ref == 'refs/heads/main' }}");
  const steps = job.steps as Step[];
  const step = (name: string) => steps.find((candidate) => candidate.name === name) as Step;
  assert.deepEqual(step("Checkout trusted main").with, {
    "persist-credentials": false,
    ref: "${{ github.sha }}",
  });
  assert.equal(
    step("Plan canonical corpus identity").run,
    "node tools/support/compat/github/canonical-corpus-observer.mjs plan",
  );
  assert.equal(
    step("Require a trusted cache writer").if,
    "${{ steps.corpus-plan.outputs.cache-trusted != 'true' }}",
  );
  const cache = step("Cache immutable trusted fixture objects");
  assert.match(cache.uses ?? "", /^actions\/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae$/);
  assert.deepEqual(cache.with, {
    path: ".git/modules/tests/_fixtures/_git",
    key: "${{ steps.corpus-plan.outputs.cache-key }}",
    "restore-keys": "${{ steps.corpus-plan.outputs.cache-restore-key }}",
  });
  assert.equal(
    step("Fetch pinned fixture objects").run,
    "git submodule update --init --checkout --depth 1 --jobs 32 -- tests/_fixtures/_git",
  );
  const order = steps.map((candidate) => candidate.name);
  assert(
    order.indexOf("Require a trusted cache writer") <
      order.indexOf("Cache immutable trusted fixture objects"),
  );
});
