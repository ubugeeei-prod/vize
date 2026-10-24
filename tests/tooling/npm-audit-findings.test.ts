import assert from "node:assert/strict";
import test from "node:test";
import { assertSourceRemediatedReport } from "../../tools/support/security/npm-audit-findings.ts";
import { advisoryId } from "../../tools/support/security/node-forge-proof.ts";

function fixture(): any {
  const entry = (id: string, name: string, version: string, trail: string) => ({
    github_advisory_id: id,
    module_name: name,
    severity: "high",
    url: "https://github.com/advisories/" + id,
    vulnerable_versions: "<=" + version,
    patched_versions: null,
    findings: [{ version, dev: false, bundled: false, optional: false, paths: [trail] }],
  });
  return {
    advisories: {
      forge: entry(advisoryId, "node-forge", "1.4.0", "nuxt>listhen>node-forge"),
      low: { severity: "low", github_advisory_id: "visible-low" },
    },
    metadata: { vulnerabilities: { info: 0, low: 1, moderate: 0, high: 1, critical: 0 } },
  };
}
test("the source-remediated finding retains visible below-threshold entries", () => {
  const report = fixture(),
    before = structuredClone(report);
  assert.deepEqual(assertSourceRemediatedReport(report, 1), [advisoryId]);
  assert.deepEqual(report, before);
  const clean = fixture();
  delete clean.advisories.forge;
  clean.metadata.vulnerabilities.high = 0;
  assert.deepEqual(assertSourceRemediatedReport(clean, 0), []);
  assert.equal(clean.advisories.low.github_advisory_id, "visible-low");
});
test("unknown severe findings, missing entries, changed identity/path and process errors remain fatal", () => {
  const mutations = [
    (r: any) => {
      r.advisories.unknown = { severity: "moderate" };
      r.metadata.vulnerabilities.moderate = 1;
    },
    (r: any) => {
      r.advisories.forge.github_advisory_id = "unknown";
    },
    (r: any) => {
      delete r.advisories.forge;
    },
    (r: any) => {
      r.metadata.vulnerabilities.high = 2;
    },
    (r: any) => {
      r.advisories.forge.severity = "critical";
      r.metadata.vulnerabilities.high = 0;
      r.metadata.vulnerabilities.critical = 1;
    },
    (r: any) => {
      r.advisories.forge.findings[0].version = "1.3.0";
    },
    (r: any) => {
      r.advisories.forge.findings[0].paths = ["new>node-forge"];
    },
    (r: any) => {
      r.advisories.forge.patched_versions = ">=1.4.1";
    },
    (r: any) => {
      r.advisories.forge.findings[0].dev = true;
    },
    (r: any) => {
      r.metadata.vulnerabilities.low = 0;
    },
    (r: any) => {
      r.error = { code: "registry failure" };
    },
  ];
  for (const mutate of mutations) {
    const report = fixture();
    mutate(report);
    assert.throws(() => assertSourceRemediatedReport(report, 1));
  }
  assert.throws(() => assertSourceRemediatedReport(fixture(), 0));
  assert.throws(() => assertSourceRemediatedReport(fixture(), 2));
});
