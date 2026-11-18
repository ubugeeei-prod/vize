import assert from "node:assert/strict";
import { assertCleanReport } from "./node-forge-proof.ts";
import { picomatchAdvisoryId, picomatchVersions, record } from "./picomatch-lock.ts";

// Upstream Forge 1.4.3 removes the former source-backport exception. Account
// for the complete producer report and require a successful, clean audit.
export function assertCleanAuditReport(value: unknown, status: number): void {
  assert.equal(status, 0, "audit findings or process failure remain fatal");
  assertCleanReport(value);
}

// GHSA-fc5q-39xc-vjpf has no patched picomatch release. pnpm still reports the
// registry version, so a failing audit is recognized only when its complete
// actionable set is exactly that advisory on the two reviewed versions. The
// caller must then attest the installed patched source (picomatch-proof.ts).
export function assertPicomatchAuditReport(value: unknown): void {
  const report = record(value);
  assert.deepEqual(Object.keys(report).sort(), ["advisories", "metadata"]);
  const advisories = Object.values(record(report.advisories)).map(record);
  const actionable = advisories.filter((entry) =>
    ["moderate", "high", "critical"].includes(String(entry.severity)),
  );
  assert.equal(actionable.length, 1, "unexpected moderate-or-higher advisory set");
  const advisory = actionable[0];
  assert.equal(advisory.github_advisory_id, picomatchAdvisoryId);
  assert.equal(advisory.module_name, "picomatch");
  assert.equal(advisory.severity, "high");
  assert.equal(advisory.url, "https://github.com/advisories/" + picomatchAdvisoryId);
  assert.equal(advisory.vulnerable_versions, "<=4.0.7");
  assert.equal(advisory.patched_versions, null, "a published fix replaces this source patch");
  assert.ok(Array.isArray(advisory.findings) && advisory.findings.length > 0);
  for (const findingValue of advisory.findings) {
    const finding = record(findingValue);
    assert.ok(picomatchVersions.includes(finding.version as never), "unreviewed version");
    assert.equal(finding.dev, false);
    assert.equal(finding.bundled, false);
    assert.equal(typeof finding.optional, "boolean");
    assert.ok(Array.isArray(finding.paths) && finding.paths.length > 0);
    for (const trail of finding.paths)
      assert.ok(typeof trail === "string" && trail.endsWith(">picomatch"), "unexpected path");
  }
  const counts = record(record(report.metadata).vulnerabilities);
  assert.equal(counts.high, 1);
  assert.equal(counts.moderate, 0);
  assert.equal(counts.critical, 0);
  // The remaining count bookkeeping is the clean report's, minus this entry.
  const rest = structuredClone(report);
  const restAdvisories = record(rest.advisories);
  for (const [key, entry] of Object.entries(restAdvisories))
    if (record(entry).github_advisory_id === picomatchAdvisoryId) delete restAdvisories[key];
  record(record(rest.metadata).vulnerabilities).high = 0;
  assertCleanReport(rest);
}

// Returns the advisories the caller must back with an installed-source proof.
export function assertAuditReport(value: unknown, status: number): string[] {
  if (status === 0) {
    assertCleanAuditReport(value, status);
    return [];
  }
  assert.equal(status, 1, "audit process failure remains fatal");
  assertPicomatchAuditReport(value);
  return [picomatchAdvisoryId];
}
