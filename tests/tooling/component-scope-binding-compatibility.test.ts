import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  auditScopeCompatibility,
  scopeBindingMode,
  scopeObservation,
  scopePacketEqual,
  scopeCorrectnessReport,
} from "../../tools/benchmarks/scripts/component-scope-binding-compatibility.ts";
const root = fileURLToPath(new URL("../..", import.meta.url));

test("constructor packets use the shared exact-transition certifier", () => {
  const workflow = fs.readFileSync(
    path.join(root, ".github/workflows/component-scope-binding.yml"),
    "utf8",
  );
  assert.match(workflow, /constructor-projection-compare\.ts/u);
  assert.doesNotMatch(workflow, /cmp "\$evidence\/base-constructor\.json"/u);
});

test("experiment remains default; malformed provenance still fails before commands", () => {
  assert.equal(scopeBindingMode(["/evidence"]).purpose, "experiment");
  assert.equal(scopeBindingMode(["/evidence", "--correctness"]).purpose, "correctness");
  assert.throws(() => scopeBindingMode(["/evidence", "--skip"]));
  assert.throws(() => auditScopeCompatibility("/not-read", {}));
  assert.throws(() =>
    auditScopeCompatibility("/not-read", {
      observerPurpose: "correctness",
      eventName: "pull_request",
      productBase: "invalid",
      productHead: "invalid",
    }),
  );
});
test("comparable and incomparable observations produce unqualified reports without normal clocks", () => {
  const comparisons = Array.from({ length: 96 }, () => ({ identical: true, sameInput: true }));
  const sourceDelta = { unclassified: "retained" },
    provenance = { source: "actual" },
    binaries = { base: "before", head: "after" };
  const equal = scopeCorrectnessReport(provenance, binaries, { comparisons, sourceDelta });
  assert.equal(equal.status, "NOT_APPLICABLE");
  assert.equal(equal.performanceQualification, false);
  assert.deepEqual(equal.normal, []);
  comparisons[0].identical = false;
  const report = scopeCorrectnessReport(provenance, binaries, { comparisons, sourceDelta });
  assert.equal(report.status, "INCOMPARABLE");
  assert.equal(report.qualification, false);
  assert.equal(report.sourceDelta, sourceDelta);
  assert.equal(report.provenance, provenance);
  assert.equal(report.binaries, binaries);
  assert.deepEqual(report.normal, []);
  assert.equal(Object.hasOwn(report, "baseMedianWallNs"), false);
  assert.equal(Object.hasOwn(report, "headMedianWallNs"), false);
  assert.throws(() => scopeObservation(comparisons.slice(1)));
  assert.equal(scopePacketEqual({ text: "whole", opaque: 1 }, { text: "whole", opaque: 2 }), false);
});
