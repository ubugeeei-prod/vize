/** Capture test runners spread across multiple steps in one Actions job. */
import { spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { parseJson } from "../../json-value.ts";
import {
  nodeTestArguments,
  plainNodeTestCase,
  vitestArguments,
  wrappedNodeArguments,
} from "./js-test-cases.ts";
import { runRustCaseCommand } from "./rust-test-cases.ts";
import {
  mergePackets,
  packet,
  parsePacket,
  vitestCases,
  type TestCasePacket,
} from "./test-cases.ts";

const self = fileURLToPath(import.meta.url);
const reporter = fileURLToPath(new URL("./test-case-reporter.ts", import.meta.url));
const root = fileURLToPath(new URL("../../../../", import.meta.url));

function executable(name: string): string {
  for (const directory of (process.env.PATH ?? "").split(path.delimiter)) {
    const candidate = path.join(directory, name);
    if (existsSync(candidate)) return candidate;
  }
  throw new Error(`${name} is not available on PATH`);
}

function environment(name: string): string {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required`);
  return value;
}

function partial(kind: string): string {
  const directory = environment("VIZE_JOB_TEST_CASE_PARTIALS");
  return path.join(
    directory,
    `${kind}-${process.pid}-${randomBytes(5).toString("hex")}.packet.json`,
  );
}

function run(command: string, args: readonly string[]): number {
  const result = spawnSync(command, args, { env: process.env, stdio: "inherit" });
  if (result.error) throw result.error;
  return result.status ?? 1;
}

function writeVitestPacket(report: string, destination: string): void {
  if (!existsSync(report)) return;
  const cases = vitestCases(parseJson(readFileSync(report, "utf8")), root);
  writeFileSync(destination, `${JSON.stringify(packet(process.env, cases))}\n`);
}

function execNode(args: readonly string[]): void {
  const destination = partial("node");
  const vitestReport = destination.replace(".packet.json", ".vitest.json");
  const rewritten =
    nodeTestArguments(args, reporter, destination) ??
    wrappedNodeArguments(args, reporter, destination, vitestReport);
  const status = run(environment("VIZE_JOB_TEST_CASE_NODE"), rewritten ?? args);
  writeVitestPacket(vitestReport, destination);
  if (!rewritten) {
    const plain = plainNodeTestCase(args, root, status);
    if (plain) writeFileSync(destination, `${JSON.stringify(packet(process.env, [plain]))}\n`);
  }
  process.exitCode = status;
}

function execVp(args: readonly string[]): void {
  const destination = partial("vp");
  const report = destination.replace(".packet.json", ".vitest.json");
  let rewritten: string[] | null = null;
  if (args[0] === "node") {
    const nodeArgs = nodeTestArguments(args.slice(1), reporter, destination);
    if (nodeArgs) rewritten = ["node", ...nodeArgs];
  } else {
    rewritten = vitestArguments(args, report);
  }
  const status = run(environment("VIZE_JOB_TEST_CASE_VP"), rewritten ?? args);
  writeVitestPacket(report, destination);
  if (!rewritten && args[0] === "node") {
    const plain = plainNodeTestCase(args.slice(1), root, status);
    if (plain) writeFileSync(destination, `${JSON.stringify(packet(process.env, [plain]))}\n`);
  }
  process.exitCode = status;
}

/** Package owners selected by a cargo test invocation. */
export function cargoTestPackages(args: readonly string[]): string[] | null {
  if (args[0] !== "test") return null;
  const packages: string[] = [];
  for (let index = 1; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "-p" || argument === "--package") {
      const value = args[index + 1];
      if (!value) throw new Error(`${argument} requires a package`);
      packages.push(value);
      index += 1;
    } else if (argument.startsWith("--package=")) packages.push(argument.slice(10));
    else if (/^-p[^-]/u.test(argument)) packages.push(argument.slice(2));
  }
  return [...new Set(packages)];
}

async function execCargo(args: readonly string[]): Promise<void> {
  const packages = cargoTestPackages(args);
  if (!packages) {
    process.exitCode = run(environment("VIZE_JOB_TEST_CASE_CARGO"), args);
    return;
  }
  const owner = packages.length ? packages.join(", ") : "workspace";
  process.exitCode = await runRustCaseCommand({
    command: [environment("VIZE_JOB_TEST_CASE_CARGO"), ...args],
    packages,
    paths: packages.length ? [] : ["Cargo.toml"],
    label: `Native runtime cargo test (${owner})`,
    output: partial("cargo"),
  });
}

function shell(tool: "node" | "vp" | "cargo"): string {
  return `#!/bin/bash\nexec "$VIZE_JOB_TEST_CASE_NODE" "$VIZE_JOB_TEST_CASE_TOOL" exec-${tool} -- "$@"\n`;
}

function setup(directory: string): void {
  const partials = path.join(path.resolve(directory), "partials");
  const bin = path.join(path.resolve(directory), "bin");
  mkdirSync(partials, { recursive: true });
  mkdirSync(bin, { recursive: true });
  const tools = { node: process.execPath, vp: executable("vp"), cargo: executable("cargo") };
  for (const name of Object.keys(tools) as Array<keyof typeof tools>)
    writeFileSync(path.join(bin, name), shell(name), { mode: 0o755 });
  const githubEnvironment = environment("GITHUB_ENV");
  const values = {
    VIZE_JOB_TEST_CASE_PARTIALS: partials,
    VIZE_JOB_TEST_CASE_TOOL: self,
    VIZE_JOB_TEST_CASE_NODE: tools.node,
    VIZE_JOB_TEST_CASE_VP: tools.vp,
    VIZE_JOB_TEST_CASE_CARGO: tools.cargo,
    VIZE_TEST_CASES_ROOT: root,
  };
  for (const [name, value] of Object.entries(values))
    appendFileSync(githubEnvironment, `${name}=${value}\n`);
  appendFileSync(environment("GITHUB_PATH"), `${bin}\n`);
}

function finalize(): void {
  const directory = environment("VIZE_JOB_TEST_CASE_PARTIALS");
  const reports = readdirSync(directory)
    .filter((name) => name.endsWith(".packet.json"))
    .sort()
    .map((name) => {
      const file = path.join(directory, name);
      return parsePacket(parseJson(readFileSync(file, "utf8")), file);
    });
  if (!reports.length) throw new Error("The test job produced no test case packets");
  const merged: TestCasePacket = mergePackets(reports);
  writeFileSync(environment("VIZE_TEST_CASES"), `${JSON.stringify(merged)}\n`);
  console.log(`Recorded ${merged.cases.length} test cases from ${reports.length} commands`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === self) {
  const [mode, separator, ...args] = process.argv.slice(2);
  if (mode === "setup" && separator) setup(separator);
  else if (mode === "finalize" && separator === undefined) finalize();
  else if (separator === "--" && mode === "exec-node") execNode(args);
  else if (separator === "--" && mode === "exec-vp") execVp(args);
  else if (separator === "--" && mode === "exec-cargo") await execCargo(args);
  else throw new Error("Usage: job-test-cases.ts <setup DIR|finalize|exec-TYPE -- ARGS>");
}
