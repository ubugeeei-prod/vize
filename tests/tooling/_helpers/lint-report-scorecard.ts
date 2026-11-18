import assert from "node:assert/strict";
import type { Result } from "./lint-report-rust-parity.ts";
import type { Divergence, Fields, ProjectReport } from "./lint-report-schema.ts";

interface ScorecardHelpers {
  assertDigest(divergence: Divergence): void;
  canonical(value: unknown): string;
  markdown(result: Result, name: string): string;
  object(value: unknown): Fields;
  runtimeVersions(provider: string): Record<string, string>;
}

const reviewedScorecardTransitions: Readonly<
  Record<
    string,
    {
      readonly ruleId: string;
      readonly sourceSha256: string;
      readonly packetSha256: string;
      readonly nativePacketSha256: string;
      readonly runtimeVersion: string;
    }
  >
> = {
  "bare-slot-shorthand": {
    ruleId: "vue/v-slot-style",
    sourceSha256: "4b7c77d382e1d3e46eeb6765eb6ab6cfaebb7c1d2b7a2a65a06871289b734c6e",
    packetSha256: "c9ecb9b4792dd220f61315e47a3ea407d0a13e0ca8fe48c933d00a562bee55c1",
    nativePacketSha256: "d61b27a83593bdd6f5ed078f954af6e908ed2ceecf81ae11fa268e37ae270dd9",
    runtimeVersion: "10.9.2",
  },
  "class-decorator-prop": {
    ruleId: "vue/no-mutating-props",
    sourceSha256: "0e8ca44978bf438cf9ff9df7bb2adf315af226efc32b511d12a10c75dc22f6e1",
    packetSha256: "2f957e4d7659b623802a5e518ff5cf6b004f90f58759e7cf02981d9f56b9e083",
    nativePacketSha256: "208dbdcd5ae68be6b45d5eda85c39ff0e77b9e528dd50274659f50237094c3a0",
    runtimeVersion: "10.9.2",
  },
  "bare-slot-shorthand-scorecard": {
    ruleId: "vue/v-slot-style",
    sourceSha256: "4b7c77d382e1d3e46eeb6765eb6ab6cfaebb7c1d2b7a2a65a06871289b734c6e",
    packetSha256: "47819ba2c93bbf1cc4d131054bca29a9ef2dfa18fee6c4f15f9a0d4f5417b362",
    nativePacketSha256: "8873508e48adc3b555e20239e65ce3c8e513b9f8eea8026490daa2eb2e1ecae6",
    runtimeVersion: "10.12.0",
  },
  "class-decorator-prop-scorecard": {
    ruleId: "vue/no-mutating-props",
    sourceSha256: "0e8ca44978bf438cf9ff9df7bb2adf315af226efc32b511d12a10c75dc22f6e1",
    packetSha256: "b5d67aac101eef5eaded5f2b4920f68451eadc479ebf7605a12eaa0c40c38750",
    nativePacketSha256: "fd06e9e496c71c5578f79449a8a9668c04f5d4fdb078c9082a0ca4de986cba93",
    runtimeVersion: "10.12.0",
  },
};

function without(fields: Fields, keys: readonly string[]): Fields {
  return Object.fromEntries(Object.entries(fields).filter(([key]) => !keys.includes(key)));
}

function json(value: unknown): string {
  const rendered = JSON.stringify(value);
  assert.notEqual(rendered, undefined);
  return rendered as string;
}

function markdownRatio(value: number): string {
  if (Object.is(value, -0)) return "-0.0";
  const magnitude = Math.abs(value);
  if (magnitude !== 0 && (magnitude < 0.00001 || magnitude >= 1e16)) return value.toExponential();
  return Number.isInteger(value) ? `${value}.0` : String(value);
}

function markdownRuleTable(title: string, findings: Fields[]): string {
  if (findings.length === 0) return `### ${title}: none\n\n`;
  const counts = new Map<string, number>();
  for (const finding of findings) {
    const key = String(finding.upstreamRuleId ?? finding.ruleId);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  const rows = [...counts].sort(
    (left, right) => right[1] - left[1] || left[0].localeCompare(right[0]),
  );
  return `${[
    `### ${title}: ${findings.length}`,
    "",
    "| Rule | Findings |",
    "| --- | ---: |",
    ...rows.map(([rule, count]) => `| \`${rule}\` | ${count} |`),
    "",
  ].join("\n")}\n`;
}

function normalizeReviewedMarkdown(
  markdown: string,
  actual: ProjectReport,
  expected: ProjectReport,
): string {
  const actualSummary = actual.divergence.summary,
    expectedSummary = expected.divergence.summary,
    actualBudget = actual.budget,
    expectedBudget = expected.budget;
  const replacements: Array<[string, string, string]> = [
    [
      `False positives: ${json(actualSummary.falsePositiveCount)} (${markdownRatio(actualSummary.falsePositiveRatio)})`,
      `False positives: ${json(expectedSummary.falsePositiveCount)} (${markdownRatio(expectedSummary.falsePositiveRatio)})`,
      "false-positive summary",
    ],
    [
      `Patina findings outside reference coverage: ${json(actualSummary.patinaOutsideBaselineSurfaceCount)}`,
      `Patina findings outside reference coverage: ${json(expectedSummary.patinaOutsideBaselineSurfaceCount)}`,
      "outside-surface summary",
    ],
    [
      `Reference coverage integrity errors: ${json(actualSummary.referenceCoverageIntegrityErrorCount)}`,
      `Reference coverage integrity errors: ${json(expectedSummary.referenceCoverageIntegrityErrorCount)}`,
      "reference-coverage summary",
    ],
    [
      `Budget verdict: ${String(actualBudget.verdict)}`,
      `Budget verdict: ${String(expectedBudget.verdict)}`,
      "budget verdict",
    ],
    [
      `Budget passed: ${actualBudget.passed === true}`,
      `Budget passed: ${expectedBudget.passed === true}`,
      "budget pass",
    ],
    [
      markdownRuleTable("False positives", actual.divergence.falsePositives),
      markdownRuleTable("False positives", expected.divergence.falsePositives),
      "false-positive table",
    ],
    [
      markdownRuleTable(
        "Patina findings outside reference coverage",
        actual.divergence.patinaOutsideBaselineSurface,
      ),
      markdownRuleTable(
        "Patina findings outside reference coverage",
        expected.divergence.patinaOutsideBaselineSurface,
      ),
      "outside-surface table",
    ],
  ];
  for (const [actualFragment, expectedFragment, label] of replacements) {
    if (actualFragment === expectedFragment) continue;
    assert.equal(markdown.split(actualFragment).length, 2, `exact reviewed Markdown ${label}`);
    markdown = markdown.replace(actualFragment, expectedFragment);
  }
  return markdown;
}

function expectedIndexBudget(projects: Array<{ project: string; budget: Fields }>): Fields {
  const failed = projects.filter(({ budget }) => budget.passed !== true);
  return {
    status: failed.length === 0 ? "success" : "failure",
    passed: failed.length === 0,
    projectCount: projects.length,
    passedCount: projects.length - failed.length,
    failedCount: failed.length,
    unusableCount: failed.filter(({ budget }) => budget.verdict === "unusable").length,
    breachedCount: failed.filter(({ budget }) => budget.verdict === "breached").length,
    failedProjects: failed.map(({ project }) => project),
  };
}

function budgetFailure(project: string, budget: Fields, summary: Fields): string {
  if (budget.verdict === "unusable")
    return `Lint divergence baseline is unusable for ${project}: ${String(budget.unusableReason)}`;
  const breaches: string[] = [];
  if (budget.falsePositivePassed !== true)
    breaches.push(
      `${String(summary.falsePositiveCount)} false positives exceed maxFalsePositiveCount 0`,
    );
  if (budget.falseNegativePassed !== true)
    breaches.push(
      `${String(summary.falseNegativeCount)} false negatives exceed maxFalseNegativeCount 0`,
    );
  return `Lint divergence budget breached for ${project}: ${breaches.join("; ")}`;
}

function normalizeReviewedCoverageIdentity(
  expected: ProjectReport,
  actual: ProjectReport,
  helpers: ScorecardHelpers,
): boolean {
  const expectedExpectations = expected.divergence.referenceCoverageExpectations,
    actualExpectations = actual.divergence.referenceCoverageExpectations;
  if (expectedExpectations.length === 0 || actualExpectations.length !== 0) return false;
  assert.equal(expectedExpectations.length, 1);
  const expectation = helpers.object(expectedExpectations[0]),
    caseId = String(expectation.case);
  assert.ok(
    Object.hasOwn(reviewedScorecardTransitions, caseId),
    `unexpected scorecard reference-coverage transition ${caseId}`,
  );
  const transition = reviewedScorecardTransitions[caseId];
  assert.equal(helpers.object(expectation.expectedFinding).ruleId, transition.ruleId);
  assert.equal(expectation.sourceSha256, transition.sourceSha256);
  assert.deepEqual(actual.raw, expected.raw, "scorecard must preserve every raw lint packet");
  helpers.assertDigest(expected.divergence);
  helpers.assertDigest(actual.divergence);
  assert.deepEqual(
    without(actual.divergence, [
      "falsePositives",
      "patinaOutsideBaselineSurface",
      "referenceCoverageExpectations",
      "sha256",
      "summary",
    ]),
    without(expected.divergence, [
      "falsePositives",
      "patinaOutsideBaselineSurface",
      "referenceCoverageExpectations",
      "sha256",
      "summary",
    ]),
  );
  const reviewed = expected.divergence.patinaOutsideBaselineSurface.map((entry) => {
    const { referenceCoverage, ...finding } = entry;
    const coverage = helpers.object(referenceCoverage);
    assert.equal(coverage.case, caseId);
    assert.equal(coverage.sourceSha256, transition.sourceSha256);
    assert.equal(coverage.packetSha256, transition.packetSha256);
    assert.equal(coverage.nativePacketSha256, transition.nativePacketSha256);
    assert.equal(
      helpers.object(coverage.runtimeVersions)["eslint-plugin-vue"],
      transition.runtimeVersion,
    );
    assert.equal(finding.ruleId, transition.ruleId);
    return finding;
  });
  const sorted = (entries: Fields[]) =>
    structuredClone(entries).sort((left, right) =>
      helpers.canonical(left).localeCompare(helpers.canonical(right)),
    );
  assert.deepEqual(
    sorted(actual.divergence.falsePositives),
    sorted([...expected.divergence.falsePositives, ...reviewed]),
  );
  assert.deepEqual(actual.divergence.patinaOutsideBaselineSurface, []);
  assert.deepEqual(
    without(actual.divergence.summary, [
      "falsePositiveCount",
      "falsePositiveRatio",
      "patinaOutsideBaselineSurfaceCount",
      "referenceCoverageIntegrityErrorCount",
    ]),
    without(expected.divergence.summary, [
      "falsePositiveCount",
      "falsePositiveRatio",
      "patinaOutsideBaselineSurfaceCount",
      "referenceCoverageIntegrityErrorCount",
    ]),
  );
  assert.equal(
    actual.divergence.summary.falsePositiveCount,
    actual.divergence.falsePositives.length,
  );
  const falsePositiveDenominator =
    actual.divergence.summary.sharedCount +
    actual.divergence.summary.falsePositiveCount +
    actual.divergence.summary.ruleLocationDivergenceCount;
  assert.equal(
    actual.divergence.summary.falsePositiveRatio,
    falsePositiveDenominator === 0
      ? 0
      : actual.divergence.summary.falsePositiveCount / falsePositiveDenominator,
  );
  assert.equal(actual.divergence.summary.patinaOutsideBaselineSurfaceCount, 0);
  assert.equal(actual.divergence.summary.referenceCoverageIntegrityErrorCount, 0);
  assert.deepEqual(
    without(helpers.object(actual.budget), [
      "falsePositivePassed",
      "passed",
      "unusableReason",
      "verdict",
    ]),
    without(helpers.object(expected.budget), [
      "falsePositivePassed",
      "passed",
      "unusableReason",
      "verdict",
    ]),
  );
  const actualBudget = helpers.object(actual.budget);
  const falsePositivePassed = actual.divergence.summary.falsePositiveCount === 0,
    falseNegativePassed = actual.divergence.summary.falseNegativeCount === 0,
    unusableReason = actualBudget.unusableReason,
    verdict =
      unusableReason !== null
        ? "unusable"
        : falsePositivePassed && falseNegativePassed
          ? "passed"
          : "breached";
  assert.equal(actualBudget.maxFalsePositiveCount, 0);
  assert.equal(actualBudget.maxFalseNegativeCount, 0);
  assert.equal(actualBudget.falsePositivePassed, falsePositivePassed);
  assert.equal(actualBudget.falseNegativePassed, falseNegativePassed);
  assert.equal(actualBudget.unusableReason, null);
  assert.equal(actualBudget.verdict, verdict);
  assert.equal(actualBudget.passed, verdict === "passed");
  return true;
}

export function assertScorecardIdentityTransition(
  expected: Result,
  actual: Result,
  helpers: ScorecardHelpers,
): void {
  const historicalVersions = helpers.runtimeVersions("eslint-plugin-vue"),
    scorecardVersions = helpers.runtimeVersions("eslint-plugin-vue-scorecard"),
    historicalVersion = historicalVersions["eslint-plugin-vue"],
    scorecardVersion = scorecardVersions["eslint-plugin-vue"];
  assert.equal(historicalVersion, "10.9.2");
  assert.equal(scorecardVersion, "10.12.0");
  const reviewedTransitions = new Map<
    string,
    {
      actualBudget: Fields;
      actualSummary: Fields;
      expectedBudget: Fields;
      expectedMarkdown: string;
      expectedSummary: Fields;
      file: string;
      markdownName: string;
      normalizedActualMarkdown: string;
    }
  >();
  for (const [file, actualArtifact] of Object.entries(actual.files)) {
    if (
      !file.endsWith("-lint-divergence.json") ||
      typeof actualArtifact !== "object" ||
      actualArtifact.schema !== "vize.fixtureLintDivergenceRun"
    )
      continue;
    const expectedArtifact = expected.files[file];
    assert.ok(
      typeof expectedArtifact === "object" &&
        expectedArtifact.schema === "vize.fixtureLintDivergenceRun",
    );
    const changed =
      "raw" in expectedArtifact && "raw" in actualArtifact
        ? normalizeReviewedCoverageIdentity(expectedArtifact, actualArtifact, helpers)
        : false;
    if (actualArtifact.baseline.comparedRuleCount === 0) {
      assert.equal(actualArtifact.baseline.version, expectedArtifact.baseline.version);
      assert.deepEqual(actualArtifact.baseline.runtimeVersions, {});
      continue;
    }
    assert.equal(expectedArtifact.baseline.version, historicalVersion);
    assert.equal(actualArtifact.baseline.version, scorecardVersion);
    assert.deepEqual(actualArtifact.baseline.runtimeVersions, scorecardVersions);
    if (expectedArtifact.baseline.runtimeVersions !== undefined)
      assert.deepEqual(expectedArtifact.baseline.runtimeVersions, historicalVersions);
    actualArtifact.baseline.version = expectedArtifact.baseline.version;
    actualArtifact.baseline.runtimeVersions = historicalVersions;

    const markdownName = file.replace(/\.json$/u, ".md");
    const expectedLine: string = `Baseline: eslint-plugin-vue ${historicalVersion}`;
    const actualLine: string = `Baseline: eslint-plugin-vue ${scorecardVersion}`;
    const expectedMarkdown = helpers.markdown(expected, markdownName);
    const actualMarkdown = helpers.markdown(actual, markdownName);
    assert.equal(expectedMarkdown.split(expectedLine).length, 2);
    assert.equal(actualMarkdown.split(actualLine).length, 2);
    const normalizedActualMarkdown = actualMarkdown.replace(actualLine, expectedLine);
    actual.files[markdownName] = normalizedActualMarkdown;
    if (changed) {
      const project = file.replace(/-lint-divergence\.json$/u, "");
      assert.equal(reviewedTransitions.has(project), false);
      reviewedTransitions.set(project, {
        actualBudget: structuredClone(helpers.object(actualArtifact.budget)),
        actualSummary: structuredClone(helpers.object(actualArtifact.divergence.summary)),
        expectedBudget: structuredClone(helpers.object(expectedArtifact.budget)),
        expectedMarkdown,
        expectedSummary: structuredClone(helpers.object(expectedArtifact.divergence.summary)),
        file,
        markdownName,
        normalizedActualMarkdown,
      });
    }
  }
  if (reviewedTransitions.size > 0) {
    const expectedIndex = expected.files["lint-divergence-summary.json"],
      actualIndex = actual.files["lint-divergence-summary.json"];
    assert.ok(
      typeof expectedIndex === "object" &&
        expectedIndex.schema === "vize.fixtureLintDivergenceIndex",
    );
    assert.ok(
      typeof actualIndex === "object" && actualIndex.schema === "vize.fixtureLintDivergenceIndex",
    );
    assert.equal(actualIndex.projects.length, expectedIndex.projects.length);
    assert.deepEqual(
      actualIndex.projects.map((project) => project.project),
      expectedIndex.projects.map((project) => project.project),
    );
    const projectEvidence = (
      result: Result,
      project: string,
      transitionSide: "actual" | "expected",
    ) => {
      const transition = reviewedTransitions.get(project);
      if (transition)
        return {
          budget: transitionSide === "actual" ? transition.actualBudget : transition.expectedBudget,
          summary:
            transitionSide === "actual" ? transition.actualSummary : transition.expectedSummary,
        };
      const artifact = result.files[`${project}-lint-divergence.json`];
      assert.ok(
        typeof artifact === "object" && artifact.schema === "vize.fixtureLintDivergenceRun",
      );
      return {
        budget: helpers.object(artifact.budget),
        summary: helpers.object(artifact.divergence.summary),
      };
    };
    const actualProjects = actualIndex.projects.map((row) => {
      const project = String(row.project),
        evidence = projectEvidence(actual, project, "actual");
      assert.deepEqual(row, { project, ...evidence.summary });
      if (!reviewedTransitions.has(project)) {
        const expectedRow = expectedIndex.projects.find(
          (candidate) => String(candidate.project) === project,
        );
        assert.deepEqual(row, expectedRow, `${project}: unrelated index row changed`);
      }
      return { project, budget: evidence.budget };
    });
    const expectedProjects = expectedIndex.projects.map((row) => {
      const project = String(row.project),
        evidence = projectEvidence(expected, project, "expected");
      assert.deepEqual(row, { project, ...evidence.summary });
      return { project, budget: evidence.budget };
    });
    const actualTotals = helpers.object(actualIndex.totals),
      expectedTotals = helpers.object(expectedIndex.totals);
    assert.deepEqual(Object.keys(actualTotals), Object.keys(expectedTotals));
    for (const key of Object.keys(actualTotals)) {
      assert.equal(
        actualTotals[key],
        actualIndex.projects.reduce((sum, row) => sum + Number(row[key] ?? 0), 0),
        `actual index total drifted: ${key}`,
      );
      assert.equal(
        expectedTotals[key],
        expectedIndex.projects.reduce((sum, row) => sum + Number(row[key] ?? 0), 0),
        `expected index total drifted: ${key}`,
      );
    }
    assert.deepEqual(helpers.object(actualIndex.budget), expectedIndexBudget(actualProjects));
    assert.deepEqual(helpers.object(expectedIndex.budget), expectedIndexBudget(expectedProjects));
    const failed = actualIndex.projects
      .map((row) => {
        const project = String(row.project),
          evidence = projectEvidence(actual, project, "actual");
        return { project, ...evidence };
      })
      .filter(({ budget }) => budget.passed !== true);
    assert.equal(actual.status, failed.length === 0 ? 0 : 1);
    assert.equal(
      actual.stderr,
      failed.length === 0
        ? ""
        : `Lint divergence budget failed for ${failed.length} project(s):\n${failed
            .map(({ project, budget, summary }) => budgetFailure(project, budget, summary))
            .join("\n")}\n`,
    );
    for (const [project, transition] of reviewedTransitions) {
      const row = actualIndex.projects.find((candidate) => String(candidate.project) === project),
        expectedRow = expectedIndex.projects.find(
          (candidate) => String(candidate.project) === project,
        );
      assert.ok(row && expectedRow);
      for (const key of Object.keys(row)) delete row[key];
      Object.assign(row, structuredClone(expectedRow));
      const actualArtifact = actual.files[transition.file],
        expectedArtifact = expected.files[transition.file];
      assert.ok(
        typeof actualArtifact === "object" &&
          actualArtifact.schema === "vize.fixtureLintDivergenceRun" &&
          "raw" in actualArtifact &&
          typeof expectedArtifact === "object" &&
          expectedArtifact.schema === "vize.fixtureLintDivergenceRun" &&
          "raw" in expectedArtifact,
      );
      assert.equal(
        normalizeReviewedMarkdown(
          transition.normalizedActualMarkdown,
          actualArtifact,
          expectedArtifact,
        ),
        transition.expectedMarkdown,
      );
      actualArtifact.divergence = structuredClone(expectedArtifact.divergence);
      actualArtifact.budget = structuredClone(expectedArtifact.budget);
      actual.files[transition.markdownName] = transition.expectedMarkdown;
    }
    for (const key of Object.keys(actualTotals)) actualTotals[key] = expectedTotals[key];
    actualIndex.budget = structuredClone(expectedIndex.budget);
    actual.status = expected.status;
    actual.stderr = expected.stderr;
  }
}
