import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import type { Result } from "./lint-report-rust-parity.ts";
export const root = path.resolve(import.meta.dirname, "../../..");
export const command = "tools/commands/fixtures/lint-divergence-report";
export const contractDirectory = "tests/_fixtures/differential/linter/reference-coverage-8849";
export const originalRef = "49545646fde08ade1000c4a33b38c9dd1ac4fb99";
export const updatedRef = "7421901d05ac2e252636dd8ee426e8783255378d";
export type Oracle = "original" | "updated";
interface Source {
  ref: string;
  files: [string, string, number][];
}
export const referenceFixtures = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../_fixtures/differential/linter/reference-coverage-8849/report-references.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as Record<Oracle, Source> & {
  fakeVizeSource: string;
  invalidArguments: string[][];
  parserVectors: { token: string; reason: string; js: string }[];
  missingModernPaths: string[][];
  seams: Record<string, unknown>;
  membershipDelta: {
    fields: Record<string, { before: Record<string, unknown>; after: Record<string, unknown> }>;
    markdown: [string, string][];
    stderr: string;
  };
  selectionCases: {
    id: string;
    globs: string[];
    files: Record<string, string>;
    expected: string[];
    gitRoot?: string;
  }[];
};
assert.equal(referenceFixtures.original.ref, originalRef);
assert.equal(referenceFixtures.updated.ref, updatedRef);
type Fields = Record<string, unknown>;
interface Finding extends Fields {
  file: string;
  line: number | bigint;
}
interface Raw {
  eslintResults: Fields[];
  patinaFindings: Finding[];
  patinaResults: (Fields & { file: string; messages: Fields[] })[];
}
interface Divergence extends Fields {
  schema: string;
  version: number;
  sha256: string;
  summary: Record<string, number>;
  referenceCoverageExpectations: { observedCount: number }[];
  falsePositives: Finding[];
  falseNegatives: (Finding & { upstreamRuleId: string })[];
  intentionalDivergences: (Finding & { reason: string })[];
  patinaOutsideBaselineSurface: Finding[];
}
interface Baseline {
  durationMs?: number;
  runtimeVersions?: unknown;
  version: string;
  comparedRuleCount: number;
  parseErrorResults: Fields[];
}
interface Budget {
  passed: boolean;
  verdict: string;
  maxFalseNegativeCount: number;
  maxFalsePositiveCount: number;
}
interface BaseReport {
  evidence: { runtime?: unknown };
  budget: Budget;
}
interface ProjectReport extends BaseReport {
  schema: "vize.fixtureLintDivergenceRun";
  baseline: Baseline;
  raw: Raw;
  divergence: Divergence;
  files: { comparedCount: number; expectedCount: number | null };
}
interface IndexReport extends BaseReport {
  schema: "vize.fixtureLintDivergenceIndex";
  totals: Record<string, number>;
  projects: Record<string, number>[];
}
interface OriginalProjectReport extends Omit<ProjectReport, "raw" | "divergence"> {
  divergence: Omit<Divergence, "referenceCoverageExpectations">;
}
export type Report = ProjectReport | OriginalProjectReport | IndexReport;
export type ReportSource = Oracle | "typescript";
export function object(value: unknown): Fields {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Fields;
}
function reportValue(value: unknown, source: ReportSource): Report {
  assert.ok(["original", "updated", "typescript"].includes(source));
  const row = object(value);
  assert.equal(typeof row.schema, "string");
  assert.equal(
    object(object(row.evidence).runtime).name,
    source === "typescript" ? "node" : "rust-script",
  );
  object(row.budget);
  assert.equal(row.version, row.schema === "vize.fixtureLintDivergenceRun" ? 2 : 1);
  if (row.schema === "vize.fixtureLintDivergenceRun") {
    object(row.baseline);
    const divergence = object(row.divergence),
      baseline = object(row.baseline);
    object(row.files);
    assert.equal(divergence.schema, "vize.fixtureLintDivergence");
    assert.equal(divergence.version, 1);
    assert.equal(typeof baseline.comparedRuleCount, "number");
    assert.ok(Array.isArray(baseline.parseErrorResults));
    const summary = object(divergence.summary);
    if (source === "original") {
      for (const [owner, key] of [
        [row, "raw"],
        [baseline, "runtimeVersions"],
        [divergence, "referenceCoverageExpectations"],
        [summary, "referenceCoverageIntegrityErrorCount"],
      ] as const)
        assert.equal(Object.hasOwn(owner, key), false, `original must not contain ${key}`);
    } else {
      const raw = object(row.raw);
      object(baseline.runtimeVersions);
      for (const key of ["patinaResults", "patinaFindings", "eslintResults"])
        assert.ok(Array.isArray(raw[key]));
      assert.ok(Array.isArray(divergence.referenceCoverageExpectations));
      assert.equal(typeof summary.referenceCoverageIntegrityErrorCount, "number");
    }
  } else {
    assert.equal(row.schema, "vize.fixtureLintDivergenceIndex");
    assert.ok(Array.isArray(row.projects));
    for (const project of row.projects as unknown[])
      assert.equal(
        Object.hasOwn(object(project), "referenceCoverageIntegrityErrorCount"),
        source !== "original",
      );
  }
  return row as unknown as Report;
}
export function reportFile(result: Result, name: string): ProjectReport {
  const value = result.files[name];
  assert.ok(
    typeof value === "object" && value.schema === "vize.fixtureLintDivergenceRun" && "raw" in value,
  );
  return value;
}
export function indexFile(result: Result): IndexReport {
  const value = result.files["lint-divergence-summary.json"];
  assert.ok(typeof value === "object" && value.schema === "vize.fixtureLintDivergenceIndex");
  return value;
}
export function markdown(result: Result, name: string): string {
  const value = result.files[name];
  assert.equal(typeof value, "string");
  return value as string;
}
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
export function parseReport(text: string, source: ReportSource): Report {
  return reportValue(
    JSON.parse(text, (_key: string, value: unknown, context?: { source?: string }) => {
      if (
        typeof value === "number" &&
        !Number.isSafeInteger(value) &&
        typeof context?.source === "string" &&
        /^-?\d+$/u.test(context.source)
      ) {
        return BigInt(context.source);
      }
      return value;
    }) as unknown,
    source,
  );
}
function canonical(value: unknown, key = ""): string {
  if (typeof value === "bigint") return String(value);
  if (Array.isArray(value)) return `[${value.map((item) => canonical(item)).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((name) => `${JSON.stringify(name)}:${canonical(object(value)[name], name)}`)
      .join(",")}}`;
  }
  // serde retains the two ratio fields as f64 even when they are zero or one.
  if (["falsePositiveRatio", "falseNegativeRatio"].includes(key) && Number.isInteger(value))
    return `${String(value)}.0`;
  return JSON.stringify(value);
}
function digestPayload(divergence: Divergence) {
  return Object.fromEntries(
    Object.entries(divergence).filter(
      ([key]) => !["schema", "version", "project", "upstream", "sha256"].includes(key),
    ),
  );
}
export function assertDigest(divergence: Divergence) {
  const { schema, version, sha256 } = divergence;
  const payload = digestPayload(divergence);
  assert.equal(schema, "vize.fixtureLintDivergence");
  assert.equal(version, 1);
  assert.equal(sha256, hash(canonical(payload)), "complete independently derived digest preimage");
}
function hostSeams(result: Result, rust: boolean) {
  const rustc = rust ? spawnSync("rustc", ["--version"], { encoding: "utf8" }) : undefined;
  if (rustc) assert.equal(rustc.status, 0);
  for (const artifact of Object.values(result.files)) {
    if (typeof artifact !== "object") continue;
    assert.deepEqual(
      artifact.evidence.runtime,
      rust
        ? { name: "rust-script", version: rustc!.stdout.trim() }
        : { name: "node", version: process.version },
    );
    delete artifact.evidence.runtime;
    if ("baseline" in artifact) {
      assert.ok(typeof artifact.baseline.durationMs === "number");
      assert.ok(Number.isSafeInteger(artifact.baseline.durationMs));
      assert.ok(artifact.baseline.durationMs >= 0 && artifact.baseline.durationMs <= 240_000);
      delete artifact.baseline.durationMs;
    }
  }
}
function runtimeVersions(provider: string): Record<string, string> {
  const require = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
  const version = (packageName: string): string => {
    const value = object(require(`${packageName}/package.json`) as unknown).version;
    assert.equal(typeof value, "string");
    return value as string;
  };
  return {
    eslint: version("eslint"),
    "eslint-plugin-vue": version(provider),
    "vue-eslint-parser": version("vue-eslint-parser"),
    "@typescript-eslint/parser": version("@typescript-eslint/parser"),
  };
}
function without(fields: Fields, keys: readonly string[]): Fields {
  return Object.fromEntries(Object.entries(fields).filter(([key]) => !keys.includes(key)));
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
function normalizeReviewedCoverageIdentity(
  expected: ProjectReport,
  actual: ProjectReport,
): boolean {
  const expectedExpectations = expected.divergence.referenceCoverageExpectations,
    actualExpectations = actual.divergence.referenceCoverageExpectations;
  if (expectedExpectations.length === 0 || actualExpectations.length !== 0) return false;
  assert.equal(expectedExpectations.length, 1);
  const expectation = object(expectedExpectations[0]),
    caseId = String(expectation.case);
  assert.ok(
    Object.hasOwn(reviewedScorecardTransitions, caseId),
    `unexpected scorecard reference-coverage transition ${caseId}`,
  );
  const transition = reviewedScorecardTransitions[caseId];
  assert.equal(object(expectation.expectedFinding).ruleId, transition.ruleId);
  assert.equal(expectation.sourceSha256, transition.sourceSha256);
  assert.deepEqual(actual.raw, expected.raw, "scorecard must preserve every raw lint packet");
  assertDigest(expected.divergence);
  assertDigest(actual.divergence);
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
    const coverage = object(referenceCoverage);
    assert.equal(coverage.case, caseId);
    assert.equal(coverage.sourceSha256, transition.sourceSha256);
    assert.equal(coverage.packetSha256, transition.packetSha256);
    assert.equal(coverage.nativePacketSha256, transition.nativePacketSha256);
    assert.equal(object(coverage.runtimeVersions)["eslint-plugin-vue"], "10.9.2");
    assert.equal(finding.ruleId, transition.ruleId);
    return finding;
  });
  const sorted = (entries: Fields[]) =>
    structuredClone(entries).sort((left, right) => canonical(left).localeCompare(canonical(right)));
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
    without(object(actual.budget), ["falsePositivePassed", "passed", "unusableReason", "verdict"]),
    without(object(expected.budget), [
      "falsePositivePassed",
      "passed",
      "unusableReason",
      "verdict",
    ]),
  );
  assert.equal(
    object(actual.budget).falsePositivePassed,
    actual.divergence.falsePositives.length === 0,
  );
  return true;
}
function assertScorecardIdentityTransition(expected: Result, actual: Result) {
  const historicalVersions = runtimeVersions("eslint-plugin-vue"),
    scorecardVersions = runtimeVersions("eslint-plugin-vue-scorecard"),
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
        ? normalizeReviewedCoverageIdentity(expectedArtifact, actualArtifact)
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
    const expectedMarkdown = markdown(expected, markdownName);
    const actualMarkdown = markdown(actual, markdownName);
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
function removeOriginalAdditions(artifact: ProjectReport) {
  const versions = runtimeVersions("eslint-plugin-vue");
  assert.deepEqual(
    artifact.baseline.runtimeVersions,
    artifact.baseline.comparedRuleCount ? versions : {},
  );
  assert.deepEqual(Object.keys(artifact.raw).sort(), [
    "eslintResults",
    "patinaFindings",
    "patinaResults",
  ]);
  assert.ok(Array.isArray(artifact.raw.eslintResults));
  assert.ok(Array.isArray(artifact.raw.patinaResults));
  assert.deepEqual(
    artifact.raw.patinaFindings,
    artifact.raw.patinaResults.flatMap((entry) =>
      entry.messages.map((finding) => ({ ...finding, file: entry.file })),
    ),
  );
  assert.deepEqual(artifact.divergence.referenceCoverageExpectations, []);
  assert.equal(artifact.divergence.summary.referenceCoverageIntegrityErrorCount, 0);
  assertDigest(artifact.divergence);
  Reflect.deleteProperty(artifact, "raw");
  delete artifact.baseline.runtimeVersions;
  Reflect.deleteProperty(artifact.divergence, "referenceCoverageExpectations");
  delete artifact.divergence.summary.referenceCoverageIntegrityErrorCount;
  artifact.divergence.sha256 = hash(canonical(digestPayload(artifact.divergence)));
}
export function assertParity(expected: Result, actual: Result, oracle: Oracle) {
  expected = structuredClone(expected);
  actual = structuredClone(actual);
  const oldHeader = `Usage: rust-script ${command}.rs [options]`;
  if (expected.stdout.startsWith(`${oldHeader}\n`)) {
    const [header, ...remaining] = actual.stdout.split("\n");
    assert.ok([oldHeader, `Usage: node ${command}.ts [options]`].includes(header));
    assert.deepEqual(remaining, expected.stdout.split("\n").slice(1));
    actual.stdout = expected.stdout;
  }
  hostSeams(expected, true);
  hostSeams(actual, false);
  assertScorecardIdentityTransition(expected, actual);
  if (/^warning: stripping debug info with `rust-objcopy` failed:[\s\S]+$/u.test(expected.stderr))
    expected.stderr = "";
  if (oracle === "original") {
    assert.deepEqual(expected.rawUnknownTokens, {});
    actual.rawUnknownTokens = {};
  }
  if (oracle === "original" && actual.files["lint-divergence-summary.json"]) {
    for (const row of indexFile(actual).projects) {
      assert.equal(row.referenceCoverageIntegrityErrorCount, 0);
      delete row.referenceCoverageIntegrityErrorCount;
    }
  }
  for (const [file, artifact] of Object.entries(actual.files)) {
    if (!file.endsWith("-lint-divergence.json")) continue;
    assert.ok(
      typeof artifact === "object" &&
        artifact.schema === "vize.fixtureLintDivergenceRun" &&
        "raw" in artifact,
    );
    assertDigest(artifact.divergence);
    if (oracle === "original") {
      removeOriginalAdditions(artifact);
      assert.equal(artifact.divergence.summary.patinaOutsideBaselineSurfaceCount, 0);
      const markdownName = file.replace(/\.json$/u, ".md");
      let markdownText = markdown(actual, markdownName);
      const additions = [
        `Patina findings outside reference coverage: ${artifact.divergence.summary.patinaOutsideBaselineSurfaceCount}\n`,
        "Reference coverage integrity errors: 0\n",
        "### Patina findings outside reference coverage: none\n\n",
      ];
      for (const addition of additions) {
        assert.equal(
          markdownText.split(addition).length,
          2,
          `exact Markdown addition: ${addition}`,
        );
        markdownText = markdownText.replace(addition, "");
        actual.files[markdownName] = markdownText;
      }
    }
  }
  assert.deepEqual(actual, expected);
}

export function assertMembershipCorrection(
  old: Result,
  actual: Result,
  entry: Record<string, unknown>,
) {
  const expected = structuredClone(old),
    project = String(entry.project),
    finding = object(entry.finding);
  assert.equal(expected.status, 0);
  assert.match(
    expected.stderr,
    /^(?:|warning: stripping debug info with `rust-objcopy` failed:[\s\S]+)$/u,
  );
  expected.stderr = "";
  const report = reportFile(expected, `${project}-lint-divergence.json`),
    index = indexFile(expected);
  assertDigest(report.divergence);
  const coverage = {
    case: entry.id,
    category: "reference-coverage-gap",
    ...Object.fromEntries(
      [
        "issue",
        "reason",
        "revision",
        "sourceSha256",
        "runtimeVersions",
        "options",
        "upstreamOptions",
        "packetSha256",
        "nativePacketSha256",
      ].map((key) => [key, entry[key]]),
    ),
  };
  assert.deepEqual(report.divergence.patinaOutsideBaselineSurface, [
    { ...finding, referenceCoverage: coverage },
  ]);
  assert.deepEqual(report.divergence.falsePositives, []);
  assert.deepEqual(report.divergence.referenceCoverageExpectations, []);
  const targets: Record<string, Fields> = {
    summary: report.divergence.summary,
    budget: object(report.budget),
    indexTotals: object(object(index).totals),
    indexProject: index.projects[0],
    indexBudget: object(index.budget),
  };
  for (const [name, delta] of Object.entries(referenceFixtures.membershipDelta.fields)) {
    const target = targets[name];
    assert.ok(target);
    assert.deepEqual(
      Object.fromEntries(Object.keys(delta.before).map((key) => [key, target[key]])),
      delta.before,
    );
    Object.assign(target, delta.after);
  }
  report.divergence.falsePositives = [finding as Finding];
  report.divergence.patinaOutsideBaselineSurface = [];
  report.divergence.sha256 = hash(canonical(digestPayload(report.divergence)));
  const file = `${project}-lint-divergence.md`;
  let text = markdown(expected, file);
  for (const [before, after] of referenceFixtures.membershipDelta.markdown) {
    assert.equal(text.split(before).length, 2);
    text = text.replace(before, after);
  }
  expected.files[file] = text;
  expected.status = 1;
  expected.stderr = referenceFixtures.membershipDelta.stderr;
  assertParity(expected, actual, "updated");
}
