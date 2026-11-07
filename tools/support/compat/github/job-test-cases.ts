/** Capture test runners spread across multiple steps in one Actions job. */
import { spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
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
  ownedPath,
  packet,
  parsePacket,
  vitestCases,
  type TestCase,
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

function hasContent(file: string): boolean {
  return existsSync(file) && statSync(file).size > 0;
}

function writeVitestPacket(report: string, destination: string): boolean {
  if (!hasContent(report)) return false;
  const cases = vitestCases(parseJson(readFileSync(report, "utf8")), root);
  writeFileSync(destination, `${JSON.stringify(packet(process.env, cases))}\n`);
  return true;
}

function commandOwner(args: readonly string[]): string {
  const candidate = args.find((argument) =>
    /(?:^|\/)[^/]+\.(?:test|spec)\.[cm]?[jt]sx?$/u.test(argument.split("\\").join("/")),
  );
  return candidate ? ownedPath(path.resolve(process.cwd(), candidate), root) : "package.json";
}

function writeFallbackPacket(
  destination: string,
  args: readonly string[],
  status: number,
  label: string,
): void {
  if (hasContent(destination)) return;
  const plain = plainNodeTestCase(args, root, status);
  const owner = plain?.path ?? commandOwner(args);
  const cases: TestCase[] = [
    plain ?? {
      path: owner,
      name: `${owner} › ${label}`,
      outcome: status === 0 ? "passed" : "failed",
    },
  ];
  writeFileSync(destination, `${JSON.stringify(packet(process.env, cases))}\n`);
}

function execNode(args: readonly string[]): void {
  const destination = partial("node");
  const vitestReport = destination.replace(".packet.json", ".vitest.json");
  const rewritten =
    nodeTestArguments(args, reporter, destination) ??
    wrappedNodeArguments(args, reporter, destination, vitestReport);
  const status = run(environment("VIZE_JOB_TEST_CASE_NODE"), rewritten ?? args);
  const wroteVitest = writeVitestPacket(vitestReport, destination);
  if (rewritten || plainNodeTestCase(args, root, status))
    if (!wroteVitest) writeFallbackPacket(destination, args, status, "test command");
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
  const wroteVitest = writeVitestPacket(report, destination);
  const nodeArgs = args[0] === "node" ? args.slice(1) : args;
  if (rewritten || (args[0] === "node" && plainNodeTestCase(nodeArgs, root, status)))
    if (!wroteVitest) writeFallbackPacket(destination, nodeArgs, status, "test command");
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

function shellValue(value: string): string {
  return `'${value.replaceAll("'", `'"'"'`)}'`;
}

type CaptureEnvironment = Readonly<Record<string, string>>;

/** A persistent job shim that remains valid through task-runner env filtering. */
export function jobTestShell(tool: "node" | "vp" | "cargo", values: CaptureEnvironment): string {
  const restore = Object.entries(values)
    .map(([name, value]) => `export ${name}=${shellValue(value)}`)
    .join("\n");
  const capture = `exec "$VIZE_JOB_TEST_CASE_NODE" "$VIZE_JOB_TEST_CASE_TOOL" exec-${tool} -- "$@"`;
  if (tool === "cargo")
    return `#!/bin/bash\n${restore}\nif [[ "\${1-}" == "test" ]]; then ${capture}; fi\nexec "$VIZE_JOB_TEST_CASE_CARGO" "$@"\n`;
  if (tool === "vp")
    return `#!/bin/bash\n${restore}\nif [[ "\${1-}" == "test" && "\${2-}" == "run" ]]; then ${capture}; fi\nexec "$VIZE_JOB_TEST_CASE_VP" "$@"\n`;
  return `#!/bin/bash
${restore}
script=""
previous=""
after=()
for argument in "$@"; do
  if [[ -n "$previous" ]]; then previous=""; continue; fi
  if [[ -z "$script" ]]; then
    case "$argument" in
      -e|--eval|-p|--print|-r|--require|--import|--loader|--experimental-loader|--test-reporter|--test-reporter-destination)
        previous=1
        continue
        ;;
    esac
    if [[ "$argument" == "--test" ]]; then ${capture}; fi
    if [[ "$argument" == -* ]]; then continue; fi
    script="$argument"
    continue
  fi
  after+=("$argument")
done
normalized="\${script//\\\\//}"
base="\${normalized##*/}"
capture_script=0
if [[ "$normalized" != */node_modules/* ]]; then
  if [[ "$base" =~ \\.(test|spec)\\.[cm]?[jt]sx?$ || "$base" =~ ^(test|tests|run-tests|run-host|host)\\.[cm]?[jt]sx?$ || "$base" =~ -judge\\.[cm]?[jt]sx?$ || "$normalized" == */test/* || "$normalized" == */tests/* || "$normalized" == */support/compat/parcel/run.ts ]]; then
    capture_script=1
  fi
fi
if [[ "$base" == "vp" && "\${after[0]-}" == "test" && "\${after[1]-}" == "run" ]]; then capture_script=1; fi
if [[ ( "$base" == "tsx" || ( "$base" == "cli.mjs" && "$normalized" == */tsx/* ) ) && " \${after[*]-} " == *" --test "* ]]; then capture_script=1; fi
if [[ "$capture_script" == 1 ]]; then ${capture}; fi
exec "$VIZE_JOB_TEST_CASE_NODE" "$@"
`;
}

function setup(directory: string): void {
  const partials = path.join(path.resolve(directory), "partials");
  const bin = path.join(path.resolve(directory), "bin");
  mkdirSync(partials, { recursive: true });
  mkdirSync(bin, { recursive: true });
  const tools = { node: process.execPath, vp: executable("vp"), cargo: executable("cargo") };
  const githubEnvironment = environment("GITHUB_ENV");
  const values = {
    VIZE_JOB_TEST_CASE_PARTIALS: partials,
    VIZE_JOB_TEST_CASE_TOOL: self,
    VIZE_JOB_TEST_CASE_NODE: tools.node,
    VIZE_JOB_TEST_CASE_VP: tools.vp,
    VIZE_JOB_TEST_CASE_CARGO: tools.cargo,
    VIZE_TEST_CASES_ROOT: root,
  };
  for (const name of Object.keys(tools) as Array<keyof typeof tools>)
    writeFileSync(path.join(bin, name), jobTestShell(name, values), { mode: 0o755 });
  for (const [name, value] of Object.entries(values))
    appendFileSync(githubEnvironment, `${name}=${value}\n`);
  appendFileSync(environment("GITHUB_PATH"), `${bin}\n`);
}

function finalize(): void {
  const directory = environment("VIZE_JOB_TEST_CASE_PARTIALS");
  const reports = readdirSync(directory)
    .filter((name) => name.endsWith(".packet.json") && statSync(path.join(directory, name)).size)
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
