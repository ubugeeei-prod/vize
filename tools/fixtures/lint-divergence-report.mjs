import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  resolveBaselineRuntime,
  runBaseline,
  selectComparableRules,
} from "./lint-divergence-baseline.mjs";
import {
  assertBudgetsPassed,
  attachBudget,
  parseBudgetMode,
  summarizeBudgets,
} from "./lint-divergence-budget.mjs";
import { renderMarkdown } from "./lint-divergence-markdown.mjs";
import { compareLintFindings } from "./lint-divergence.mjs";
import { collectRunEvidence } from "./run-evidence.mjs";
import { collectVueInputPaths } from "./tool-matrix-inputs.mjs";
import { resolveVizeLaunch } from "./tool-matrix-run.mjs";

/**
 * Run `tools/fixtures/lint-divergence.mjs` over the pinned real-project corpus.
 *
 * This is the linter's counterpart to `typecheck-divergence-report.mjs`: the
 * comparator that classifies patina against `eslint-plugin-vue` existed and was
 * unit-tested, but nothing ever pointed it at a real repository, so the reviewed
 * ledger in `tests/_fixtures/patina-lint-documented-divergences.json` was empty
 * for want of a measurement rather than for want of divergences.
 *
 * Both linters read the same file list — the registry's `vueGlobs`, the same
 * input set `tool-matrix-report.mjs` feeds `vize lint` — so a file count
 * mismatch is reported rather than silently compared. Findings are classified by
 * rule-map status, so an unimplemented upstream rule lands in its own bucket and
 * never inflates the false-negative count.
 */
const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..");
const defaultRegistry = join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const ruleMapPath = join(repoRoot, "tests", "_fixtures", "patina-eslint-vue-rule-map.json");
const ledgerPath = join(repoRoot, "tests", "_fixtures", "patina-lint-documented-divergences.json");
const defaultPreset = "ecosystem";

export async function runLintDivergenceReport(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const registry = readJson(args.registry);
  const ruleMap = readJson(ruleMapPath);
  const ledger = readLedger();
  const runtime = resolveBaselineRuntime();
  const launch = resolveVizeLaunch(args.vizeBin, false);
  const { rules, skippedByPreset } = selectComparableRules(ruleMap, args.preset, {
    includeUnimplemented: args.measureCoverageGap,
  });
  const mappedRuleCount = Object.values(ruleMap.entries).filter(
    (entry) => entry.status === "mapped",
  ).length;

  const projects = registry.projects
    .filter((project) => project.coverage.includes("linter"))
    .filter((project) => args.projects.length === 0 || args.projects.includes(project.id))
    .filter((_, index) => index % args.shardCount === args.shardIndex);
  if (projects.length === 0) throw new Error("No linter-covered project matched the selection");

  mkdirSync(args.outputDir, { recursive: true });
  const evidence = collectRunEvidence();
  const artifacts = [];
  for (const project of projects) {
    const artifact = await measureProject({
      project,
      args,
      evidence,
      launch,
      ledger,
      mappedRuleCount,
      ruleMap,
      rules,
      runtime,
      skippedByPreset,
    });
    if (artifact != null) artifacts.push(artifact);
  }
  writeIndex(args.outputDir, evidence, args, artifacts);
  assertBudgetsPassed(artifacts, args.budgetMode);
  return artifacts;
}

async function measureProject(context) {
  const { args, project, ruleMap, rules, runtime } = context;
  const cwd = resolve(repoRoot, project.fixturePath);
  if (!isHydrated(cwd)) {
    process.stdout.write(`Skipped ${project.id}: fixture is not hydrated\n`);
    return null;
  }
  const files = collectVueInputPaths(cwd, project.vueGlobs);
  if (files.length === 0 && project.expectedVueFileCount !== 0) {
    throw new Error(`${project.id} matched no Vue files`);
  }

  const patinaFindings = runPatina(context, cwd, files);
  const startedAt = Date.now();
  const baselineRun = await runBaseline(runtime, cwd, files, rules);
  const eslintResults = baselineRun.results;
  const baselineDurationMs = Date.now() - startedAt;
  reconcileCorpus(project.id, files, eslintResults, cwd);

  const divergence = compareLintFindings({
    projectId: project.id,
    cwd,
    ruleMap,
    patinaFindings,
    eslintResults,
    documentedDivergences: context.ledger,
  });
  const artifact = attachBudget({
    schema: "vize.fixtureLintDivergenceRun",
    version: 2,
    project: project.id,
    revision: project.revision,
    preset: args.preset ?? "all-mapped",
    evidence: context.evidence,
    files: { comparedCount: files.length },
    baseline: {
      package: ruleMap.upstream.package,
      version: runtime.version,
      comparedRuleCount: Object.keys(rules).length,
      mappedRuleCount: context.mappedRuleCount,
      skippedByPresetCount: context.skippedByPreset.length,
      droppedConfigMessageCount: baselineRun.droppedConfigMessageCount,
      durationMs: baselineDurationMs,
    },
    divergence,
  });
  const jsonPath = join(args.outputDir, `${project.id}-lint-divergence.json`);
  const markdownPath = join(args.outputDir, `${project.id}-lint-divergence.md`);
  writeFileSync(jsonPath, `${JSON.stringify(artifact, null, 2)}\n`);
  writeFileSync(markdownPath, renderMarkdown(artifact));
  process.stdout.write(`Wrote ${relative(repoRoot, jsonPath)}\n`);
  return artifact;
}

/**
 * The patina half, invoked exactly as the corpus matrix invokes it
 * (`tools/fixtures/tool-matrix-command.mjs`) so the two runs stay comparable,
 * then flattened from the per-file envelope into the one-record-per-finding
 * dialect the comparator normalizes.
 */
function runPatina(context, cwd, files) {
  const { args, launch, project } = context;
  const commandArgs = [
    ...launch.prefix,
    "lint",
    ...project.vueGlobs,
    "--format",
    "json",
    "--no-config",
  ];
  if (args.preset != null) commandArgs.push("--preset", args.preset);
  const result = spawnSync(launch.command, commandArgs, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 1024 * 1024 * 1024,
    timeout: args.timeoutMs,
  });
  if (result.error != null) throw new Error(`vize lint failed to run: ${result.error.message}`);
  if (result.status !== 0 && result.status !== 1) {
    throw new Error(`vize lint exited with unsupported status ${result.status}: ${result.stderr}`);
  }
  let envelope;
  try {
    envelope = JSON.parse(result.stdout);
  } catch (error) {
    throw new Error(`vize lint produced invalid JSON: ${error.message}`);
  }
  if (!Array.isArray(envelope)) throw new Error("vize lint envelope must be an array");
  if (envelope.length !== files.length) {
    throw new Error(
      `${project.id}: vize lint reported ${envelope.length} files, expected ${files.length}`,
    );
  }
  return envelope.flatMap((entry) =>
    entry.messages.map((message) => ({ ...message, file: entry.file })),
  );
}

/**
 * Both linters must have read the same corpus. A baseline that silently skipped
 * files — an ignore file, an unreadable path — would report every finding in
 * them as a false negative, so the mismatch is raised here instead.
 */
export function reconcileCorpus(projectId, files, eslintResults, cwd) {
  const linted = new Set(
    eslintResults.map((result) => relative(cwd, result.filePath).replaceAll("\\", "/")),
  );
  const missing = files.filter((file) => !linted.has(file));
  if (missing.length > 0) {
    throw new Error(
      `${projectId}: baseline skipped ${missing.length} of ${files.length} files, ` +
        `starting with ${missing[0]}`,
    );
  }
}

function writeIndex(outputDir, evidence, args, artifacts) {
  const summary = {
    schema: "vize.fixtureLintDivergenceIndex",
    version: 1,
    evidence,
    preset: args.preset ?? "all-mapped",
    projectCount: artifacts.length,
    budget: summarizeBudgets(artifacts),
    totals: sumTotals(artifacts),
    projects: artifacts.map((artifact) => ({
      project: artifact.project,
      ...artifact.divergence.summary,
    })),
  };
  const path = join(outputDir, "lint-divergence-summary.json");
  writeFileSync(path, `${JSON.stringify(summary, null, 2)}\n`);
  process.stdout.write(`Wrote ${relative(repoRoot, path)}\n`);
  return summary;
}

function sumTotals(artifacts) {
  const keys = [
    "patinaFindingCount",
    "baselineFindingCount",
    "comparableBaselineCount",
    "sharedCount",
    "messageDifferenceCount",
    "documentedDivergenceCount",
    "falsePositiveCount",
    "falseNegativeCount",
    "unimplementedCount",
    "intentionalDivergenceCount",
    "patinaOnlyRuleFindingCount",
    "baselineParseErrorCount",
    "baselineInvalidRangeCount",
  ];
  const totals = Object.fromEntries(keys.map((key) => [key, 0]));
  for (const artifact of artifacts) {
    for (const key of keys) totals[key] += artifact.divergence.summary[key];
  }
  return totals;
}

function isHydrated(cwd) {
  return existsSync(cwd) && readdirSync(cwd).some((entry) => entry !== ".git");
}

function readLedger() {
  const ledger = readJson(ledgerPath);
  if (!Array.isArray(ledger)) throw new Error("The lint divergence ledger must be an array");
  return ledger;
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function parseArgs(argv) {
  const args = {
    measureCoverageGap: false,
    budgetMode: "enforce",
    outputDir: join(repoRoot, ".vize", "lint-divergence"),
    preset: defaultPreset,
    projects: [],
    registry: defaultRegistry,
    shardCount: 1,
    shardIndex: 0,
    timeoutMs: 600_000,
    vizeBin: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const value = () => {
      if (argv[index + 1] == null) throw new Error(`${arg} requires a value`);
      return argv[++index];
    };
    if (arg === "--output-dir") args.outputDir = resolve(value());
    else if (arg === "--budget-mode") args.budgetMode = parseBudgetMode(value());
    else if (arg === "--preset") args.preset = value();
    else if (arg === "--all-mapped-rules") args.preset = null;
    else if (arg === "--measure-coverage-gap") args.measureCoverageGap = true;
    else if (arg === "--project") args.projects.push(...splitCsv(value()));
    else if (arg === "--registry") args.registry = resolve(value());
    else if (arg === "--shard-count") args.shardCount = positiveInteger(value(), arg);
    else if (arg === "--shard-index") args.shardIndex = nonNegativeInteger(value(), arg);
    else if (arg === "--timeout-ms") args.timeoutMs = positiveInteger(value(), arg);
    else if (arg === "--vize-bin") args.vizeBin = value();
    else if (arg === "--help" || arg === "-h") return printHelpAndExit();
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (args.shardIndex >= args.shardCount) {
    throw new Error("--shard-index must be less than --shard-count");
  }
  return args;
}

function printHelpAndExit() {
  process.stdout.write(
    [
      "Usage: rust-script tools/commands/fixtures/lint-divergence-report.rs [options]",
      "",
      "Classify vize lint against eslint-plugin-vue over the pinned real projects.",
      "",
      "  --project <id[,id]>     Limit registry projects",
      "  --preset <name>         Patina preset under test (default: ecosystem)",
      "  --all-mapped-rules      Compare every mapped rule, ignoring preset membership",
      "  --measure-coverage-gap  Also enable unimplemented upstream rules, to count",
      "                          the findings a project loses (never a false negative)",
      "  --shard-index <n>       Zero-based project shard index",
      "  --shard-count <n>       Total balanced project shards",
      "  --output-dir <dir>      Report directory",
      "  --budget-mode <mode>    enforce or record-only (default: enforce)",
      "  --vize-bin <path>       Vize executable",
      "  --timeout-ms <n>        Per-project vize lint timeout",
      "",
    ].join("\n"),
  );
  process.exit(0);
}

function splitCsv(value) {
  return value
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean);
}

function positiveInteger(value, name) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return parsed;
}

function nonNegativeInteger(value, name) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 0) {
    throw new Error(`${name} must be a non-negative integer`);
  }
  return parsed;
}

if (process.argv[1] != null && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    await runLintDivergenceReport();
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exit(1);
  }
}
