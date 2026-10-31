import assert from "node:assert/strict";
import { assertCleanReport } from "./node-forge-proof.ts";

// Upstream Forge 1.4.1 removes the former source-backport exception. Account
// for the complete producer report and require a successful, clean audit.
export function assertCleanAuditReport(value: unknown, status: number): void {
  assert.equal(status, 0, "audit findings or process failure remain fatal");
  assertCleanReport(value);
}
