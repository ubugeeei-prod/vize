import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";

import type { Diagnostic } from "../../tools/benchmarks/scripts/typecheck-corpus-pair-parse.ts";
import {
  type LaneJudgement,
  type Mutation,
  type MutationSet,
  MUTATION_FAMILIES,
  gateMutations,
  judgeLane,
  loadMutationSet,
  mutateFile,
  mutateSources,
  referenceAnswer,
  renderMutations,
} from "../../tools/benchmarks/scripts/typecheck-corpus-mutations.ts";
import { CORPORA } from "../../tools/benchmarks/scripts/typecheck-corpus-pair.ts";
import { root } from "./support/github-workflows.ts";

const SOURCE = `<template>
  <Child :value="count" />
  <p>{{ label }}</p>
</template>
`;

const plant = (overrides: Partial<Mutation> = {}): Mutation => ({
  id: "prop",
  family: "prop-bound",
  file: "src/A.vue",
  anchor: ':value="count"',
  replacement: ":value=\"'1'\"",
  expect: { line: 2, column: 11, code: "TS2322" },
  ...overrides,
});
const typo = plant({
  id: "interp",
  family: "interpolation-identifier",
  anchor: "{{ label }}",
  replacement: "{{ labl }}",
  expect: { line: 3, column: 9, code: "TS2339", message: "'labl'" },
});
const set = (mutations: Mutation[]): MutationSet => ({
  corpus: "fixture",
  submodule: "tests/_fixtures/_git/fixture",
  revision: "0".repeat(40),
  mutations,
});
const diagnostic = (line: number, column: number, code: string, message = "m"): Diagnostic => [
  "src/A.vue",
  "error",
  String(line),
  String(column),
  code,
  message,
];

void test("mutations apply in place and keep every other line where it was", () => {
  const { text, regions } = mutateFile("src/A.vue", SOURCE, [plant(), typo]);
  assert.equal(text, SOURCE.replace(':value="count"', ":value=\"'1'\"").replace("label", "labl"));
  assert.deepEqual(regions.get("prop"), {
    file: "src/A.vue",
    start: { line: 2, column: 10 },
    end: { line: 2, column: 22 },
  });
});

void test("ambiguous, overlapping, same-line and misplaced mutations are rejected", () => {
  assert.throws(
    () => mutateFile("src/A.vue", `${SOURCE}${SOURCE}`, [plant()]),
    /occurs more than once/u,
  );
  assert.throws(
    () => mutateFile("src/A.vue", SOURCE, [plant(), plant({ id: "b", anchor: '="count" />' })]),
    /overlaps/u,
  );
  assert.throws(
    () =>
      mutateFile("src/A.vue", SOURCE, [
        plant(),
        plant({
          id: "b",
          anchor: "<Child",
          replacement: "<Kid",
          expect: { line: 2, column: 4, code: "TS1" },
        }),
      ]),
    /shares a line/u,
  );
  assert.throws(
    () =>
      mutateFile("src/A.vue", SOURCE, [plant({ expect: { line: 3, column: 3, code: "TS2322" } })]),
    /outside its replacement/u,
  );
});

void test("a plant counts every copy at its position, so a duplicate is not a catch", () => {
  const fixture = set([plant(), typo]);
  const { regions } = mutateSources(fixture, () => SOURCE);
  const pristine = [diagnostic(9, 1, "TS2307")];
  const judged = judgeLane(
    fixture,
    regions,
    [
      ...pristine,
      diagnostic(2, 11, "TS2322"),
      diagnostic(2, 11, "TS2322"),
      diagnostic(3, 9, "TS2339", "Property 'labl' does not exist"),
    ],
    pristine,
  );
  assert.deepEqual(judged.plants, {
    prop: { count: 2, near: [] },
    interp: { count: 1, near: [] },
  });
  assert.deepEqual(judged.collateral, { added: [], removed: [] });
});

void test("diagnostics outside the plants, new or lost, are collateral", () => {
  const fixture = set([plant()]);
  const { regions } = mutateSources(fixture, () => SOURCE);
  const judged = judgeLane(
    fixture,
    regions,
    [diagnostic(2, 15, "TS2345"), diagnostic(7, 1, "TS2322")],
    [diagnostic(9, 1, "TS2307")],
  );
  assert.deepEqual(judged.plants.prop, { count: 0, near: ["2:15 TS2345"] });
  assert.deepEqual(judged.collateral, {
    added: [diagnostic(2, 15, "TS2345"), diagnostic(7, 1, "TS2322")],
    removed: [diagnostic(9, 1, "TS2307")],
  });
  assert.equal(referenceAnswer(judged, "prop"), "near TS2345");
});

void test("the n8n fallthrough plant owns the inserted tabindex attribute", () => {
  const mutation = loadMutationSet("n8n").mutations.find(
    ({ id }) => id === "fallthrough-status-tabindex",
  );
  assert.ok(mutation);
  const source = `${"\n".repeat(147)}\t\t\t\t<TitledList :title="\`\${i18n.baseText('node.issues')}:\`" ${mutation.anchor}\n`;
  const result = mutateFile(mutation.file, source, [mutation]);
  const line = result.text.trimEnd().split("\n").at(-1)!;
  assert.equal(line.indexOf(":items") + 1, 61, "the previous prop starts at the stale column");
  assert.equal(
    line.slice(mutation.expect.column - 1, mutation.expect.column - 1 + "tabindex".length),
    "tabindex",
    "the expected diagnostic belongs to the newly inserted fallthrough attribute name",
  );
  assert.deepEqual(
    judgeLane(
      set([mutation]),
      result.regions,
      [
        [
          mutation.file,
          "error",
          String(mutation.expect.line),
          String(mutation.expect.column),
          mutation.expect.code,
          mutation.expect.message!,
        ],
      ],
      [],
    ).plants[mutation.id],
    { count: 1, near: [] },
  );
});

const lane = (counts: Record<string, number>, collateral = 0): LaneJudgement => ({
  plants: Object.fromEntries(
    Object.entries(counts).map(([id, count]) => [id, { count, near: [] }]),
  ),
  collateral: {
    added: Array.from({ length: collateral }, () => diagnostic(7, 1, "TS2322")),
    removed: [],
  },
});

void test("head fails on a lost plant, a duplicate and collateral, and passes a known issue", () => {
  const known = plant({ id: "slot", known: { issue: 12479, count: 2 } });
  const fixture = set([plant(), typo, known]);
  assert.deepEqual(
    gateMutations(
      fixture,
      { tsc: lane({ prop: 1, interp: 1, slot: 2 }) },
      { tsc: lane({ prop: 1, interp: 1, slot: 2 }) },
    ),
    { failures: [], flips: [] },
  );
  const broken = gateMutations(
    fixture,
    { tsc: lane({ prop: 1, interp: 1, slot: 2 }) },
    { tsc: lane({ prop: 0, interp: 2, slot: 0 }, 1) },
  );
  assert.deepEqual(broken.failures, [
    "fixture tsc prop: head reports 0x, base reports it once",
    "fixture tsc interp: head reports 2x, base reports it once",
    "fixture tsc slot: head reports 0x, expected once or 2x (#12479)",
    "fixture tsc: 1 collateral diagnostics added and 0 lost: +src/A.vue:error:7:1:TS2322",
  ]);
  const fixed = gateMutations(
    fixture,
    { tsc: lane({ prop: 1, interp: 1, slot: 2 }) },
    { tsc: lane({ prop: 1, interp: 1, slot: 1 }) },
  );
  assert.deepEqual(fixed, {
    failures: [],
    flips: ['fixture tsc slot: reported once; #12479 landed, drop "known"'],
  });
});

void test("the summary table lists every plant per lane and the gate verdict", () => {
  const fixture = set([plant(), typo]);
  const markdown = renderMutations(
    [
      {
        set: fixture,
        vize: {
          base: { tsc: lane({ prop: 1, interp: 1 }) },
          head: { tsc: lane({ prop: 0, interp: 1 }) },
        },
        references: { "vue-tsc": lane({ prop: 1, interp: 0 }) },
      },
    ],
    { failures: ["fixture tsc prop: head reports 0x, base reports it once"], flips: [] },
  );
  assert.match(
    markdown,
    /^\| Plant \| Family \| Expected \| base tsc \| head tsc \| vue-tsc \|$/mu,
  );
  assert.match(
    markdown,
    /^\| `prop` \| prop-bound \| A\.vue:2:11 TS2322 \| 1 \| \*\*0x\*\* \| exact \|$/mu,
  );
  assert.match(markdown, /^\| vize head tsc \| 1\/2 \| \+0\/-0 \|$/mu);
  assert.match(markdown, /^Gate: \*\*fail\*\*\.$/mu);
});

void test("each corpus set covers every family, pins the checked-out gitlink and names its issues", () => {
  for (const corpus of CORPORA) {
    const mutations = loadMutationSet(corpus.id);
    assert.ok(corpus.source.startsWith(`${mutations.submodule}/`));
    const tree = spawnSync("git", ["-C", root, "ls-tree", "HEAD", mutations.submodule], {
      encoding: "utf8",
    });
    assert.equal(tree.stdout.split(/\s/u)[2], mutations.revision);
    assert.ok(mutations.mutations.length >= 25 && mutations.mutations.length <= 40);
    const families = new Set(mutations.mutations.map((mutation) => mutation.family));
    for (const family of Object.keys(MUTATION_FAMILIES)) {
      assert.ok(
        families.has(family as Mutation["family"]) ||
          mutations.absent?.[family as Mutation["family"]],
        `${corpus.id}: ${family}`,
      );
    }
    for (const mutation of mutations.mutations) {
      assert.ok(mutation.expect.message, `${mutation.id}: message`);
      if (mutation.known) assert.ok(mutation.known.issue > 0 && mutation.known.count !== 1);
    }
  }
});
