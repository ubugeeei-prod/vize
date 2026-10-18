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
import {
  errorMessage,
  get,
  list,
  projectString,
  readJson,
  relativePath,
  repoRoot,
  unsigned,
  writeJson,
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
function measureProject(
  root: string,
  args: Args,
  launch: Launch,
  evidence: Json,
  ruleMap: Json,
  comparable: ReturnType<typeof selectComparableRules>,
  project: Json,
): JsonObject | null {
  const id = projectString(project, "id"),
    cwd = path.resolve(root, projectString(project, "fixturePath"));
  if (!hydrated(cwd)) {
    console.log(`Skipped ${id}: fixture is not hydrated`);
    return null;
  }
  const files = collectVueInputPaths(cwd, project),
    expectedCount = unsigned(get(project, "expectedVueFileCount")) ?? null;
  if (expectedCount === 0 && files.length)
    throw new Error(`${id} matched ${files.length} Vue files, expected 0`);
  if (!files.length && expectedCount !== 0) throw new Error(`${id} matched no Vue files`);
  const patina = runPatina(cwd, project, args, launch, files.length);
  const started = performance.now(),
    baseline = runEslintBaseline(root, cwd, files, comparable.rules),
    duration = Math.floor(performance.now() - started);
  reconcileCorpus(id, files, baseline.results, cwd);
  const divergence = compareLintFindings(id, cwd, ruleMap, patina.findings, baseline.results, {
    revision: get(project, "revision"),
    preset: args.preset,
    runtimeVersions: baseline.runtimeVersions,
    rules: comparable.rules,
    files,
  });
  const artifact: JsonObject = {
    schema: "vize.fixtureLintDivergenceRun",
    version: 2,
    project: id,
    revision: get(project, "revision"),
    preset: args.preset ?? "all-mapped",
    evidence,
    files: { comparedCount: files.length, expectedCount },
    baseline: {
      package: get(ruleMap, "upstream", "package"),
      version: baseline.version,
      runtimeVersions: baseline.runtimeVersions,
      comparedRuleCount: Object.keys(comparable.rules).length,
      mappedRuleCount: comparable.mappedRuleCount,
      skippedByPresetCount: comparable.skippedByPresetCount,
      droppedConfigMessageCount: baseline.droppedConfigMessageCount,
      durationMs: duration,
      parseErrorResults: baseline.results.filter((result) =>
        list(get(result, "messages")).some((message) => get(message, "ruleId") === null),
      ),
    },
    raw: {
      patinaResults: patina.results,
      patinaFindings: patina.findings,
      eslintResults: baseline.rawResults,
    },
    divergence,
  };
  artifact.budget = evaluateBudget(artifact);
  const json = path.join(args.outputDir, `${id}-lint-divergence.json`),
    markdown = path.join(args.outputDir, `${id}-lint-divergence.md`);
  writeJson(json, artifact);
  writeText(markdown, renderMarkdown(artifact));
  console.log(`Wrote ${relativePath(root, json)}`);
  return artifact;
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
  const launch = resolveVizeLaunch(root, args.vizeBin),
    evidence = collectRunEvidence(root),
    comparable = selectComparableRules(ruleMap, args.preset, args.measureCoverageGap);
  try {
    fs.mkdirSync(args.outputDir, { recursive: true });
  } catch (error) {
    throw new Error(`cannot create ${args.outputDir}: ${errorMessage(error)}`);
  }
  const artifacts: Json[] = [];
  for (const project of projects) {
    const artifact = measureProject(root, args, launch, evidence, ruleMap, comparable, project);
    if (artifact !== null) artifacts.push(artifact);
  }
  writeIndex(root, args.outputDir, args.preset, evidence, artifacts);
  assertBudgetsPassed(artifacts, args.budgetMode);
  return artifacts;
}
