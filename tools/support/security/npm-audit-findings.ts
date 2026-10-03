import assert from "node:assert/strict";
import {
  advisoryId as forgeId,
  assertCleanReport,
  assertRemediatedReport,
} from "./node-forge-proof.ts";
import { bracesAdvisoryId, record } from "./braces-lock.ts";

const levels = ["info", "low", "moderate", "high", "critical"];

function assertBracesFinding(advisory: Record<string, unknown>): void {
  assert.equal(advisory.github_advisory_id, bracesAdvisoryId);
  assert.equal(advisory.module_name, "braces");
  assert.equal(advisory.severity, "high");
  assert.equal(advisory.url, "https://github.com/advisories/" + bracesAdvisoryId);
  assert.equal(advisory.vulnerable_versions, "<=3.0.3");
  assert.equal(advisory.patched_versions, null);
  assert.ok(Array.isArray(advisory.findings) && advisory.findings.length > 0);
  for (const value of advisory.findings) {
    const finding = record(value);
    assert.equal(finding.version, "3.0.3");
    assert.equal(finding.dev, false);
    assert.equal(finding.bundled, false);
    assert.equal(typeof finding.optional, "boolean");
    assert.ok(Array.isArray(finding.paths) && finding.paths.length > 0);
    for (const trail of finding.paths)
      assert.ok(
        typeof trail === "string" && />(?:chokidar|micromatch)>braces$/.test(trail),
        "new Braces audit consumer",
      );
  }
}

// Account for the whole original producer report before forming the single
// Forge view consumed by its unchanged validator. The original report remains
// visible; these views never replace or remove printed audit findings.
export function assertSourceRemediatedReport(value: unknown, status: number): string[] {
  assert.ok(status === 0 || status === 1, "unexpected audit process status");
  const report = record(value);
  assert.deepEqual(Object.keys(report).sort(), ["advisories", "metadata"]);
  const metadata = record(report.metadata),
    counts = record(metadata.vulnerabilities),
    advisories = record(report.advisories);
  const printed: Record<string, number> = Object.fromEntries(levels.map((level) => [level, 0]));
  const severe: { key: string; advisory: Record<string, unknown> }[] = [];
  for (const [key, entry] of Object.entries(advisories)) {
    const advisory = record(entry);
    assert.ok(
      typeof advisory.severity === "string" && levels.includes(advisory.severity),
      "unknown severity",
    );
    printed[advisory.severity]++;
    if (levels.indexOf(advisory.severity) >= 2) severe.push({ key, advisory });
  }
  for (const level of levels) {
    assert.ok(Number.isSafeInteger(counts[level]) && Number(counts[level]) >= 0);
    if (levels.indexOf(level) >= 2)
      assert.equal(printed[level], counts[level], "missing actionable audit entry");
    else assert.ok(printed[level] <= Number(counts[level]), "inconsistent below-threshold count");
  }
  if (status === 0) {
    assertCleanReport(report);
    return [];
  }
  assert.deepEqual(
    severe
      .map(({ advisory }) => advisory.github_advisory_id)
      .sort((a, b) => String(a).localeCompare(String(b))),
    [forgeId, bracesAdvisoryId].sort(),
    "unreviewed moderate-or-higher set",
  );
  assert.equal(counts.high, 2);
  assert.equal(counts.moderate, 0);
  assert.equal(counts.critical, 0);
  const forge = severe.find(({ advisory }) => advisory.github_advisory_id === forgeId)!;
  const braces = severe.find(({ advisory }) => advisory.github_advisory_id === bracesAdvisoryId)!;
  assertBracesFinding(braces.advisory);
  const below = Object.fromEntries(
    Object.entries(advisories).filter(
      ([, value]) => levels.indexOf(String(record(value).severity)) < 2,
    ),
  );
  assertRemediatedReport({
    advisories: { ...below, [forge.key]: forge.advisory },
    metadata: { ...metadata, vulnerabilities: { ...counts, high: 1 } },
  });
  return severe.map(({ advisory }) => String(advisory.github_advisory_id)).sort();
}
