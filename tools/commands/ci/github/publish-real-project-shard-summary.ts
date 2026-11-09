import { spawnSync } from "node:child_process";
import { appendFileSync, existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";

type JsonObject = { readonly [key: string]: unknown };
type Renderer = (value: unknown) => string;

function isRecord(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function jsonAt(value: unknown, pointer: string): unknown {
  let current = value;
  for (const segment of pointer.split("/").slice(1)) {
    if (!isRecord(current) || !(segment in current)) {
      throw new Error(`missing JSON field ${pointer}`);
    }
    current = current[segment];
  }
  return current;
}

function numberAt(value: unknown, pointer: string): number {
  const field = jsonAt(value, pointer);
  if (typeof field !== "number" || !Number.isSafeInteger(field)) {
    throw new Error(`JSON field ${pointer} must be a number`);
  }
  return field;
}

function stringAt(value: unknown, pointer: string): string {
  const field = jsonAt(value, pointer);
  return typeof field === "string" ? field : "";
}

function stringArrayAt(value: unknown, pointer: string): string[] {
  const field = jsonAt(value, pointer);
  if (!Array.isArray(field)) throw new Error(`JSON field ${pointer} must be an array`);
  return field.map((entry) => (typeof entry === "string" ? entry : ""));
}

function isFile(file: string): boolean {
  try {
    return statSync(file).isFile();
  } catch {
    return false;
  }
}

function isNonemptyFile(file: string): boolean {
  try {
    const metadata = statSync(file);
    return metadata.isFile() && metadata.size > 0;
  } catch {
    return false;
  }
}

function appendLine(output: string, line: string): void {
  appendFileSync(output, `${line}\n`);
}

function appendFileOrLine(output: string, file: string, fallback: string): void {
  if (isFile(file)) appendFileSync(output, readFileSync(file));
  else appendLine(output, fallback);
}

function readJson(file: string): unknown {
  return JSON.parse(readFileSync(file, "utf8"));
}

function appendJsonLineOr(
  output: string,
  file: string,
  fallback: string,
  render: Renderer,
  requireNonempty: boolean,
): void {
  const usable = requireNonempty ? isNonemptyFile(file) : isFile(file);
  appendLine(output, usable ? render(readJson(file)) : fallback);
}

function sortedMatches(directory: string, suffix: string): string[] {
  if (!existsSync(directory)) return [];
  return readdirSync(directory)
    .filter((name) => name.endsWith(suffix))
    .map((name) => path.join(directory, name))
    .filter(isFile)
    .sort();
}

function lspSummary(value: unknown): string {
  return (
    `LSP lifecycle: ${numberAt(value, "/summary/projectCount")} project(s), ` +
    `${numberAt(value, "/summary/actualFileCount")} actual file(s), ` +
    `${numberAt(value, "/summary/authoredFeatureProjectCount")} authored feature oracle(s), ` +
    `${numberAt(value, "/summary/authoredAnchorCount")} authored anchor(s), ` +
    `${numberAt(value, "/summary/vueFileCount")} Vue file(s), ` +
    `${numberAt(value, "/summary/failedProjectCount")} failed project(s); missing authored oracles: ` +
    stringArrayAt(value, "/summary/missingAuthoredFeatureProjectIds").join(", ")
  );
}

function syntaxSummary(value: unknown): string {
  return (
    `Syntax highlighter: ${numberAt(value, "/summary/projectCount")} project(s), ` +
    `${numberAt(value, "/summary/fileCount")} file(s), ` +
    `${numberAt(value, "/summary/lineCount")} line(s), ` +
    `${numberAt(value, "/summary/failedProjectCount")} failed project(s)`
  );
}

function lintSummary(value: unknown): string {
  return (
    `Lint divergence: ${numberAt(value, "/projectCount")} project(s), ` +
    `${numberAt(value, "/totals/sharedCount")} shared, ` +
    `${numberAt(value, "/totals/falsePositiveCount")} false positive(s), ` +
    `${numberAt(value, "/totals/falseNegativeCount")} false negative(s), ` +
    `${numberAt(value, "/totals/patinaOnlyRuleFindingCount")} patina-only finding(s)`
  );
}

function waiverSummary(value: unknown): string {
  const issues = jsonAt(value, "/issues");
  if (!Array.isArray(issues)) throw new Error("JSON field /issues must be an array");
  return `Formatter waivers: ${numberAt(value, "/waiverCount")} precise waiver(s), ${issues.length} open owner Issue(s)`;
}

function surfaceSummary(value: unknown): string {
  return `Surface verdict: ${stringAt(value, "/status")}; failed: ${stringArrayAt(value, "/failedSurfaceNames").join(", ")}`;
}

function publish(reportDirectory: string, summaryPath: string): void {
  const report = (name: string) => path.join(reportDirectory, name);
  appendFileOrLine(summaryPath, report("summary.md"), "No fixture tool report was produced.");
  appendJsonLineOr(
    summaryPath,
    report("lsp-lifecycle-summary.json"),
    "No LSP lifecycle report was produced.",
    lspSummary,
    true,
  );
  appendJsonLineOr(
    summaryPath,
    report("syntax-highlighter-summary.json"),
    "No syntax-highlighter report was produced.",
    syntaxSummary,
    false,
  );

  const lint = report("lint-divergence-summary.json");
  if (isNonemptyFile(lint)) {
    appendLine(summaryPath, lintSummary(readJson(lint)));
    for (const file of sortedMatches(reportDirectory, "-lint-divergence.md")) {
      if (isNonemptyFile(file)) appendFileSync(summaryPath, readFileSync(file));
    }
  } else {
    appendLine(summaryPath, "No lint divergence report was produced.");
  }

  const typecheckReports = sortedMatches(reportDirectory, "-typecheck-divergence.md");
  if (typecheckReports.length === 1) appendFileSync(summaryPath, readFileSync(typecheckReports[0]));
  else appendLine(summaryPath, "No unique typecheck divergence report was produced.");

  appendJsonLineOr(
    summaryPath,
    report("glyph-waiver-issues.json"),
    "No formatter waiver owner report was produced.",
    waiverSummary,
    true,
  );
  appendJsonLineOr(
    summaryPath,
    report("surface-verdict.json"),
    "No real-project surface verdict was produced.",
    surfaceSummary,
    true,
  );
}

function dehydrateSelectedFixtureShard(reportDirectory: string): void {
  const selectedPath = path.join(reportDirectory, "selected-fixtures.txt");
  if (!isNonemptyFile(selectedPath)) return;
  const fixtures = readFileSync(selectedPath, "utf8")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
  if (fixtures.length === 0) return;

  const result = spawnSync("git", ["submodule", "deinit", "--force", "--", ...fixtures], {
    stdio: "inherit",
  });
  if (result.error) throw new Error(`failed to run git submodule deinit: ${result.error.message}`);
  if (result.status !== 0)
    throw new Error(`git submodule deinit failed with exit code ${result.status ?? 1}`);
}

function main(): number {
  const errors: Error[] = [];
  const reportDirectory = process.env.FIXTURE_REPORT_DIR;
  const summaryPath = process.env.GITHUB_STEP_SUMMARY;

  if (process.argv.length > 2) {
    errors.push(
      new Error(
        `Usage: node tools/commands/ci/github/publish-real-project-shard-summary.ts; unexpected argument ${process.argv[2]}`,
      ),
    );
  } else if (!reportDirectory || !summaryPath) {
    errors.push(
      new Error(
        !reportDirectory ? "FIXTURE_REPORT_DIR is required" : "GITHUB_STEP_SUMMARY is required",
      ),
    );
  } else {
    try {
      publish(reportDirectory, summaryPath);
    } catch (error) {
      errors.push(error instanceof Error ? error : new Error(String(error)));
    }
  }

  if (reportDirectory) {
    try {
      dehydrateSelectedFixtureShard(reportDirectory);
    } catch (error) {
      errors.push(error instanceof Error ? error : new Error(String(error)));
    }
  }
  for (const error of errors) console.error(error.message);
  return errors.length === 0 ? 0 : 1;
}

process.exitCode = main();
