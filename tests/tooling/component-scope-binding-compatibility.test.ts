import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import test from "node:test";
import {
  classifyScopePair,
  intendedScopeChanges,
  intendedScopeSources,
  requireScopeSourceTransition,
} from "../../tools/benchmarks/scripts/component-scope-binding-compatibility.ts";

test("only the exact two complete-packet source-bound transitions are incomparable", () => {
  for (const known of intendedScopeChanges) {
    const pair = { ...known, count: 32, scriptBindings: 10000, identical: false };
    assert.equal(classifyScopePair(pair), "incomparable");
    for (const change of [
      { count: 128 },
      { scriptBindings: 32 },
      { shape: "missing" },
      { sourceSha256: "0".repeat(64) },
      { beforePacketSha256: "0".repeat(64) },
      { afterPacketSha256: "0".repeat(64) },
      { beforePacketSha256: pair.afterPacketSha256, afterPacketSha256: pair.beforePacketSha256 },
    ])
      assert.equal(classifyScopePair({ ...pair, ...change }), "unexpected");
  }
});

test("whole equality is separate from an incomparable or unexpected packet", () => {
  const pair = {
    shape: "camel",
    count: 32,
    scriptBindings: 32,
    sourceSha256: "a".repeat(64),
    beforePacketSha256: "b".repeat(64),
    afterPacketSha256: "b".repeat(64),
    identical: true,
  };
  assert.equal(classifyScopePair(pair), "equal");
  assert.equal(classifyScopePair({ ...pair, identical: false }), "unexpected");
  // The existing comparator checks complete JSON values, not object insertion order.
  assert.equal(classifyScopePair({ ...pair, afterPacketSha256: "c".repeat(64) }), "equal");
});

test("missing, extra, reordered or changed source identities refuse the controlled path", () => {
  requireScopeSourceTransition([...intendedScopeSources]);
  for (const actual of [
    intendedScopeSources.slice(1),
    [...intendedScopeSources, intendedScopeSources[0]],
    [...intendedScopeSources].reverse(),
    intendedScopeSources.map((row) => ({ ...row, beforeSha256: "0".repeat(64) })),
    intendedScopeSources.map((row) => ({ ...row, afterSha256: "0".repeat(64) })),
  ])
    assert.throws(() => requireScopeSourceTransition(actual));
});

test("the refusal precedes all normal clocks and the existing strict comparable path remains", () => {
  const source = fs.readFileSync(
    new URL("../../tools/benchmarks/scripts/component-scope-binding-evidence.ts", import.meta.url),
    "utf8",
  );
  const preflight = source.indexOf("auditScopeCompatibility(root, provenance)");
  assert(preflight > 0 && preflight < source.indexOf("const profiles:"));
  assert(source.indexOf('status: "INCOMPARABLE"') < source.indexOf("const normal:"));
  assert(source.includes("qualification: false"));
  assert(source.includes('normal: "Not executed: whole output parity is absent"'));
  assert(source.includes('"the entire generation packet must remain identical"'));
  assert(source.includes('"all four normal mapper packets must remain identical"'));
  assert(source.includes("number(headAlloc.requestedBytes) <= number(baseAlloc.requestedBytes)"));
  const start = source.indexOf("  const incomparable = auditScopeCompatibility(root, provenance);");
  const end = source.indexOf("  const profiles: ObjectValue[] = [];", start);
  const inverse = (source.slice(0, start) + source.slice(end))
    .replace(
      'import { auditScopeCompatibility } from "./component-scope-binding-compatibility.ts";\n',
      "",
    )
    .replace("    qualification: true,\n", "");
  assert.equal(
    createHash("sha256").update(inverse).digest("hex"),
    "cfe37531a9f638fdeb115e2ef7ccc08c3e4aa9918bd261af8b2765faf682daea",
  );
});
