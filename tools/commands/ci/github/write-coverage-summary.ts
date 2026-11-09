import { spawnSync } from "node:child_process";
import { appendFileSync, writeFileSync } from "node:fs";

const coverageArguments = [
  "run",
  "--profile",
  "ci",
  "-p",
  "vize_test_runner",
  "--bin",
  "coverage",
] as const;

function lastLines(text: string, count: number): string {
  const trimmed = text.replace(/\n+$/, "");
  if (trimmed.length === 0) return "";
  return trimmed.split("\n").slice(-count).join("\n");
}

function coverageSummary(stdout: string): string {
  const summary = lastLines(stdout, 7);
  return summary.length === 0 ? "### Coverage Summary\n\n" : `### Coverage Summary\n${summary}\n`;
}

function main(): number {
  const summaryPath = process.env.GITHUB_STEP_SUMMARY;
  if (!summaryPath) {
    console.log("GITHUB_STEP_SUMMARY is not set");
    return 1;
  }

  const result = spawnSync(process.env.VIZE_COVERAGE_CARGO || "cargo", coverageArguments, {
    encoding: "utf8",
  });
  if (result.error) {
    console.error(`failed to run cargo: ${result.error.message}`);
    return 1;
  }

  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
  writeFileSync("coverage-report.txt", result.stdout);
  appendFileSync(summaryPath, coverageSummary(result.stdout));
  return result.status ?? 1;
}

process.exitCode = main();
