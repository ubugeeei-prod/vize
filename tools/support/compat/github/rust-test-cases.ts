/** Stable-Rust bridge from cargo/libtest output to the shared Actions case packet. */
import { spawn } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { type Outcome, type TestCase, packet, parsePacket, workspaceCrates } from "./test-cases.ts";
import { parseJson } from "../../json-value.ts";

const ansi = /\x1b\[[0-?]*[ -/]*[@-~]/g;
const testLine = /^test (.+?) \.\.\. (ok|FAILED|ignored)(?:\b.*)?$/;

function ownerForTest(
  name: string,
  packages: readonly string[],
  crates: ReadonlyMap<string, string>,
): string {
  const normalized = name.split("\\").join("/");
  const documented = [...crates.values()]
    .sort((left, right) => right.length - left.length)
    .find((member) => normalized.startsWith(`${member}/`));
  if (documented) return documented;
  const owner = packages.length === 1 ? crates.get(packages[0]) : undefined;
  if (!owner) throw new Error(`Rust test ${name} does not identify one of: ${packages.join(", ")}`);
  return owner;
}

/** Parse stable libtest output, falling back to one command case per selected owner. */
export function rustOutputCases(
  output: string,
  packages: readonly string[],
  label: string,
  status: number,
  crates: ReadonlyMap<string, string>,
  paths: readonly string[] = [],
): TestCase[] {
  const cases: TestCase[] = [];
  for (const line of output.replace(ansi, "").split(/\r?\n/)) {
    const match = testLine.exec(line.trim());
    if (!match) continue;
    const [, name, result] = match;
    cases.push({
      path: ownerForTest(name, packages, crates),
      name: `${label} › ${name}`,
      outcome: result === "ok" ? "passed" : result === "ignored" ? "skipped" : "failed",
    });
  }
  const owners = [...packages.map((name) => crates.get(name)), ...paths];
  if (owners.some((owner) => !owner))
    throw new Error(`Unknown Rust package in ${packages.join(", ")}`);
  if (cases.length === 0) {
    if (owners.length === 0) throw new Error(`${label} has no package or path owner`);
    return owners.map((owner) => ({
      path: owner as string,
      name: owners.length === 1 ? label : `${label} › ${owner}`,
      outcome: status === 0 ? "passed" : "failed",
    }));
  }
  if (status !== 0 && cases.every(({ outcome }) => outcome !== "failed")) {
    cases.push({
      path: (owners[0] ?? cases[0].path) as string,
      name: `${label} › command completion`,
      outcome: "failed",
    });
  }
  return cases;
}

function appendCases(file: string, cases: readonly TestCase[], env = process.env): void {
  const next = packet(env, cases);
  const previous = existsSync(file)
    ? parsePacket(parseJson(readFileSync(file, "utf8")), file)
    : null;
  if (previous && (previous.job !== next.job || previous.attempt !== next.attempt))
    throw new Error(`${file} belongs to another job or run attempt`);
  writeFileSync(
    file,
    `${JSON.stringify({ ...next, cases: [...(previous?.cases ?? []), ...cases] })}\n`,
  );
}

export async function runRustCaseCommand(options: {
  readonly command: readonly string[];
  readonly packages?: readonly string[];
  readonly paths?: readonly string[];
  readonly label: string;
  readonly output: string;
  readonly env?: NodeJS.ProcessEnv;
}): Promise<number> {
  const [executable, ...args] = options.command;
  if (!executable) throw new Error("Rust case command is empty");
  const chunks: string[] = [];
  const status = await new Promise<number>((resolve) => {
    const child = spawn(executable, args, {
      env: options.env ?? process.env,
      shell: false,
      stdio: ["inherit", "pipe", "pipe"],
    });
    child.stdout.on("data", (chunk: Buffer) => {
      process.stdout.write(chunk);
      chunks.push(chunk.toString());
    });
    child.stderr.on("data", (chunk: Buffer) => {
      process.stderr.write(chunk);
      chunks.push(chunk.toString());
    });
    child.once("error", (error) => {
      const message = `${error.stack ?? error.message}\n`;
      process.stderr.write(message);
      chunks.push(message);
      resolve(1);
    });
    child.once("close", (code) => resolve(code ?? 1));
  });
  const root = fileURLToPath(new URL("../../../../", import.meta.url));
  appendCases(
    options.output,
    rustOutputCases(
      chunks.join(""),
      options.packages ?? [],
      options.label,
      status,
      workspaceCrates(root),
      options.paths,
    ),
  );
  return status;
}

function option(args: string[], name: string): string | undefined {
  const index = args.indexOf(name);
  if (index < 0) return undefined;
  const value = args[index + 1];
  if (!value) throw new Error(`${name} requires a value`);
  args.splice(index, 2);
  return value;
}

async function main(args = process.argv.slice(2)): Promise<void> {
  const mode = args.shift();
  const output = process.env.VIZE_TEST_CASES;
  if (!output) throw new Error("VIZE_TEST_CASES is required");
  const label = option(args, "--label");
  if (!label) throw new Error("--label is required");
  if (mode === "record") {
    const owner = option(args, "--path");
    const outcome = option(args, "--outcome") as Outcome | undefined;
    if (!owner || !outcome || !["passed", "failed", "skipped"].includes(outcome) || args.length)
      throw new Error(
        "Usage: rust-test-cases.ts record --label NAME --path PATH --outcome OUTCOME",
      );
    appendCases(output, [{ path: owner, name: label, outcome }]);
    return;
  }
  if (mode !== "capture")
    throw new Error(
      "Usage: rust-test-cases.ts capture --label NAME [--package PKG|--path PATH] -- COMMAND",
    );
  const packages: string[] = [];
  const paths: string[] = [];
  while (args.includes("--package")) packages.push(option(args, "--package") as string);
  while (args.includes("--path")) paths.push(option(args, "--path") as string);
  const separator = args.indexOf("--");
  if (separator !== 0 || args.length < 2) throw new Error("capture requires -- COMMAND");
  const status = await runRustCaseCommand({
    command: args.slice(1),
    packages,
    paths,
    label,
    output,
  });
  process.exitCode = status;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1]))
  await main();
