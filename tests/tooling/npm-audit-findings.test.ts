import assert from "node:assert/strict";
import test from "node:test";
import { assertSourceRemediatedReport } from "../../tools/support/security/npm-audit-findings.ts";
import { advisoryId } from "../../tools/support/security/node-forge-proof.ts";
import { bracesAdvisoryId } from "../../tools/support/security/braces-lock.ts";

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
      braces: entry(bracesAdvisoryId, "braces", "3.0.3", "nuxt>micromatch>braces"),
      low: { severity: "low", github_advisory_id: "visible-low" },
    },
    metadata: { vulnerabilities: { info: 0, low: 1, moderate: 0, high: 2, critical: 0 } },
  };
}
test("both independently source-remediated findings retain visible below-threshold entries", () => {
  const report = fixture(),
    before = structuredClone(report);
  assert.deepEqual(assertSourceRemediatedReport(report, 1), [advisoryId, bracesAdvisoryId].sort());
  assert.deepEqual(report, before);
  const clean = fixture();
  delete clean.advisories.forge;
  delete clean.advisories.braces;
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
      r.advisories.braces.github_advisory_id = "unknown";
    },
    (r: any) => {
      delete r.advisories.braces;
    },
    (r: any) => {
      r.metadata.vulnerabilities.high = 3;
    },
    (r: any) => {
      r.advisories.braces.severity = "critical";
      r.metadata.vulnerabilities.high = 1;
      r.metadata.vulnerabilities.critical = 1;
    },
    (r: any) => {
      r.advisories.braces.findings[0].version = "3.0.2";
    },
    (r: any) => {
      r.advisories.braces.findings[0].paths = ["new>braces"];
    },
    (r: any) => {
      r.advisories.forge.findings[0].paths = ["new>node-forge"];
    },
    (r: any) => {
      r.advisories.braces.patched_versions = ">=3.0.4";
    },
    (r: any) => {
      r.advisories.braces.findings[0].dev = true;
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
