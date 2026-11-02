import assert from "node:assert/strict";
import test from "node:test";
import {
  auditBindingViewCorrectness,
  bindingViewMode,
  bindingViewObservation,
  bindingViewCorrectnessReport,
  sameBindingPackets,
} from "../../tools/benchmarks/scripts/component-binding-view-eligibility.ts";
test("historical experiment remains default; malformed purpose/provenance still fails", () => {
  assert.equal(bindingViewMode(["/evidence"]).purpose, "experiment");
  assert.equal(bindingViewMode(["/evidence", "--experiment"]).purpose, "experiment");
  assert.equal(bindingViewMode(["/evidence", "--correctness"]).purpose, "correctness");
  assert.throws(() => bindingViewMode(["/evidence", "--skip"]));
  assert.throws(() => auditBindingViewCorrectness("/not-read", {}));
  assert.throws(() =>
    auditBindingViewCorrectness("/not-read", {
      observerPurpose: "correctness",
      eventName: "pull_request",
      productBase: "invalid",
      productHead: "invalid",
    }),
  );
});
test("ordinary reports retain unknowns and never qualify timing or gain", () => {
  const comparisons = Array.from({ length: 72 }, () => ({
    identical: true,
    sameInput: true,
    knownInput: true,
  }));
  const sourceDelta = { unclassified: "retained" };
  const equal = bindingViewCorrectnessReport({}, {}, { comparisons, sourceDelta });
  assert.equal(equal.status, "NOT_APPLICABLE");
  assert.equal(equal.performanceQualification, false);
  assert.deepEqual(equal.normal, []);
  for (const field of ["identical", "sameInput", "knownInput"] as const) {
    const rows = structuredClone(comparisons);
    rows[0][field] = false;
    const report = bindingViewCorrectnessReport({}, {}, { comparisons: rows, sourceDelta });
    assert.equal(report.status, "INCOMPARABLE");
    assert.equal(report.qualification, false);
    assert.equal(report.sourceDelta, sourceDelta);
    assert.deepEqual(report.normal, []);
    assert.equal(Object.hasOwn(report, "baseMedianWallNs"), false);
  }
  assert.throws(() => bindingViewObservation(comparisons.slice(1)));
  assert.equal(
    sameBindingPackets({ text: "whole", opaque: 1 }, { text: "whole", opaque: 2 }),
    false,
  );
});
