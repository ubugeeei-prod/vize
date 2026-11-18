import assert from "node:assert/strict";
import test from "node:test";
import {
  assertAuditReport,
  assertCleanAuditReport,
} from "../../tools/support/security/npm-audit-findings.ts";
import { picomatchAdvisoryId } from "../../tools/support/security/picomatch-lock.ts";
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

function picomatchReport() {
  const report = fixture();
  report.advisories.picomatch = {
    github_advisory_id: picomatchAdvisoryId,
    module_name: "picomatch",
    severity: "high",
    url: "https://github.com/advisories/" + picomatchAdvisoryId,
    vulnerable_versions: "<=4.0.7",
    patched_versions: null,
    findings: [
      {
        version: "2.3.2",
        dev: false,
        bundled: false,
        optional: false,
        paths: ["npm__framework__nuxt>nuxt>@nuxt/devtools>unstorage>anymatch>picomatch"],
      },
      {
        version: "4.0.7",
        dev: false,
        bundled: false,
        optional: false,
        paths: ["npm__builder__unplugin>unplugin>picomatch"],
      },
    ],
  };
  report.metadata.vulnerabilities.high = 1;
  return report;
}

test("the unpatched-upstream picomatch finding is recognized only on a failing audit", () => {
  const report = picomatchReport();
  const before = structuredClone(report);
  assert.deepEqual(assertAuditReport(report, 1), [picomatchAdvisoryId]);
  assert.deepEqual(report, before);
  assert.deepEqual(assertAuditReport(fixture(), 0), []);
  assert.throws(() => assertAuditReport(report, 0));
  assert.throws(() => assertCleanAuditReport(report, 1));
  for (const status of [2, -1]) assert.throws(() => assertAuditReport(report, status));
});

test("picomatch recognition rejects changed identity, versions, paths, counts and extra findings", () => {
  const mutations: ((report: ReturnType<typeof picomatchReport>) => void)[] = [
    (report) => (report.advisories.picomatch.patched_versions = ">=4.0.8"),
    (report) => (report.advisories.picomatch.vulnerable_versions = "<=4.0.8"),
    (report) => (report.advisories.picomatch.github_advisory_id = "GHSA-0000-0000-0000"),
    (report) => (report.advisories.picomatch.module_name = "micromatch"),
    (report) => (report.advisories.picomatch.severity = "critical"),
    (report) => ((report.advisories.picomatch.findings as any)[1].version = "4.0.6"),
    (report) => ((report.advisories.picomatch.findings as any)[1].dev = true),
    (report) => ((report.advisories.picomatch.findings as any)[1].bundled = true),
    (report) => ((report.advisories.picomatch.findings as any)[1].paths = ["x>micromatch"]),
    (report) => (report.advisories.picomatch.findings = []),
    (report) => (report.metadata.vulnerabilities.high = 2),
    (report) => {
      report.advisories.other = { severity: "moderate", github_advisory_id: "other" };
      report.metadata.vulnerabilities.moderate = 1;
    },
    (report) => (report.metadata.vulnerabilities.low = 0),
    (report) => {
      delete report.advisories.picomatch;
      report.metadata.vulnerabilities.high = 0;
    },
  ];
  for (const mutate of mutations) {
    const report = picomatchReport();
    mutate(report);
    assert.throws(() => assertAuditReport(report, 1), String(mutate));
  }
  assert.throws(() => assertAuditReport({ ...picomatchReport(), error: "registry failure" }, 1));
});
