import fs from "node:fs";

import { runEntrypoint } from "../../support/command-runtime.ts";

const surfaceNames = [
  "waiver-audit",
  "typecheck-dependencies",
  "core-tools",
  "lsp",
  "lint-divergence",
  "syntax-highlighter",
  "glyph",
  "typecheck-divergence",
] as const;

type SurfaceName = (typeof surfaceNames)[number];
type SurfaceOutcome = "success" | "failure" | "cancelled" | "skipped";

interface SurfaceResult {
  readonly name: SurfaceName;
  readonly outcome: SurfaceOutcome;
  readonly rawOutcome: string;
  readonly budgetMode: string;
}

interface Arguments {
  readonly output: string;
  readonly fromWorkflowEnvironment: boolean;
  readonly surfaces: SurfaceResult[];
}

interface WorkflowSurface {
  readonly name: SurfaceName;
  readonly outcomeVariable: string;
  readonly modeVariable?: string;
}

const workflowSurfaces: readonly WorkflowSurface[] = [
  { name: "waiver-audit", outcomeVariable: "VIZE_WAIVER_AUDIT_OUTCOME" },
  {
    name: "typecheck-dependencies",
    outcomeVariable: "VIZE_TYPECHECK_DEPENDENCIES_OUTCOME",
    modeVariable: "TYPECHECK_DEPENDENCIES_MODE",
  },
  {
    name: "core-tools",
    outcomeVariable: "VIZE_CORE_TOOLS_OUTCOME",
    modeVariable: "CORE_TOOLS_MODE",
  },
  { name: "lsp", outcomeVariable: "VIZE_LSP_OUTCOME", modeVariable: "LSP_MODE" },
  {
    name: "lint-divergence",
    outcomeVariable: "VIZE_LINT_DIVERGENCE_OUTCOME",
    modeVariable: "LINT_DIVERGENCE_MODE",
  },
  { name: "syntax-highlighter", outcomeVariable: "VIZE_SYNTAX_HIGHLIGHTER_OUTCOME" },
  { name: "glyph", outcomeVariable: "VIZE_GLYPH_OUTCOME" },
  {
    name: "typecheck-divergence",
    outcomeVariable: "VIZE_TYPECHECK_DIVERGENCE_OUTCOME",
    modeVariable: "TYPECHECK_DIVERGENCE_MODE",
  },
];

function requiredValue(args: readonly string[], index: number, option: string): string {
  const value = args[index];
  if (value === undefined || value.startsWith("--")) throw new Error(`${option} requires a value`);
  return value;
}

function isSurfaceName(value: string): value is SurfaceName {
  return surfaceNames.some((name) => name === value);
}

function parseSurface(value: string): SurfaceResult {
  const separator = value.indexOf("=");
  const name = separator < 0 ? "" : value.slice(0, separator);
  const outcome = separator < 0 ? "" : value.slice(separator + 1);
  if (name.length === 0) throw new Error(`invalid --surface value: ${value}`);
  if (!isSurfaceName(name)) throw new Error(`unknown real-project surface: ${name}`);
  return {
    name,
    outcome: validateOutcome(name, outcome),
    rawOutcome: outcome,
    budgetMode: "enforce",
  };
}

function parseArguments(args: readonly string[]): Arguments {
  let output: string | undefined;
  let fromWorkflowEnvironment = false;
  const surfaces: SurfaceResult[] = [];
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "--from-workflow-env") fromWorkflowEnvironment = true;
    else if (argument === "--output") output = requiredValue(args, ++index, argument);
    else if (argument === "--surface")
      surfaces.push(parseSurface(requiredValue(args, ++index, argument)));
    else throw new Error(`unknown or incomplete argument: ${argument}`);
  }
  if (output === undefined) throw new Error("--output is required");
  if (fromWorkflowEnvironment && surfaces.length > 0)
    throw new Error("--from-workflow-env cannot be combined with --surface");
  return { output, fromWorkflowEnvironment, surfaces };
}

function validateOutcome(name: string, value: string): SurfaceOutcome {
  if (["success", "failure", "cancelled", "skipped"].includes(value))
    return value as SurfaceOutcome;
  throw new Error(`invalid outcome for ${name}: ${value}`);
}

function workflowResults(): SurfaceResult[] {
  return workflowSurfaces.map(({ name, outcomeVariable, modeVariable }) => {
    const rawOutcome = process.env[outcomeVariable] ?? "";
    const budgetMode = modeVariable === undefined ? "enforce" : (process.env[modeVariable] ?? "");
    const outcome =
      (budgetMode === "record-only" && rawOutcome === "failure") ||
      (budgetMode === "skip" && rawOutcome === "skipped")
        ? "success"
        : validateOutcome(name, rawOutcome);
    return { name, outcome, rawOutcome, budgetMode };
  });
}

function orderAndValidate(results: readonly SurfaceResult[]): SurfaceResult[] {
  const byName = new Map<SurfaceName, SurfaceResult>();
  for (const result of results) {
    if (byName.has(result.name)) throw new Error(`duplicate real-project surface: ${result.name}`);
    byName.set(result.name, result);
  }
  const missing = surfaceNames.filter((name) => !byName.has(name));
  if (missing.length > 0)
    throw new Error(`missing real-project surface verdict(s): ${missing.join(", ")}`);
  return surfaceNames.map((name) => byName.get(name)!);
}

function main(): number {
  const args = parseArguments(process.argv.slice(2));
  const surfaces = orderAndValidate(
    args.fromWorkflowEnvironment ? workflowResults() : args.surfaces,
  );
  const failedSurfaceNames = surfaces
    .filter(({ outcome }) => outcome !== "success")
    .map(({ name }) => name)
    .sort();
  const verdict = {
    schema: "vize.realProjectSurfaceVerdict",
    version: 2,
    sourceCommit: process.env.GITHUB_SHA,
    shardIndex: process.env.FIXTURE_SHARD_INDEX,
    status: failedSurfaceNames.length === 0 ? "success" : "failure",
    surfaces,
    failedSurfaceNames,
  };
  fs.writeFileSync(args.output, `${JSON.stringify(verdict, null, 2)}\n`);
  if (failedSurfaceNames.length > 0)
    throw new Error(`real-project surfaces failed: ${failedSurfaceNames.join(", ")}`);
  console.log(`all ${surfaces.length} real-project surfaces succeeded`);
  return 0;
}

runEntrypoint(main);
