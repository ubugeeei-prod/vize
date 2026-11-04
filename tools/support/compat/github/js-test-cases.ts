/**
 * Count `node --test`, `tsx --test`, and `vp test run` while a package-test
 * command runs, and write one test case packet for the Check test report.
 * Plain scripts have no case list and are left out.
 */
import { spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import {
  accessSync,
  constants,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parseJson } from "../../json-value.ts";
import { packet, parsePacket, vitestCases, type TestCase } from "./test-cases.ts";

const reporter = fileURLToPath(new URL("./test-case-reporter.ts", import.meta.url));
const self = fileURLToPath(import.meta.url);

/** Repository root the package runners use, unless a test overrides it. */
export function jsPackageRoot(env: Readonly<Record<string, string | undefined>>): string {
  const root = env.VIZE_TEST_CASES_ROOT || fileURLToPath(new URL("../../../../", import.meta.url));
  return root.endsWith(path.sep) ? root.slice(0, -1) : root;
}

// `--test` only counts when it is a runner flag, not an argument of a script.
function testFlag(args: readonly string[]): number {
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--") return -1;
    if (arg === "--test") return index;
    if (!arg.startsWith("-")) return -1;
  }
  return -1;
}

/** Argv for a `node --test` or `tsx --test` process, plus this job's reporter. */
export function nodeTestArguments(
  args: readonly string[],
  reporterPath: string,
  destination: string,
): string[] | null {
  const index = testFlag(args);
  if (index < 0) return null;
  if (args.some((arg) => arg.includes("test-case-reporter.ts"))) return [...args];
  const extra: string[] = [];
  if (!args.some((arg) => arg === "--test-reporter" || arg.startsWith("--test-reporter=")))
    extra.push("--test-reporter=spec", "--test-reporter-destination=stdout");
  extra.push(`--test-reporter=${reporterPath}`, `--test-reporter-destination=${destination}`);
  return [...args.slice(0, index + 1), ...extra, ...args.slice(index + 1)];
}

/** Argv for `vp test run`, plus a JSON report beside the default reporter. */
export function vitestArguments(args: readonly string[], outputFile: string): string[] | null {
  if (args[0] !== "test" || args[1] !== "run") return null;
  if (args.some((arg) => arg.startsWith("--outputFile.json="))) return [...args];
  const extra = [
    ...(args.some((arg) => arg === "--reporter" || arg.startsWith("--reporter="))
      ? []
      : ["--reporter=default"]),
    "--reporter=json",
    `--outputFile.json=${outputFile}`,
  ];
  return [...args.slice(0, 2), ...extra, ...args.slice(2)];
}

// Node options that consume the following argv entry. `.bin` shims do not pass
// these; they matter when a script itself invokes `node --require …`.
const NODE_OPTIONS_WITH_VALUE = new Set([
  "-e",
  "--eval",
  "-p",
  "--print",
  "-r",
  "--require",
  "--import",
  "--loader",
  "--experimental-loader",
  "--test-reporter",
  "--test-reporter-destination",
]);

function splitNodeArgv(
  args: readonly string[],
): { nodeArgs: string[]; script: string; scriptArgs: string[] } | null {
  const nodeArgs: string[] = [];
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--") {
      const script = args[index + 1];
      if (!script) return null;
      return { nodeArgs, script, scriptArgs: args.slice(index + 2) };
    }
    if (arg === "--test") return null;
    if (arg.startsWith("-")) {
      nodeArgs.push(arg);
      const name = arg.includes("=") ? arg.slice(0, arg.indexOf("=")) : arg;
      if (!arg.includes("=") && NODE_OPTIONS_WITH_VALUE.has(name)) {
        const value = args[index + 1];
        if (value === undefined) return null;
        nodeArgs.push(value);
        index += 1;
      }
      continue;
    }
    return { nodeArgs, script: arg, scriptArgs: args.slice(index + 1) };
  }
  return null;
}

function isTsxCli(script: string): boolean {
  const base = path.basename(script);
  if (base === "tsx") return true;
  return base === "cli.mjs" && script.split("\\").join("/").includes("/tsx/");
}

/**
 * Argv for `node <vp-or-tsx> …` after a package script has prepended
 * `node_modules/.bin`. Those shims `exec node` on the real CLI, so PATH order
 * never sees `vp` or `tsx`. Returns null when this invocation is not a test.
 */
export function wrappedNodeArguments(
  args: readonly string[],
  reporterPath: string,
  nodeDestination: string,
  vitestOutput: string,
): string[] | null {
  const split = splitNodeArgv(args);
  if (!split) return null;
  const base = path.basename(split.script);
  const rewritten =
    base === "vp"
      ? vitestArguments(split.scriptArgs, vitestOutput)
      : isTsxCli(split.script)
        ? nodeTestArguments(split.scriptArgs, reporterPath, nodeDestination)
        : null;
  if (!rewritten) return null;
  if (
    rewritten.length === split.scriptArgs.length &&
    rewritten.every((arg, index) => arg === split.scriptArgs[index])
  )
    return null;
  return [...split.nodeArgs, split.script, ...rewritten];
}

function partialName(kind: "node" | "vitest"): string {
  return `${kind}-${process.pid}-${randomBytes(4).toString("hex")}.json`;
}

function lookup(name: string): string {
  const shim = process.env.VIZE_TEST_CASE_SHIM;
  if (!shim) throw new Error("VIZE_TEST_CASE_SHIM is required");
  const shimPath = path.resolve(shim);
  for (const dir of (process.env.PATH ?? "").split(path.delimiter)) {
    if (!dir || path.resolve(dir) === shimPath) continue;
    const candidate = path.join(dir, name);
    try {
      accessSync(candidate, constants.X_OK);
      return candidate;
    } catch {
      // The next PATH entry may have it.
    }
  }
  throw new Error(`Cannot find ${name} on PATH`);
}

function run(command: string, args: readonly string[]): number {
  const result = spawnSync(command, args, { env: process.env, stdio: "inherit" });
  if (result.error) throw result.error;
  return result.status ?? 1;
}

function execNode(args: readonly string[]): void {
  const partials = process.env.VIZE_TEST_CASE_PARTIALS;
  if (!partials) throw new Error("VIZE_TEST_CASE_PARTIALS is required");
  const destination = path.join(partials, partialName("node"));
  const rewritten = nodeTestArguments(args, reporter, destination);
  process.exitCode = run(process.env.VIZE_TEST_CASES_NODE || process.execPath, rewritten ?? args);
}

function execTsx(args: readonly string[]): void {
  const partials = process.env.VIZE_TEST_CASE_PARTIALS;
  if (!partials) throw new Error("VIZE_TEST_CASE_PARTIALS is required");
  const destination = path.join(partials, partialName("node"));
  const rewritten = nodeTestArguments(args, reporter, destination);
  process.exitCode = run(lookup("tsx"), rewritten ?? args);
}

function execVp(args: readonly string[]): void {
  const partials = process.env.VIZE_TEST_CASE_PARTIALS;
  if (!partials) throw new Error("VIZE_TEST_CASE_PARTIALS is required");
  const output = path.join(partials, partialName("vitest"));
  const rewritten = vitestArguments(args, output);
  process.exitCode = run(lookup("vp"), rewritten ?? args);
}

function execWrapped(args: readonly string[]): void {
  const partials = process.env.VIZE_TEST_CASE_PARTIALS;
  if (!partials) throw new Error("VIZE_TEST_CASE_PARTIALS is required");
  const rewritten = wrappedNodeArguments(
    args,
    reporter,
    path.join(partials, partialName("node")),
    path.join(partials, partialName("vitest")),
  );
  process.exitCode = run(process.env.VIZE_TEST_CASES_NODE || process.execPath, rewritten ?? args);
}

function writeMerged(partials: string, root: string): void {
  const out = process.env.VIZE_TEST_CASES;
  if (!out) throw new Error("VIZE_TEST_CASES is required");
  const cases: TestCase[] = [];
  for (const name of readdirSync(partials).sort()) {
    const file = path.join(partials, name);
    if (!name.endsWith(".json") || !statSync(file).isFile()) continue;
    const text = readFileSync(file, "utf8");
    if (!text.trim()) continue;
    if (name.startsWith("vitest-")) cases.push(...vitestCases(parseJson(text), root));
    else cases.push(...parsePacket(parseJson(text), file).cases);
  }
  writeFileSync(out, `${JSON.stringify(packet(process.env, cases))}\n`);
}

function writeShim(bin: string, name: string, body: string): void {
  writeFileSync(path.join(bin, name), body, { mode: 0o755 });
}

const reporterVariables = [
  "VIZE_TEST_CASES",
  "VIZE_TEST_CASES_ROOT",
  "VIZE_TEST_CASES_NODE",
  "VIZE_TEST_CASE_PARTIALS",
  "VIZE_TEST_CASE_SHIM",
  "VIZE_JS_TEST_CASES",
  "TEST_CASES_JOB",
  "GITHUB_RUN_ATTEMPT",
] as const;
type ReporterEnvironment = Readonly<Record<(typeof reporterVariables)[number], string>>;

function shim(tool: "node" | "tsx" | "vp", environment: ReporterEnvironment): string {
  // Cached tasks may filter their environment while retaining this PATH shim.
  // Bind only this capture's reporter transport, never the caller's other env.
  const restore = reporterVariables
    .map((name) => `export ${name}='${environment[name].replaceAll("'", "'\\''")}'`)
    .join("\n");
  const find = `
real=""
IFS=:
for dir in $PATH; do
  if [[ "$dir" == "$VIZE_TEST_CASE_SHIM" || -z "$dir" ]]; then continue; fi
  if [[ -x "$dir/${tool}" && ! -d "$dir/${tool}" ]]; then real="$dir/${tool}"; break; fi
done
unset IFS
if [[ -z "$real" ]]; then echo "${tool}: command not found" >&2; exit 127; fi
`;
  const inject = `
inject=0
for arg in "$@"; do
  if [[ "$arg" == "--" ]]; then break; fi
  if [[ "$arg" == "--test" ]]; then inject=1; break; fi
  if [[ "$arg" != -* ]]; then break; fi
done
`;
  if (tool === "node")
    return `#!/bin/bash
${restore}
# .bin/vp and .bin/tsx are shell shims that exec node on the real CLI, and
# package scripts prepend node_modules/.bin over this directory.
script=""
prev=""
for arg in "$@"; do
  if [[ "$arg" == "--" ]]; then break; fi
  if [[ -n "$prev" ]]; then prev=""; continue; fi
  case "$arg" in
    -e|--eval|-p|--print|-r|--require|--import|--loader|--experimental-loader|--test-reporter|--test-reporter-destination)
      prev=1
      continue
      ;;
  esac
  if [[ "$arg" == "--test" ]]; then
    exec "$VIZE_TEST_CASES_NODE" "$VIZE_JS_TEST_CASES" exec-node -- "$@"
  fi
  if [[ "$arg" == -* ]]; then continue; fi
  script="$arg"
  break
done
base="\${script##*/}"
if [[ "$base" == "vp" || "$base" == "tsx" || ( "$base" == "cli.mjs" && "$script" == *"/tsx/"* ) ]]; then
  exec "$VIZE_TEST_CASES_NODE" "$VIZE_JS_TEST_CASES" exec-wrapped -- "$@"
fi
exec "$VIZE_TEST_CASES_NODE" "$@"
`;
  if (tool === "tsx")
    return `#!/bin/bash
${restore}
${find}
${inject}
if [[ "$inject" == 1 ]]; then exec "$VIZE_TEST_CASES_NODE" "$VIZE_JS_TEST_CASES" exec-tsx -- "$@"; fi
exec "$real" "$@"
`;
  return `#!/bin/bash
${restore}
${find}
if [[ "\${1-}" == "test" && "\${2-}" == "run" ]]; then exec "$VIZE_TEST_CASES_NODE" "$VIZE_JS_TEST_CASES" exec-vp -- "$@"; fi
exec "$real" "$@"
`;
}

function capture(command: string, args: readonly string[]): void {
  const root = jsPackageRoot(process.env);
  const partials = mkdtempSync(path.join(tmpdir(), "vize-js-cases-"));
  const bin = path.join(partials, "bin");
  mkdirSync(bin);
  const reporterEnvironment: ReporterEnvironment = {
    VIZE_TEST_CASES: process.env.VIZE_TEST_CASES ?? "",
    VIZE_TEST_CASE_PARTIALS: partials,
    VIZE_TEST_CASE_SHIM: bin,
    VIZE_TEST_CASES_NODE: realpathSync(process.execPath),
    VIZE_TEST_CASES_ROOT: root,
    VIZE_JS_TEST_CASES: self,
    TEST_CASES_JOB: process.env.TEST_CASES_JOB ?? "",
    GITHUB_RUN_ATTEMPT: process.env.GITHUB_RUN_ATTEMPT ?? "",
  };
  for (const tool of ["node", "tsx", "vp"] as const)
    writeShim(bin, tool, shim(tool, reporterEnvironment));
  const env = {
    ...process.env,
    PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
    ...reporterEnvironment,
  };
  let status = 1;
  try {
    const result = spawnSync(command, args, { env, stdio: "inherit" });
    if (result.error) throw result.error;
    status = result.status ?? 1;
  } finally {
    try {
      writeMerged(partials, root);
    } finally {
      rmSync(partials, { recursive: true, force: true });
    }
  }
  process.exitCode = status;
}

if (process.argv[1] && path.resolve(process.argv[1]) === self) {
  const [mode, ...rest] = process.argv.slice(2);
  const separator = rest.indexOf("--");
  const args = separator < 0 ? [] : rest.slice(separator + 1);
  if (mode === "capture" && args[0]) capture(args[0], args.slice(1));
  else if (mode === "exec-node") execNode(args);
  else if (mode === "exec-tsx") execTsx(args);
  else if (mode === "exec-vp") execVp(args);
  else if (mode === "exec-wrapped") execWrapped(args);
  else throw new Error("Usage: js-test-cases.ts capture -- <command> [args...]");
}
