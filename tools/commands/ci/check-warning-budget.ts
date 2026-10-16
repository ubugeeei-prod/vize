#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { stripVTControlCharacters } from "node:util";

const warningBudgetError = "JS/TS warning budget is 0 for v1 alpha CI";

function run(): number {
  const args = process.argv.slice(2);
  if (args[0] === "--") args.shift();
  const command = args.shift();
  if (!command) {
    console.error("Usage: node tools/commands/ci/check-warning-budget.ts -- <command> [args...]");
    return 2;
  }

  // cmd.exe resolves Windows .cmd package-manager shims. An explicit command
  // avoids Node's deprecated shell: true + argv combination.
  const executable = process.platform === "win32" ? (process.env.ComSpec ?? "cmd.exe") : command;
  const commandArgs =
    process.platform === "win32"
      ? ["/d", "/s", "/c", command.replaceAll("/", "\\"), ...args]
      : args;
  const result = spawnSync(executable, commandArgs, {
    // Match the previous wrapper's unbounded capture for large lint output.
    maxBuffer: Infinity,
  });
  if (result.error) {
    console.error(`failed to run ${command}: ${result.error.message}`);
    return 1;
  }
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  if (result.status !== 0) return result.status ?? 1;

  const output = stripVTControlCharacters(
    (result.stdout?.toString("utf8") ?? "") + (result.stderr?.toString("utf8") ?? ""),
  );
  const warningCount = output.split(/\r?\n/).reduce((count, line) => {
    const start = line.indexOf("Found ");
    if (start < 0) return count;
    const summary = line.slice(start);
    const match = /^Found \d+ errors? and (\d+) warnings?$/.exec(summary);
    return count + (match ? Number(match[1]) : 0);
  }, 0);
  if (warningCount > 0) {
    console.error(`${warningBudgetError}; found ${warningCount} warnings.`);
    return 1;
  }
  if (output.replace(/\s+/g, " ").includes("warn: Lint warnings found")) {
    console.error(`${warningBudgetError}; found unparsed warnings.`);
    return 1;
  }
  return 0;
}

process.exitCode = run();
