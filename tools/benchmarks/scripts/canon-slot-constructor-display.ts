/** Exact observed successors for two helper-name diagnostic displays. */
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
