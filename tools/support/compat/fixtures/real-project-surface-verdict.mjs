import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const realProjectSurfaceNames = [
  "waiver-audit",
  "typecheck-dependencies",
  "core-tools",
  "lsp",
  "lint-divergence",
  "syntax-highlighter",
  "glyph",
  "typecheck-divergence",
];

const validOutcomes = new Set(["success", "failure", "cancelled", "skipped"]);

function recordOnlyVerdict(outcome, mode) {
  return mode === "record-only" && outcome === "failure" ? "success" : outcome;
}

export function createRealProjectSurfaceResultsFromWorkflow(environment = process.env) {
  return [
    { name: "waiver-audit", outcome: environment.VIZE_WAIVER_AUDIT_OUTCOME },
    {
      name: "typecheck-dependencies",
      outcome: recordOnlyVerdict(
        environment.VIZE_TYPECHECK_DEPENDENCIES_OUTCOME,
        environment.TYPECHECK_DEPENDENCIES_MODE,
      ),
    },
    {
      name: "core-tools",
      outcome: recordOnlyVerdict(environment.VIZE_CORE_TOOLS_OUTCOME, environment.CORE_TOOLS_MODE),
    },
    {
      name: "lsp",
      outcome: recordOnlyVerdict(environment.VIZE_LSP_OUTCOME, environment.LSP_MODE),
    },
    {
      name: "lint-divergence",
      outcome: recordOnlyVerdict(
        environment.VIZE_LINT_DIVERGENCE_OUTCOME,
        environment.LINT_DIVERGENCE_MODE,
      ),
    },
    { name: "syntax-highlighter", outcome: environment.VIZE_SYNTAX_HIGHLIGHTER_OUTCOME },
    { name: "glyph", outcome: environment.VIZE_GLYPH_OUTCOME },
    {
      name: "typecheck-divergence",
      outcome: recordOnlyVerdict(
        environment.VIZE_TYPECHECK_DIVERGENCE_OUTCOME,
        environment.TYPECHECK_DIVERGENCE_MODE,
      ),
    },
  ];
}

export function createRealProjectSurfaceVerdict(results, environment = process.env) {
  const expected = new Set(realProjectSurfaceNames);
  const seen = new Set();
  for (const result of results) {
    if (result == null || typeof result !== "object" || Array.isArray(result)) {
      throw new Error("surface results must be objects");
    }
    if (!expected.has(result.name)) {
      throw new Error(`unknown real-project surface: ${String(result.name)}`);
    }
    if (seen.has(result.name)) {
      throw new Error(`duplicate real-project surface: ${result.name}`);
    }
    if (!validOutcomes.has(result.outcome)) {
      throw new Error(`invalid outcome for ${result.name}: ${String(result.outcome)}`);
    }
    seen.add(result.name);
  }
  const missing = realProjectSurfaceNames.filter((name) => !seen.has(name));
  if (missing.length > 0) {
    throw new Error(`missing real-project surface verdict(s): ${missing.join(", ")}`);
  }
  const failed = results.filter((result) => result.outcome !== "success");
  return {
    schema: "vize.realProjectSurfaceVerdict",
    version: 1,
    sourceCommit: environment.GITHUB_SHA ?? null,
    shardIndex: environment.FIXTURE_SHARD_INDEX ?? null,
    status: failed.length === 0 ? "success" : "failure",
    surfaces: realProjectSurfaceNames.map((name) => results.find((result) => result.name === name)),
    failedSurfaceNames: failed
      .map((result) => result.name)
      .sort((left, right) => left.localeCompare(right)),
  };
}

function parseArguments(argv) {
  let output = null;
  let fromWorkflowEnv = false;
  const results = [];
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    const value = argv[index + 1];
    if (argument === "--from-workflow-env") {
      fromWorkflowEnv = true;
      continue;
    }
    if (argument === "--output" && output == null && value && !value.startsWith("--")) {
      output = resolve(value);
      index += 1;
      continue;
    }
    if (argument === "--surface" && value && !value.startsWith("--")) {
      const separator = value.indexOf("=");
      if (separator < 1) throw new Error(`invalid --surface value: ${value}`);
      results.push({ name: value.slice(0, separator), outcome: value.slice(separator + 1) });
      index += 1;
      continue;
    }
    throw new Error(`unknown or incomplete argument: ${argument}`);
  }
  if (output == null) throw new Error("--output is required");
  if (fromWorkflowEnv && results.length > 0) {
    throw new Error("--from-workflow-env cannot be combined with --surface");
  }
  return { output, fromWorkflowEnv, results };
}

function main() {
  const { output, fromWorkflowEnv, results } = parseArguments(process.argv.slice(2));
  const surfaces = fromWorkflowEnv ? createRealProjectSurfaceResultsFromWorkflow() : results;
  const artifact = createRealProjectSurfaceVerdict(surfaces);
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, `${JSON.stringify(artifact, null, 2)}\n`);
  if (artifact.status !== "success") {
    throw new Error(`real-project surfaces failed: ${artifact.failedSurfaceNames.join(", ")}`);
  }
  process.stdout.write(`all ${artifact.surfaces.length} real-project surfaces succeeded\n`);
}

const isMain = process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isMain) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
