import { spawn, spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { performance } from "node:perf_hooks";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: {
    runner: { type: "string", default: "nextest" },
    output: { type: "string", default: "target/nextest/full/workspace-timing.json" },
  },
});
if (!["cargo", "nextest"].includes(values.runner))
  throw new Error("runner must be cargo or nextest");
if (values.runner === "nextest") {
  const version = spawnSync("cargo", ["nextest", "--version"], { encoding: "utf8" });
  if (version.status !== 0 || !/^cargo-nextest 0\.9\.146(?:\s|$)/.test(version.stdout)) {
    throw new Error("full workspace tests require cargo-nextest 0.9.146");
  }
}

const phases = [];
const started = performance.now();
async function run(name, args) {
  const phaseStarted = performance.now();
  const exitCode = await new Promise((resolve) => {
    const child = spawn("cargo", args, { stdio: "inherit" });
    child.once("error", (error) => {
      console.error(error);
      resolve(1);
    });
    child.once("close", (code) => resolve(code ?? 1));
  });
  phases.push({
    name,
    args,
    durationMs: performance.now() - phaseStarted,
    exitCode,
    status: exitCode === 0 ? "passed" : "failed",
  });
  return exitCode;
}

let exitCode;
if (values.runner === "cargo") {
  exitCode = await run("workspace-and-doctests", ["test", "--locked", "--workspace"]);
} else {
  exitCode = await run("workspace", [
    "nextest",
    "run",
    "--locked",
    "--workspace",
    "--profile",
    "full",
  ]);
  const docExitCode = await run("doctests", ["test", "--locked", "--workspace", "--doc"]);
  exitCode ||= docExitCode;
}
mkdirSync(dirname(values.output), { recursive: true });
writeFileSync(
  values.output,
  `${JSON.stringify(
    {
      schema: "vize.rust.workspace-walltime",
      schemaVersion: 1,
      sourceSha: process.env.GITHUB_SHA ?? null,
      runner: values.runner,
      status: exitCode === 0 ? "passed" : "failed",
      durationMs: performance.now() - started,
      phases,
    },
    null,
    2,
  )}\n`,
);
process.exitCode = exitCode;
