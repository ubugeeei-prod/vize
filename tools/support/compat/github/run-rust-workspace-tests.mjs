import { spawn, spawnSync } from "node:child_process";
import {
  closeSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  openSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { performance } from "node:perf_hooks";
import { parseArgs } from "node:util";
import {
  digest,
  normalizeWorkspaceInventory,
  workspaceFingerprint,
} from "./rust-workspace-inventory.mjs";

const { values } = parseArgs({
  options: {
    runner: { type: "string", default: "nextest" },
    output: { type: "string", default: "target/nextest/full/workspace-timing.json" },
    "cargo-profile": { type: "string", default: "test" },
    "inventory-only": { type: "boolean", default: false },
  },
});
if (!["cargo", "nextest"].includes(values.runner))
  throw new Error("runner must be cargo or nextest");
if (!["test", "ci"].includes(values["cargo-profile"]))
  throw new Error("cargo profile must be test or ci");
if (values.runner === "cargo" && values["inventory-only"])
  throw new Error("inventory requires nextest");
if (values.runner === "nextest") {
  const version = spawnSync("cargo", ["nextest", "--version"], { encoding: "utf8" });
  if (version.status !== 0 || !/^cargo-nextest 0\.9\.146(?:\s|$)/.test(version.stdout)) {
    throw new Error("full workspace tests require cargo-nextest 0.9.146");
  }
}
const source = workspaceFingerprint();
const cargoProfile = values["cargo-profile"];
const profile = ["--profile", cargoProfile];
const directory = dirname(values.output);
mkdirSync(directory, { recursive: true });

const phases = [];
const started = performance.now();
async function run(name, args, output) {
  const phaseStarted = performance.now();
  const exitCode = await new Promise((resolve) => {
    const descriptor = output ? openSync(output, "w") : null;
    const child = spawn("cargo", args, {
      stdio: descriptor === null ? "inherit" : ["inherit", descriptor, "inherit"],
    });
    let closed = false;
    const finish = (code) => {
      if (closed) return;
      closed = true;
      if (descriptor !== null) closeSync(descriptor);
      resolve(code);
    };
    child.once("error", (error) => {
      console.error(error);
      finish(1);
    });
    child.once("close", (code) => finish(code ?? 1));
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

let exitCode = 0;
let inventoryError = null;
if (values.runner === "cargo") {
  exitCode = await run("workspace-and-doctests", ["test", "--locked", "--workspace", ...profile]);
} else {
  const rawFile = `workspace-inventory-${cargoProfile}.raw.json`;
  exitCode = await run(
    "inventory",
    [
      "nextest",
      "list",
      "--locked",
      "--workspace",
      "--cargo-profile",
      cargoProfile,
      "--profile",
      "full",
      "--run-ignored",
      "all",
      "--message-format",
      "json",
    ],
    join(directory, rawFile),
  );
  if (exitCode === 0) {
    try {
      const raw = readFileSync(join(directory, rawFile));
      const inventory = {
        schema: "vize.rust.workspace-inventory",
        schemaVersion: 1,
        source,
        cargoProfile,
        rawFile,
        rawSha256: digest(raw),
        binaries: normalizeWorkspaceInventory(JSON.parse(raw)),
      };
      writeFileSync(
        join(directory, `workspace-inventory-${cargoProfile}.json`),
        `${JSON.stringify(inventory, null, 2)}\n`,
      );
    } catch (error) {
      inventoryError = error.message;
      console.error(error);
      exitCode = 1;
    }
  }
  if (exitCode === 0 && !values["inventory-only"])
    exitCode = await run("workspace", [
      "nextest",
      "run",
      "--locked",
      "--workspace",
      "--cargo-profile",
      cargoProfile,
      "--profile",
      "full",
    ]);
  if (!values["inventory-only"]) {
    const docExitCode = await run("doctests", [
      "test",
      "--locked",
      "--workspace",
      ...profile,
      "--doc",
    ]);
    exitCode ||= docExitCode;
  }
}
if (
  values.runner === "nextest" &&
  !values["inventory-only"] &&
  existsSync("target/nextest/full/junit.xml")
)
  copyFileSync(
    "target/nextest/full/junit.xml",
    join(directory, `workspace-junit-${cargoProfile}.xml`),
  );
mkdirSync(dirname(values.output), { recursive: true });
writeFileSync(
  values.output,
  `${JSON.stringify(
    {
      schema: "vize.rust.workspace-walltime",
      schemaVersion: 2,
      sourceSha: source.sha,
      source,
      cargoProfile,
      inventoryOnly: values["inventory-only"],
      inventoryError,
      execution: {
        runId: process.env.GITHUB_RUN_ID ?? null,
        attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
      },
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
