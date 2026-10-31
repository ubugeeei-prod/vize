/** Exact observed helper displays and one independently verified props correction. */
import assert from "node:assert/strict";
import fs from "node:fs";
import { hash } from "./canon-instance-directives-io.ts";
import { isRecord, jsonValue } from "./canon-instance-directives-model.ts";
import type { SlotCase } from "./canon-slot-constructor-cases.ts";

const certificateBytes = fs.readFileSync(
  new URL(
    "../../../tests/_fixtures/differential/typechecker/zero-argument-slot-constructor-11013/display-rename-11153.json",
    import.meta.url,
  ),
);
assert.equal(
  hash(certificateBytes),
  "3098b2095848dd9ef4878b10d395efcad0748eb8016652dc77e9cddffacbf131",
  "the independently observed diagnostic display certificate must stay immutable",
);
const certificate = jsonValue(certificateBytes.toString("utf8"));
assert.ok(isRecord(certificate) && isRecord(certificate.cases));
const cases = certificate.cases;
const propsCorrectionBytes = fs.readFileSync(
  new URL(
    "../../../tests/_fixtures/differential/typechecker/constructor-props-correlation-11023/overload-last-zero-cli.json",
    import.meta.url,
  ),
);
assert.equal(
  hash(propsCorrectionBytes),
  "e06bbe5ff85792c7d3daaa1398e86648b3bebab3e9f542ae234a7ad0d6f09749",
  "the complete observed props-correction packets must stay immutable",
);
const propsCorrectionValue = jsonValue(propsCorrectionBytes.toString("utf8"));
assert.ok(isRecord(propsCorrectionValue));
const propsCorrection = propsCorrectionValue;

function observedCase(row: SlotCase) {
  const observed = cases[row.id];
  assert.ok(isRecord(observed), "only a certified case may change diagnostic display");
  assert.equal(hash(row.source), observed.sourceSha256);
  assert.equal(hash(row.component), observed.componentSha256);
  return observed;
}

export function assertAuthoredNativeDiagnostics(row: SlotCase, before: unknown, after: unknown) {
  if (row.id !== "missing-symbol") {
    assert.deepEqual(after, before, "every authored native diagnostic stays exact");
    return;
  }
  const observed = observedCase(row).native;
  assert.ok(isRecord(observed));
  assert.deepEqual(before, observed.base, "the complete published native diagnostic stays exact");
  assert.deepEqual(after, observed.head, "the complete renamed native diagnostic stays exact");
}

export function assertCliDiagnosticContract(row: SlotCase, before: unknown, after: unknown) {
  if (row.id === "overload-last-zero") {
    // This is an explicit semantic repair against the independent original
    // reference, not preservation of the published false authored error.
    assert.equal(row.id, propsCorrection.caseId);
    assert.equal(hash(row.source), propsCorrection.sourceSha256);
    assert.equal(hash(row.component), propsCorrection.componentSha256);
    assert.deepEqual(
      before,
      propsCorrection.before,
      "the whole published false-error packet stays exact",
    );
    assert.deepEqual(
      after,
      propsCorrection.after,
      "the whole corrected CLI packet must match the observed repair",
    );
    return;
  }
  if (row.id !== "unknown-prop") {
    assert.deepEqual(after, before, "whole public CLI diagnostics and project metadata stay exact");
    return;
  }
  // Shorter helper names change TypeScript's display truncation boundary too.
  // Compare both complete observed packets, including their exact truncation,
  // duplicated raw strings, source coordinates, counts and project metadata.
  const observed = observedCase(row).cli;
  assert.ok(isRecord(observed));
  assert.deepEqual(before, observed.base, "the complete published CLI packet stays exact");
  assert.deepEqual(after, observed.head, "the complete renamed CLI packet stays exact");
}
