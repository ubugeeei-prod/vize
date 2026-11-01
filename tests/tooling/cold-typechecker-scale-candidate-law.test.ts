import assert from "node:assert/strict";
import { test } from "node:test";
import {
  verifyComponentChecksTransition,
  classifyRoutePopulation,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-law.ts";

const cwd = "/owned/project/apps/web";
const assets = {
  before: Buffer.from('declare module "vize:virtual-ts" {\nexport type Props<C> = C;\n}\n'),
  after: Buffer.from(
    'declare module "vize:virtual-ts" {\ntype Meta<C> = { props: C };\nexport type Props<C> = Meta<C>["props"];\n}\n',
  ),
};
const source = "export const component = 1;\n";
const stdout = Buffer.from(
  JSON.stringify({
    programs: [{ root: ".", opaque: { future: [1, "雪🌸"] } }],
    files: [{ file: "Component.vue", virtualTs: source, opaque: true }],
    opaque: { bytes: "unchanged" },
  }),
);
const suffix = Buffer.from(`\n=== ${cwd}/Component.vue ===\n${source}\n`);
const debug = (asset: Buffer, tail = "// other helper bytes\n") =>
  Buffer.concat([
    Buffer.from("\n=== __vize_helpers.d.ts ===\n// before asset\n"),
    asset,
    Buffer.from(tail),
    suffix,
  ]);

test("one complete component asset reconstructs all after bytes without mutating originals", () => {
  const before = { stdout, stderr: debug(assets.before) };
  const after = { stdout: Buffer.from(stdout), stderr: debug(assets.after) };
  const saved = [before.stdout, before.stderr, after.stdout, after.stderr].map((value) =>
    Buffer.from(value),
  );
  const result = verifyComponentChecksTransition(before, after, cwd, assets);
  assert.equal(result.sourceAssetOccurrences, 1);
  assert.equal(result.beforeAssetBytes, assets.before.length);
  assert.equal(result.afterAssetBytes, assets.after.length);
  [before.stdout, before.stderr, after.stdout, after.stderr].forEach((value, index) =>
    assert(value.equals(saved[index])),
  );
});

test("unknown fields, non-asset helper changes, missing/duplicate asset and geometry refuse", () => {
  const before = { stdout, stderr: debug(assets.before) };
  const after = { stdout, stderr: debug(assets.after) };
  for (const changed of [
    { ...after, stdout: Buffer.concat([stdout, Buffer.from("\n")]) },
    { ...after, stdout: Buffer.from(stdout.toString().replace('"opaque":true', '"opaque":false')) },
    { ...after, stderr: debug(assets.after, "// changed opaque helper bytes\n") },
    { ...after, stderr: Buffer.concat([after.stderr, Buffer.from("extra")]) },
    {
      ...after,
      stderr: Buffer.from(after.stderr.toString().replace("component = 1", "component = 2")),
    },
  ])
    assert.throws(() => verifyComponentChecksTransition(before, changed, cwd, assets));
  for (const changed of [
    { ...before, stderr: debug(Buffer.from("missing source asset\n")) },
    { ...before, stderr: debug(Buffer.concat([assets.before, assets.before])) },
  ])
    assert.throws(() => verifyComponentChecksTransition(changed, after, cwd, assets));
  assert.throws(() => verifyComponentChecksTransition(before, after, "relative", assets));
  assert.throws(() =>
    verifyComponentChecksTransition(before, after, cwd, {
      before: assets.before,
      after: assets.before,
    }),
  );
});

test("empty and all-negative populations remain distinct; capacities stay unknown", () => {
  const empty = classifyRoutePopulation({
    bindings: 0,
    positiveBindings: 0,
    negativeBindings: 0,
    bindingInlineInitializedBytes: 0,
  });
  assert.equal(empty.population, "empty-binding-vector");
  assert.equal(empty.initializedBindingCloneWorkPresent, false);
  const negative = classifyRoutePopulation({
    bindings: 2,
    positiveBindings: 0,
    negativeBindings: 2,
    bindingInlineInitializedBytes: 128,
  });
  assert.equal(negative.population, "nonempty-negative-bindings");
  assert.equal(negative.initializedBindingCloneWorkPresent, true);
  assert.equal(negative.outerBindingVectorCapacity, "unobserved");
  assert.equal(
    classifyRoutePopulation({
      bindings: 2,
      positiveBindings: 1,
      negativeBindings: 1,
      bindingInlineInitializedBytes: 128,
    }).population,
    "positive-route-payload",
  );
  for (const counts of [
    { bindings: 1, positiveBindings: 0, negativeBindings: 0, bindingInlineInitializedBytes: 64 },
    { bindings: 0, positiveBindings: 0, negativeBindings: 0, bindingInlineInitializedBytes: 64 },
    { bindings: 1, positiveBindings: 1, negativeBindings: 0, bindingInlineInitializedBytes: 0 },
    { bindings: -1, positiveBindings: 0, negativeBindings: -1, bindingInlineInitializedBytes: 0 },
    {
      bindings: Number.MAX_SAFE_INTEGER + 1,
      positiveBindings: 0,
      negativeBindings: Number.MAX_SAFE_INTEGER + 1,
      bindingInlineInitializedBytes: 128,
    },
  ])
    assert.throws(() => classifyRoutePopulation(counts));
});
