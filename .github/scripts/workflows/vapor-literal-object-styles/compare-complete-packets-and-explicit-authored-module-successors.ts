import { pathToFileURL } from "node:url";
import { resolve as resolveWorkflowInput } from "node:path";
import assert from "node:assert/strict";
import fs from "node:fs";
const {
  assertVdomChildStylePackets: assertChildStylePackets,
  assertVdomChildPreservedStylePacket: assertChildPreservedStylePacket,
} = (await import(
  pathToFileURL(resolveWorkflowInput("./tests/tooling/support/jsx-vdom-child-style-successors.ts"))
    .href
)) as typeof import("../../../../tests/tooling/support/jsx-vdom-child-style-successors.ts");
const { assertVaporSfcExposeTransition } = (await import(
  pathToFileURL(resolveWorkflowInput("./tests/_helpers/vapor-ssr-expose-successor.ts")).href
)) as typeof import("../../../../tests/_helpers/vapor-ssr-expose-successor.ts");
const { assertComposedVaporSfcStylePackets } = (await import(
  pathToFileURL(
    resolveWorkflowInput("./tests/tooling/support/vapor-style-successor-composition.ts"),
  ).href
)) as typeof import("../../../../tests/tooling/support/vapor-style-successor-composition.ts");
const { assertCurrentSfcModulePackets } = (await import(
  pathToFileURL(resolveWorkflowInput("./tests/tooling/support/sfc-style-raw-module-successors.ts"))
    .href
)) as typeof import("../../../../tests/tooling/support/sfc-style-raw-module-successors.ts");
type StyleRow = Record<string, unknown> & {
  output: Record<string, unknown> & { code: string };
  expressionReparses: number;
  fold: boolean;
  vapor: boolean;
  family: string;
  source: string;
  mode: string;
  ssr: boolean;
  counters: { name: string; total: number }[];
  samplesNs: number[];
  arenaBytes: number;
};
const directory = `${process.env.RUNNER_TEMP}/vapor-literal-styles`;
const base = JSON.parse(fs.readFileSync(`${directory}/base.json`, "utf8")) as StyleRow[];
const head = JSON.parse(fs.readFileSync(`${directory}/head.json`, "utf8")) as StyleRow[];
assert.equal(head.length, 180);
assert.equal(base.length, head.length);
const originalHead = JSON.parse(
  fs.readFileSync(
    "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/observed-head.json",
    "utf8",
  ),
) as StyleRow[];
assert.equal(originalHead.length, head.length);
const childOutputSuccessors = assertChildStylePackets("main", head);
head.forEach((row: { output: Record<string, unknown> }, index: number) =>
  assertChildPreservedStylePacket("main", index, originalHead[index].output, row.output),
);
const beforeCase = JSON.parse(
  fs.readFileSync(`${directory}/base-attribute-case.json`, "utf8"),
) as StyleRow[];
const afterCase = JSON.parse(
  fs.readFileSync(`${directory}/head-attribute-case.json`, "utf8"),
) as StyleRow[];
assert.equal(beforeCase.length, 48);
assert.equal(afterCase.length, 48);
assertChildStylePackets("attribute-case", afterCase);
afterCase.forEach(
  (row: { [x: string]: unknown; output: Record<string, unknown> }, index: number) => {
    const previous = beforeCase[index];
    for (const key of ["id", "family", "source", "standalone", "retained", "mapped"])
      assert.deepEqual((row as Record<string, unknown>)[key], previous[key]);
    assertChildPreservedStylePacket("attribute-case", index, previous.output, row.output);
  },
);
let changed = 0;
for (let index = 0; index < head.length; index++) {
  const current = head[index],
    previous = base[index];
  for (const key of [
    "id",
    "family",
    "source",
    "retained",
    "mapped",
    "mode",
    "ssr",
    "standalone",
    "fold",
  ])
    assert.deepEqual(current[key], previous[key]);
  assert.ok(
    current.expressionReparses <= previous.expressionReparses,
    `no new expression parsing ${index}`,
  );
  if (!current.fold)
    assertChildPreservedStylePacket("main", index, previous.output, current.output);
  else {
    assert.notDeepEqual(
      current.output,
      previous.output,
      `real fold must alter the full packet ${index}`,
    );
    changed++;
  }
  if (current.family === "jsx" && (current.mode === "Vdom" || current.ssr)) {
    assert.equal(
      current.counters.find(
        (row: { name: string }) => row.name === "relief.inline_style.literal_object_candidates",
      )?.total ?? 0,
      0,
    );
  }
}
const beforeReview = JSON.parse(
  fs.readFileSync(`${directory}/base-review.json`, "utf8"),
) as StyleRow[];
const afterReview = JSON.parse(
  fs.readFileSync(`${directory}/head-review.json`, "utf8"),
) as StyleRow[];
assert.equal(beforeReview.length, 48);
assert.equal(afterReview.length, 48);
assertChildStylePackets("review", afterReview);
afterReview.forEach((row, index) => {
  const previous = beforeReview[index];
  for (const key of ["id", "family", "source", "standalone", "retained", "mapped", "fold"])
    assert.deepEqual((row as Record<string, unknown>)[key], previous[key]);
  if (row.fold) assert.notDeepEqual(row.output, previous.output);
  else assertChildPreservedStylePacket("review", index, previous.output, row.output);
});
const beforePriority = JSON.parse(
  fs.readFileSync(`${directory}/base-priority.json`, "utf8"),
) as StyleRow[];
const afterPriority = JSON.parse(
  fs.readFileSync(`${directory}/head-priority.json`, "utf8"),
) as StyleRow[];
assert.equal(beforePriority.length, 48);
assert.equal(afterPriority.length, 48);
assertChildStylePackets("priority", afterPriority);
afterPriority.forEach((row, index) => {
  const previous = beforePriority[index];
  for (const key of ["id", "family", "source", "standalone", "retained", "mapped", "fold"])
    assert.deepEqual((row as Record<string, unknown>)[key], previous[key]);
  if (row.fold) assert.notDeepEqual(row.output, previous.output);
  else assertChildPreservedStylePacket("priority", index, previous.output, row.output);
  assert.ok(row.expressionReparses <= previous.expressionReparses);
});
const beforeSfc = JSON.parse(fs.readFileSync(`${directory}/base-sfc.json`, "utf8")) as StyleRow[];
const observedSfc = JSON.parse(fs.readFileSync(`${directory}/head-sfc.json`, "utf8")) as StyleRow[];
const afterSfc = assertCurrentSfcModulePackets(observedSfc);
function sfcRows(rows: typeof afterSfc): asserts rows is ((typeof rows)[number] & {
  source: string;
  vapor: boolean;
  ssr: boolean;
  fold: boolean;
  output: { code: string };
})[] {
  for (const row of rows) {
    assert.equal(typeof row.source, "string");
    assert.equal(typeof row.output.code, "string");
    for (const key of ["vapor", "ssr", "fold"]) assert.equal(typeof row[key], "boolean");
  }
}
sfcRows(afterSfc);
assert.equal(beforeSfc.length, 4);
assert.equal(afterSfc.length, 4);
assertChildStylePackets("sfc", afterSfc);
assertVaporSfcExposeTransition(beforeSfc, afterSfc);
assertComposedVaporSfcStylePackets(beforeSfc, afterSfc);
afterSfc.forEach((row, index) => {
  const previous = beforeSfc[index];
  for (const key of ["source", "vapor", "ssr", "fold"])
    assert.deepEqual((row as Record<string, unknown>)[key], previous[key]);
  if (row.fold) assert.notDeepEqual(row.output, previous.output);
  else assertChildPreservedStylePacket("sfc", index, previous.output, row.output);
});
const beforeProfile = JSON.parse(
  fs.readFileSync(`${directory}/base-profile.json`, "utf8"),
) as StyleRow[];
const afterProfile = JSON.parse(
  fs.readFileSync(`${directory}/head-profile.json`, "utf8"),
) as StyleRow[];
assert.equal(beforeProfile.length, 24);
assert.equal(afterProfile.length, 24);
const median = (xs: number[]) => xs.toSorted((a, b) => a - b)[Math.floor(xs.length / 2)];
const measurements = afterProfile.map((row, index) => {
  const previous = beforeProfile[index];
  for (const key of [
    "id",
    "family",
    "source",
    "mode",
    "ssr",
    "standalone",
    "retained",
    "iterationsPerSample",
  ])
    assert.deepEqual((row as Record<string, unknown>)[key], previous[key]);
  assert.equal(row.samplesNs.length, 9);
  assert.equal(previous.samplesNs.length, 9);
  return {
    id: row.id,
    family: row.family,
    mode: row.mode,
    ssr: row.ssr,
    standalone: row.standalone,
    retained: row.retained,
    baseMedianNs: median(previous.samplesNs),
    headMedianNs: median(row.samplesNs),
    ratio: median(row.samplesNs) / median(previous.samplesNs),
    baseArenaBytes: previous.arenaBytes,
    headArenaBytes: row.arenaBytes,
    baseExpressionReparses: previous.expressionReparses,
    headExpressionReparses: row.expressionReparses,
  };
});
fs.writeFileSync(
  `${directory}/measurement.json`,
  JSON.stringify(
    {
      scope:
        "CI source full-compile observer including parsing/lowering/codegen/JSON; arena used bytes excludes owned heap; changed-work timings descriptive only",
      performanceQualified: false,
      equivalentWorkTiming: false,
      measurements,
    },
    null,
    2,
  ) + "\n",
);
fs.writeFileSync(
  `${directory}/comparison.json`,
  JSON.stringify(
    {
      scope:
        "source-only; complete raw outputs; explicit authored module successors preserve all other facets; no browser or published artifact credit",
      rows: head.length,
      styleFoldChanges: changed,
      styleConservativeRows: head.length - changed,
      childOutputSuccessors,
      vaporChildOutputSuccessors: 30,
      vdomChildOutputSuccessors: 12,
      runtimeQualified: false,
      performanceQualified: false,
    },
    null,
    2,
  ) + "\n",
);
