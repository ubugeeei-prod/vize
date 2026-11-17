import fs from "node:fs";
import path from "node:path";
import { compareLintFindings, selectComparableRules } from "./compare.ts";
import { normalizePath } from "./records.ts";
import { assertBudgetsPassed, evaluateBudget, renderMarkdown, writeIndex } from "./reporting.ts";
import {
  collectRunEvidence,
  resolveVizeLaunch,
  runEslintBaseline,
  runPatina,
  type Launch,
} from "./runtime.ts";
import { collectVueInputPaths, parseArgs, selectedProjects, type Args } from "./selection.ts";
import { BaselineArtifactStage, removeStagedBaselineArtifacts } from "./baseline_stage.ts";
import { isolationRoot, relocatePaths, withIsolatedFixture } from "./isolation.ts";
import {
  matrixRuleOptions,
  matrixVueVersion,
  writeMatrixConfig,
  type MatrixRuleOption,
} from "./matrix_rule_options.ts";
import { vue2DialectDisabledRules } from "./vue2_dialect.ts";
import {
  errorMessage,
  get,
  object,
  projectString,
  readJson,
  relativePath,
  repoRoot,
  unsigned,
  writeText,
  type Json,
  type JsonObject,
} from "./value.ts";

export function reconcileCorpus(
  project: string,
  files: string[],
  results: Json[],
  cwd: string,
): void {
  const linted = new Set(
    results.flatMap((result) => {
      const file = get(result, "filePath");
      return typeof file === "string" ? [normalizePath(file, cwd, "filePath")] : [];
    }),
  );
  const missing = files.filter((file) => !linted.has(file));
  if (missing.length)
    throw new Error(
      `${project}: baseline skipped ${missing.length} of ${files.length} files, starting with ${missing[0]}`,
    );
}
function hydrated(directory: string): boolean {
  try {
    return fs.statSync(directory).isDirectory() && fs.readdirSync(directory).length > 0;
  } catch {
    return false;
  }
}
function removePriorArtifacts(outputDir: string, projects: Json[]): void {
  removeStagedBaselineArtifacts(outputDir);
  for (const filename of [
    "lint-divergence-summary.json",
    ...projects.flatMap((project) => {
      const id = projectString(project, "id");
      return [`${id}-lint-divergence.json`, `${id}-lint-divergence.md`];
    }),
  ])
    fs.rmSync(path.join(outputDir, filename), { force: true });
}
function measureProject(
  root: string,
  args: Args,
  launch: Launch,
  evidence: Json,
  ruleMap: Json,
  comparable: ReturnType<typeof selectComparableRules>,
  project: Json,
  matrix: { overrides: readonly MatrixRuleOption[]; ledger: Json } = {
    overrides: [],
    ledger: null,
  },
): JsonObject | null {
  const id = projectString(project, "id"),
    fixtureDir = path.resolve(root, projectString(project, "fixturePath"));
  if (!hydrated(fixtureDir)) {
    console.log(`Skipped ${id}: fixture is not hydrated`);
    return null;
  }
  const files = collectVueInputPaths(fixtureDir, project),
    expectedCount = unsigned(get(project, "expectedVueFileCount")) ?? null;
  if (expectedCount === 0 && files.length)
    throw new Error(`${id} matched ${files.length} Vue files, expected 0`);
  if (!files.length && expectedCount !== 0) throw new Error(`${id} matched no Vue files`);
  // Both providers lint a mirror that resolves packages only from the
  // fixture's own install, never from the repository's `node_modules`.
  return withIsolatedFixture(fixtureDir, isolationRoot(), id, (cwd) =>
    measureMirror(
      { cwd, fixtureDir },
      root,
      args,
      launch,
      evidence,
      ruleMap,
      comparable,
      project,
      files,
      matrix,
    ),
  );
}

function measureMirror(
  { cwd, fixtureDir }: { cwd: string; fixtureDir: string },
  root: string,
  args: Args,
  launch: Launch,
  evidence: Json,
  ruleMap: Json,
  comparable: ReturnType<typeof selectComparableRules>,
  project: Json,
  files: string[],
  matrix: { overrides: readonly MatrixRuleOption[]; ledger: Json },
): JsonObject {
  const id = projectString(project, "id"),
    expectedCount = unsigned(get(project, "expectedVueFileCount")) ?? null;
  const vueVersion = matrixVueVersion(matrix.ledger, projectString(project, "fixturePath"));
  const config = writeMatrixConfig(matrix.overrides, vueVersion);
  const patina = (() => {
    try {
      return files.length === 0
        ? { results: [], findings: [] }
        : runPatina(cwd, project, args, launch, files, config);
    } finally {
      if (config !== null) fs.rmSync(path.dirname(config), { recursive: true, force: true });
    }
  })();
  const json = path.join(args.outputDir, `${id}-lint-divergence.json`),
    markdown = path.join(args.outputDir, `${id}-lint-divergence.md`),
    stage = new BaselineArtifactStage(args.outputDir, { from: cwd, to: fixtureDir });
  try {
    const started = performance.now(),
      baseline = runEslintBaseline(
        root,
        cwd,
        files,
        comparable.rules,
        process.env.VIZE_LINT_BASELINE_UNBOUNDED === "1" ? null : undefined,
        stage,
      ),
      duration = Math.floor(performance.now() - started),
      selectedPreset = args.preset ?? "all-mapped";
    const baselineRules = object(baseline.rules);
    if (!baselineRules) throw new Error("baseline must report its available rules");
    if (baseline.results.length) reconcileCorpus(id, files, baseline.results, cwd);
    const divergence = compareLintFindings(
      id,
      cwd,
      ruleMap,
      patina.findings,
      baseline.classification,
      {
        revision: get(project, "revision"),
        preset: selectedPreset,
        runtimeVersions: baseline.runtimeVersions,
        rules: baselineRules,
        files,
      },
      vueVersion === null ? new Set() : vue2DialectDisabledRules(root),
    );
    const artifact: JsonObject = {
      schema: "vize.fixtureLintDivergenceRun",
      version: 2,
      project: id,
      revision: get(project, "revision"),
      preset: selectedPreset,
      evidence,
      files: { comparedCount: files.length, expectedCount },
      baseline: {
        package: get(ruleMap, "upstream", "package"),
        version: baseline.version,
        runtimeVersions: baseline.runtimeVersions,
        comparedRuleCount: Object.keys(baselineRules).length,
        mappedRuleCount: comparable.mappedRuleCount,
        skippedByPresetCount: comparable.skippedByPresetCount,
        droppedConfigMessageCount: baseline.droppedConfigMessageCount,
        durationMs: duration,
        parseErrorResults: baseline.parseErrorResults,
      },
      raw: {
        patinaResults: patina.results,
        patinaFindings: patina.findings,
        eslintResults: [],
      },
      divergence,
    };
    artifact.budget = evaluateBudget(artifact);
    // Reports name the fixture, not the temporary mirror.
    artifact.baseline = relocatePaths(artifact.baseline, cwd, fixtureDir);
    stage.writeArtifact(json, artifact);
    writeText(markdown, renderMarkdown(artifact));
    console.log(`Wrote ${relativePath(root, json)}`);
    return artifact;
  } catch (error) {
    fs.rmSync(json, { force: true });
    fs.rmSync(markdown, { force: true });
    throw error;
  } finally {
    stage.cleanup();
  }
}

/** Execute the complete measured report; no compatibility implementation runs. */
export function runLintDivergenceReport(argv: string[] = process.argv.slice(2)): Json[] {
  const root = repoRoot(),
    args = parseArgs(argv, root);
  if (args === null) return [];
  const registry = readJson(args.registry),
    ruleMap = readJson(path.join(root, "tests/_fixtures/patina-eslint-vue-rule-map.json"));
  let ledger: Json = [];
  try {
    ledger = readJson(path.join(root, "tests/_fixtures/patina-lint-documented-divergences.json"));
  } catch {
    /* The retained command treats an unreadable ledger as empty. */
  }
  if (!Array.isArray(ledger)) throw new Error("The lint divergence ledger must be an array");
  const projects = selectedProjects(registry, args);
  if (!projects.length) throw new Error("No linter-covered project matched the selection");
  const matrix = {
    overrides: matrixRuleOptions(root, ruleMap),
    ledger: readJson(path.join(root, "tests/_fixtures/fixture-compatibility-ledger.json")),
  };
  const launch = resolveVizeLaunch(root, args.vizeBin),
    evidence = collectRunEvidence(root),
    comparable = selectComparableRules(ruleMap, args.preset, args.measureCoverageGap);
  try {
    fs.mkdirSync(args.outputDir, { recursive: true });
  } catch (error) {
    throw new Error(`cannot create ${args.outputDir}: ${errorMessage(error)}`);
  }
  removePriorArtifacts(args.outputDir, projects);
  const artifacts: Json[] = [];
  for (const project of projects) {
    const artifact = measureProject(
      root,
      args,
      launch,
      evidence,
      ruleMap,
      comparable,
      project,
      matrix,
    );
    if (artifact !== null) artifacts.push(artifact);
  }
  writeIndex(root, args.outputDir, args.preset, evidence, artifacts);
  assertBudgetsPassed(artifacts, args.budgetMode);
  return artifacts;
}
