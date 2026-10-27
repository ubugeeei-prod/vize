import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import {
  assertVaporSfcExposeTransition,
  assertVaporSsrExposeSuccessor,
  frozenVaporSsrExposePair,
  verifyVaporSsrExposeRegistration,
} from "../_helpers/vapor-ssr-expose-successor.ts";

const directory = new URL(
  "../_fixtures/differential/compiler/vapor-literal-object-styles-10749/ssr-expose-successor-10923/",
  import.meta.url,
);
const read = (file: string) => JSON.parse(fs.readFileSync(new URL(file, directory), "utf8"));

test("both complete original SFC archives retain exact source and artifact custody", () => {
  const admission = read("admission.json");
  verifyVaporSsrExposeRegistration(admission);
  for (const [file, hash] of Object.entries(admission.rawFiles)) {
    assert.equal(
      createHash("sha256")
        .update(fs.readFileSync(new URL(file, directory)))
        .digest("hex"),
      hash,
    );
  }
  const producer = read("producer-capture.json");
  assert.deepEqual(admission.archive, {
    sourceHead: producer.sourceHead,
    sourceTree: producer.sourceTree,
    runId: producer.runId,
    artifactId: producer.artifactId,
    artifactDigest: producer.artifactDigest,
  });
  assert.match(admission.scope, /no JSX feature dependency or runtime\/native parity credit/);
  assert.equal(read("base-sfc.json").length, 4);
  assert.equal(read("head-sfc.json").length, 4);
});

test("only SSR rows 1 and 3 admit one default expose edit with every other field unchanged", () => {
  for (const index of [1, 3]) {
    const { before, after } = frozenVaporSsrExposePair(index);
    assertVaporSsrExposeSuccessor(index, before, after);
    assert.equal(after.output.code.split("    __expose();").length - 1, 1);
    assert.deepEqual(after.output, {
      ...before.output,
      code: before.output.code.replace(
        "setup(__props) {",
        "setup(__props, { expose: __expose }) {\n    __expose();",
      ),
    });
    assert.throws(() => assertVaporSsrExposeSuccessor(index, after, after), /original complete/);
    assert.throws(() => assertVaporSsrExposeSuccessor(index, before, before), /current complete/);
  }
  for (const index of [0, 2, -1, 4, 1.5]) assert.throws(() => frozenVaporSsrExposePair(index));
});

test("extra, omitted or mutated output facets on either actual side fail closed", () => {
  for (const index of [1, 3]) {
    const { before, after } = frozenVaporSsrExposePair(index);
    for (const side of ["before", "after"] as const) {
      for (const facet of Object.keys(before.output)) {
        for (const mutation of ["replace", "delete"]) {
          const pair = structuredClone({ before, after });
          if (mutation === "delete") delete pair[side].output[facet];
          else pair[side].output[facet] = "changed";
          assert.throws(
            () => assertVaporSsrExposeSuccessor(index, pair.before, pair.after),
            `${index}/${side}/${facet}/${mutation}`,
          );
        }
      }
      const pair = structuredClone({ before, after });
      pair[side].output.extra = true;
      assert.throws(() => assertVaporSsrExposeSuccessor(index, pair.before, pair.after));
    }
  }
});

test("source metadata, missing expose calls and extra expose calls cannot qualify", () => {
  for (const index of [1, 3]) {
    const { before, after } = frozenVaporSsrExposePair(index);
    for (const key of ["source", "ssr", "vapor", "fold"] as const) {
      const changed = { ...after, [key]: key === "source" ? "different" : !after[key] };
      assert.throws(() => assertVaporSsrExposeSuccessor(index, before, changed));
    }
    for (const code of [
      after.output.code.replace("    __expose();", ""),
      after.output.code.replace("    __expose();", "    __expose();\n    __expose();"),
    ])
      assert.throws(() =>
        assertVaporSsrExposeSuccessor(index, before, {
          ...after,
          output: { ...after.output, code },
        }),
      );
  }
});

test("registration cannot widen indices, facets, provenance, edit anchors or immutable pins", () => {
  const original = read("admission.json");
  for (const key of [
    "indices",
    "changedFacets",
    "sourceMerge",
    "sourcePullRequest",
    "codeEdit",
    "archive",
    "rawFiles",
  ]) {
    const changed = structuredClone(original);
    if (key === "indices") changed.indices = [0, 1, 2, 3];
    else if (key === "changedFacets") changed.changedFacets = ["code", "errors"];
    else changed[key] = "changed";
    assert.throws(() => verifyVaporSsrExposeRegistration(changed), key);
  }
  assert.throws(() =>
    verifyVaporSsrExposeRegistration({ ...original, jsxFeatureDependency: true }),
  );
});

test("the actual workflow dispatch rejects both-side fold reclassification and missing rows", () => {
  const before = read("base-sfc.json");
  const after = read("head-sfc.json");
  assertVaporSfcExposeTransition(before, after);
  for (const index of [1, 3]) {
    const changedBefore = structuredClone(before);
    const changedAfter = structuredClone(after);
    changedBefore[index].fold = true;
    changedAfter[index].fold = true;
    assert.throws(() => assertVaporSfcExposeTransition(changedBefore, changedAfter));
  }
  const changedBefore = structuredClone(before);
  const changedAfter = structuredClone(after);
  changedBefore[0].fold = true;
  changedAfter[0].fold = true;
  changedAfter[0].output.code += "\nchanged non-SSR output";
  assert.throws(() => assertVaporSfcExposeTransition(changedBefore, changedAfter));
  changedBefore[0] = before[0];
  changedAfter[0] = after[0];
  changedBefore[2].fold = false;
  changedAfter[2].fold = false;
  changedAfter[2].output = structuredClone(changedBefore[2].output);
  assert.throws(() => assertVaporSfcExposeTransition(changedBefore, changedAfter));
  assert.throws(() => assertVaporSfcExposeTransition(before.slice(1), after.slice(1)));
  assert.throws(() => assertVaporSfcExposeTransition([...before, before[0]], [...after, after[0]]));
  const missingBefore = structuredClone(before);
  const missingAfter = structuredClone(after);
  delete missingBefore[1];
  delete missingAfter[1];
  assert.throws(() => assertVaporSfcExposeTransition(missingBefore, missingAfter));
});
