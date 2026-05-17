#!/usr/bin/env node
import { spawn } from "node:child_process";

const separatorIndex = process.argv.indexOf("--");
const command = separatorIndex === -1 ? undefined : process.argv[separatorIndex + 1];
const args = separatorIndex === -1 ? [] : process.argv.slice(separatorIndex + 2);

if (!command) {
  console.error("Usage: node tools/vite-plus/check-warning-budget.mjs -- <command> [args...]");
  process.exit(2);
}

let output = "";
const ansiPattern = new RegExp(`${String.fromCharCode(27)}\\[[0-9;]*m`, "g");
const child = spawn(command, args, {
  shell: false,
  stdio: ["inherit", "pipe", "pipe"],
});

const capture = (chunk, target) => {
  const text = chunk.toString();
  output += text.replace(ansiPattern, "");
  target.write(chunk);
};

child.stdout.on("data", (chunk) => capture(chunk, process.stdout));
child.stderr.on("data", (chunk) => capture(chunk, process.stderr));
child.on("error", (error) => {
  console.error(`Failed to start ${command}: ${error.message}`);
  process.exit(1);
});
child.on("close", (code, signal) => {
  if (signal) {
    console.error(`${command} exited from signal ${signal}`);
    process.exit(1);
  }

  if (code !== 0) {
    process.exit(code ?? 1);
  }

  const warningCount = Array.from(output.matchAll(/Found \d+ errors? and (\d+) warnings?/g))
    .map((match) => Number(match[1]))
    .reduce((total, count) => total + count, 0);

  if (warningCount > 0 || /\bwarn:\s+Lint warnings found\b/.test(output)) {
    console.error(
      `JS/TS warning budget is 0 for v1 alpha CI; found ${warningCount || "unparsed"} warnings.`,
    );
    process.exit(1);
  }
});
