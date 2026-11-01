// Pure complete-array assertion, shared by replay and adversarial order controls.
import assert from "node:assert/strict";

export function assertWholeNumberArray(actual: unknown, expected: unknown, label: string) {
  assert(Array.isArray(actual) && Array.isArray(expected), label + ": arrays required");
  assert.deepEqual(actual, expected, label);
  assert.equal(JSON.stringify(actual), JSON.stringify(expected), label + ": all fields/order");
}
