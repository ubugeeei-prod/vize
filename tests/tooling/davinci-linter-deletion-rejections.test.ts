import assert from "node:assert/strict";
import { test } from "node:test";
import { canonicalSha256 } from "../differential/linter-deletion-schema.ts";
import {
  noFilesLinterStderr,
  unmatchedPatternsLinterStderr,
} from "../differential/linter-deletion-matrix.ts";
import {
  assessLinterDeletionShard,
  type LintProjectReport,
  validateLinterDeletionArtifact,
} from "../differential/linter-deletion-verifier.ts";
import { control } from "./_helpers/davinci-linter-deletion-fixture.ts";

void test("linter preflight rejects reduced, stale, stitched and forged evidence", () => {
  const missing = control();
  missing.projectArtifacts.delete("with-vue");
  assert.throws(() => assessLinterDeletionShard(missing), /linter project artifacts/);

  const missingRaw = control();
  missingRaw.rawLinterRuns.delete("with-vue");
  assert.throws(() => assessLinterDeletionShard(missingRaw), /raw Matrix linter artifacts/);

  assert.throws(
    () => assessLinterDeletionShard(control({ indexSourceRevision: "0".repeat(40) })),
    /source SHA drifted/,
  );
  const stitched = control();
  stitched.matrix.value.evidence.machine.arch = "arm64";
  assert.throws(() => assessLinterDeletionShard(stitched), /machine identity drifted/);

  const rawDrift = control();
  const firstRawResult = rawDrift.rawLinterRuns.get("with-vue")?.value.parsed[0];
  assert(firstRawResult);
  firstRawResult.file = "src/Changed.vue";
  assert.throws(() => assessLinterDeletionShard(rawDrift), /stdout and parsed output drifted/);

  for (const stderr of ["", "No .vue files found\n"]) {
    const zeroStderrDrift = control();
    zeroStderrDrift.rawLinterRuns.get("without-vue")!.value.stderr = stderr;
    assert.throws(() => assessLinterDeletionShard(zeroStderrDrift), /raw linter stderr drifted/);
  }
  const nonzeroStderr = control();
  nonzeroStderr.rawLinterRuns.get("with-vue")!.value.stderr =
    nonzeroStderr.rawLinterRuns.get("without-vue")!.value.stderr;
  assert.throws(() => assessLinterDeletionShard(nonzeroStderr), /raw linter stderr drifted/);

  for (const stderr of [
    "",
    unmatchedPatternsLinterStderr(["unknown/**/*.vue"]),
    unmatchedPatternsLinterStderr(["missing-a/**/*.vue", "missing-b/**/*.vue", "src/**/*.vue"]),
    unmatchedPatternsLinterStderr(["missing-b/**/*.vue", "missing-a/**/*.vue"]),
    unmatchedPatternsLinterStderr(["missing-a/**/*.vue", "missing-a/**/*.vue"]),
    'Warning: no .vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, or .tsx files found matching patterns: ["missing-a/**/*.vue","missing-b/**/*.vue"]\n',
  ]) {
    const warningDrift = control();
    warningDrift.registryProjects[0].vueGlobs = [
      "missing-a/**/*.vue",
      "missing-b/**/*.vue",
      "src/**/*.vue",
    ];
    warningDrift.rawLinterRuns.get("with-vue")!.value.stderr = stderr;
    assert.throws(() => assessLinterDeletionShard(warningDrift), /raw linter stderr drifted/);
  }

  const warningForZero = control();
  warningForZero.rawLinterRuns.get("without-vue")!.value.stderr = unmatchedPatternsLinterStderr([
    "**/*.vue",
  ]);
  assert.throws(() => assessLinterDeletionShard(warningForZero), /raw linter stderr drifted/);

  const wrongZeroPatterns = control();
  wrongZeroPatterns.rawLinterRuns.get("without-vue")!.value.stderr = noFilesLinterStderr([
    "src/**/*.vue",
  ]);
  assert.throws(() => assessLinterDeletionShard(wrongZeroPatterns), /raw linter stderr drifted/);

  const foreignParsedFile = control(),
    foreignRaw = foreignParsedFile.rawLinterRuns.get("with-vue")!.value,
    foreignReport = foreignParsedFile.projectArtifacts.get("with-vue")!.value;
  foreignRaw.parsed[0].file = "foreign/App.vue";
  foreignRaw.stdout = JSON.stringify(foreignRaw.parsed);
  (foreignReport.raw.patinaResults[0] as Record<string, unknown>).file = "foreign/App.vue";
  for (const finding of foreignReport.raw.patinaFindings as Array<Record<string, unknown>>)
    finding.file = "foreign/App.vue";
  foreignRaw.stderr = unmatchedPatternsLinterStderr(["src/**/*.vue"]);
  assert.throws(
    () => assessLinterDeletionShard(foreignParsedFile),
    /parsed file falls outside registry vueGlobs/,
  );

  const reportDrift = control();
  (
    reportDrift.projectArtifacts.get("with-vue")!.value.raw.patinaResults[0] as Record<
      string,
      unknown
    >
  ).file = "src/Other.vue";
  assert.throws(() => assessLinterDeletionShard(reportDrift), /Matrix and divergence runs drifted/);

  const stringSeverity = control(),
    severityRaw = stringSeverity.rawLinterRuns.get("with-vue")!.value,
    severityReport = stringSeverity.projectArtifacts.get("with-vue")!.value,
    rawMessage = (severityRaw.parsed[0].messages as Array<Record<string, unknown>>)[0],
    reportMessage = (
      (severityReport.raw.patinaResults[0] as Record<string, unknown>).messages as Array<
        Record<string, unknown>
      >
    )[0];
  rawMessage.severity = "2";
  reportMessage.severity = "2";
  (severityReport.raw.patinaFindings[0] as Record<string, unknown>).severity = "2";
  severityRaw.stdout = JSON.stringify(severityRaw.parsed);
  assert.throws(() => assessLinterDeletionShard(stringSeverity), /severity must be 1 or 2/);

  for (const vueGlobs of [
    [],
    [""],
    ["src/*.vue"],
    ["./src/**/*.vue"],
    ["../src/**/*.vue"],
    ["src/**/*.vue", "src/**/*.vue"],
  ]) {
    const registryDrift = control();
    registryDrift.registryProjects[0].vueGlobs = vueGlobs;
    assert.throws(() => assessLinterDeletionShard(registryDrift), /vueGlobs/);
  }

  const missingLinterCoverage = control();
  missingLinterCoverage.registryProjects[0].coverage = ["formatter"];
  assert.throws(
    () => assessLinterDeletionShard(missingLinterCoverage),
    /every Matrix project must carry linter coverage/,
  );

  const forgedReason = control({
    projects: { "with-vue": { unusableReason: "unreviewed excuse" } },
  });
  assert.throws(() => assessLinterDeletionShard(forgedReason), /budget reason drifted/);

  const forgedDigest = control();
  forgedDigest.projectArtifacts.get("with-vue")!.value.divergence.falsePositives.push({});
  assert.throws(() => assessLinterDeletionShard(forgedDigest), /count drifted|digest mismatch/);

  for (const [label, mutate] of [
    [
      "baseline",
      (report) => {
        report.baseline.version = "10.9.1";
      },
    ],
    [
      "runtime",
      (report) => {
        report.baseline.runtimeVersions["eslint-plugin-vue"] = "10.9.1";
      },
    ],
    [
      "divergence",
      (report) => {
        report.divergence.upstream.version = "10.9.1";
      },
    ],
  ] satisfies Array<[string, (report: LintProjectReport) => void]>) {
    const versionDrift = control(),
      report = versionDrift.projectArtifacts.get("with-vue")!.value;
    mutate(report);
    assert.throws(
      () => assessLinterDeletionShard(versionDrift),
      /baseline version drifted|baseline runtime version drifted|upstream identity drifted/,
      `${label} version must match the hashed rule map`,
    );
  }

  for (const field of ["mappedRuleCount", "skippedByPresetCount", "comparedRuleCount"] as const) {
    const coverageDrift = control(),
      report = coverageDrift.projectArtifacts.get("with-vue")!.value;
    report.baseline[field]++;
    assert.throws(
      () => assessLinterDeletionShard(coverageDrift),
      /rule count drifted|coverage-gap selection/,
      `${field} must be derived from the hashed rule map`,
    );
  }

  const missingDigest = control();
  missingDigest.ruleMap.sha256 = "";
  assert.throws(
    () => assessLinterDeletionShard(missingDigest),
    /linter input digests must be sha256/,
  );

  const openBuildReceipt = control();
  openBuildReceipt.buildReceipt.unversioned = true;
  assert.throws(() => assessLinterDeletionShard(openBuildReceipt), /fields drifted/);

  const artifact = assessLinterDeletionShard(control());
  artifact.projects[0].comparedFileCount++;
  assert.throws(() => validateLinterDeletionArtifact(artifact), /linter deletion totals drifted/);

  for (const mutate of [
    (candidate) => {
      candidate.unversioned = true;
    },
    (candidate) => {
      candidate.projects[0].unversioned = true;
    },
    (candidate) => {
      Object.assign(candidate.projects[0].budget, { unversioned: true });
    },
    (candidate) => {
      candidate.blockers[0].unversioned = true;
    },
  ] satisfies Array<(candidate: ReturnType<typeof assessLinterDeletionShard>) => void>) {
    const candidate = assessLinterDeletionShard(control());
    mutate(candidate);
    candidate.sha256 = canonicalSha256(
      Object.fromEntries(Object.entries(candidate).filter(([key]) => key !== "sha256")),
    );
    assert.throws(() => validateLinterDeletionArtifact(candidate), /fields drifted/);
  }

  const inconsistentBudget = assessLinterDeletionShard(control());
  inconsistentBudget.projects[0].budget.passed = false;
  inconsistentBudget.sha256 = canonicalSha256(
    Object.fromEntries(Object.entries(inconsistentBudget).filter(([key]) => key !== "sha256")),
  );
  assert.throws(
    () => validateLinterDeletionArtifact(inconsistentBudget),
    /projected budget passed drifted/,
  );

  const missingConfigBlocker = assessLinterDeletionShard(
    control({ projects: { "with-vue": { droppedConfigMessageCount: 1 } } }),
  );
  missingConfigBlocker.blockers = missingConfigBlocker.blockers.filter(
    (blocker) => blocker.code !== "linter-baseline-config",
  );
  missingConfigBlocker.sha256 = canonicalSha256(
    Object.fromEntries(Object.entries(missingConfigBlocker).filter(([key]) => key !== "sha256")),
  );
  assert.throws(
    () => validateLinterDeletionArtifact(missingConfigBlocker),
    /blocker projection drifted/,
  );
});
