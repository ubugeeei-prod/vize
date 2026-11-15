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
    }
  >
> = {
  "bare-slot-shorthand": {
    ruleId: "vue/v-slot-style",
    sourceSha256: "4b7c77d382e1d3e46eeb6765eb6ab6cfaebb7c1d2b7a2a65a06871289b734c6e",
    packetSha256: "c9ecb9b4792dd220f61315e47a3ea407d0a13e0ca8fe48c933d00a562bee55c1",
    nativePacketSha256: "d61b27a83593bdd6f5ed078f954af6e908ed2ceecf81ae11fa268e37ae270dd9",
  },
  "class-decorator-prop": {
    ruleId: "vue/no-mutating-props",
    sourceSha256: "0e8ca44978bf438cf9ff9df7bb2adf315af226efc32b511d12a10c75dc22f6e1",
    packetSha256: "2f957e4d7659b623802a5e518ff5cf6b004f90f58759e7cf02981d9f56b9e083",
    nativePacketSha256: "208dbdcd5ae68be6b45d5eda85c39ff0e77b9e528dd50274659f50237094c3a0",
  },
};

function without(fields: Fields, keys: readonly string[]): Fields {
  return Object.fromEntries(Object.entries(fields).filter(([key]) => !keys.includes(key)));
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
    assert.equal(helpers.object(coverage.runtimeVersions)["eslint-plugin-vue"], "10.9.2");
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
  assert.equal(
    helpers.object(actual.budget).falsePositivePassed,
    actual.divergence.falsePositives.length === 0,
  );
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
  let reviewedCoverageIdentityChanged = false;
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
    reviewedCoverageIdentityChanged ||= changed;
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
    actual.files[markdownName] = actualMarkdown.replace(actualLine, expectedLine);
    if (changed) {
      actualArtifact.divergence = structuredClone(expectedArtifact.divergence);
      actualArtifact.budget = structuredClone(expectedArtifact.budget);
      actual.files[markdownName] = expectedMarkdown;
    }
  }
  if (reviewedCoverageIdentityChanged) {
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
    actual.files["lint-divergence-summary.json"] = structuredClone(expectedIndex);
    actual.status = expected.status;
    actual.stderr = expected.stderr;
  }
}
