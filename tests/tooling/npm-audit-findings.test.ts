import assert from "node:assert/strict";
import test from "node:test";
import { assertCleanAuditReport } from "../../tools/support/security/npm-audit-findings.ts";
import { advisoryId } from "../../tools/support/security/node-forge-proof.ts";

function fixture() {
  return {
    advisories: {
      low: { severity: "low", github_advisory_id: "visible-low" },
    } as Record<string, Record<string, unknown>>,
    metadata: { vulnerabilities: { info: 0, low: 1, moderate: 0, high: 0, critical: 0 } },
  };
}

test("a successful clean audit retains visible below-threshold entries", () => {
  const report = fixture();
  const before = structuredClone(report);
  assertCleanAuditReport(report, 0);
  assert.deepEqual(report, before);
});

test("even the former Forge exception and published repair metadata remain fatal", () => {
  for (const patched of [null, ">=1.4.1"]) {
    const report = fixture();
    report.advisories.forge = {
      github_advisory_id: advisoryId,
      module_name: "node-forge",
      severity: "high",
      url: "https://github.com/advisories/" + advisoryId,
      vulnerable_versions: "<=1.4.0",
      patched_versions: patched,
      findings: [
        {
          version: "1.4.0",
          dev: false,
          bundled: false,
          optional: false,
          paths: ["nuxt>listhen>node-forge"],
        },
      ],
    };
    report.metadata.vulnerabilities.high = 1;
    for (const status of [0, 1]) assert.throws(() => assertCleanAuditReport(report, status));
  }
});

test("unknown severe findings, missing entries, inconsistent counts and process errors remain fatal", () => {
  const mutations: ((report: ReturnType<typeof fixture>) => void)[] = [
    (report) => {
      report.advisories.unknown = { severity: "moderate" };
      report.metadata.vulnerabilities.moderate = 1;
    },
    (report) => {
      report.metadata.vulnerabilities.high = 1;
    },
    (report) => {
      report.advisories.unknown = { severity: "critical" };
      report.metadata.vulnerabilities.critical = 1;
    },
    (report) => {
      report.advisories.unknown = { severity: "unrecognized" };
    },
    (report) => {
      report.metadata.vulnerabilities.low = 0;
    },
    (report) => {
      report.metadata.vulnerabilities.high = -1;
    },
  ];
  for (const mutate of mutations) {
    const report = fixture();
    mutate(report);
    assert.throws(() => assertCleanAuditReport(report, 0));
  }
  for (const status of [1, 2, -1]) assert.throws(() => assertCleanAuditReport(fixture(), status));
  assert.throws(() => assertCleanAuditReport({ ...fixture(), error: "registry failure" }, 0));
  assert.throws(() => assertCleanAuditReport({ error: "registry failure" }, 0));
});
