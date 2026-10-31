import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { mkdir, open } from "node:fs/promises";
import path from "node:path";
import { inspectUiCores } from "./ui-stack-core.ts";

const source = path.resolve(process.argv[2] ?? "");
const output = path.resolve(process.argv[3] ?? "");
const scratch = path.resolve(process.argv[4] ?? "");
assert.equal(process.platform, "linux");
assert.equal(process.env.GITHUB_ACTIONS, "true");
assert.notEqual(source, output);
assert.notEqual(scratch, output);
await mkdir(output, { recursive: true });
await mkdir(scratch, { recursive: true });
const digest = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
function command(program: string, args: readonly string[], cwd = source): string {
  const result = spawnSync(program, args, { cwd, encoding: "utf8", maxBuffer: 8 * 1024 * 1024 });
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, `${program}: ${result.stderr}`);
  return result.stdout.trim();
}
const git = (...args: string[]): string => command("git", args);
const expectedSource = "86fabe7388b45ac4ab7d938f910b7e1add6b9d7a";
const expectedTree = "a53037dc37df5558bc81eb07458529bf5a4800ae";
assert.equal(git("rev-parse", "HEAD"), expectedSource);
assert.equal(git("rev-parse", "HEAD^{tree}"), expectedTree);
assert.equal(
  git("rev-list", "--parents", "-n", "1", "HEAD"),
  `${expectedSource} be8aa8108652c10b7eb02d963e8862fcae6b542c 564abf0290fd730b5a4df78c2c4baa9d80db5e59`,
);
assert.equal(git("diff", "--binary", "HEAD"), "");
// Do not change any stack allocation budget or worker count for this observation.
for (const variable of ["RUST_MIN_STACK", "TOKIO_WORKER_THREADS"])
  assert.equal(process.env[variable], undefined, `${variable} must remain unset`);
assert.equal(process.env.CARGO_INCREMENTAL, "0");
assert.equal(process.env.CARGO_BUILD_JOBS, "12");
const locks = Object.fromEntries(
  ["Cargo.lock", "pnpm-lock.yaml"].map((name) => [
    name,
    digest(readFileSync(path.join(source, name))),
  ]),
);
const toolNames = ["vp", "pnpm", "node", "cargo", "rustc", "strace", "gdb"];
const tools = Object.fromEntries(
  toolNames.map((name) => {
    const filename = command("bash", ["-c", 'command -v "$1"', "tool", name]);
    return [name, { filename, sha256: digest(readFileSync(filename)) }];
  }),
);
const originalCommand =
  "vp run --filter './npm/native' build:ci && vp run --filter './npm/ui' check";
const observedEnvironment: NodeJS.ProcessEnv = { ...process.env, RUST_BACKTRACE: "full" };
// These belong to the reused driver workflow and were absent from the original UI step.
for (const name of [
  "VIZE_CSS_MAP_SOURCE_SHA",
  "VIZE_CSS_MAP_PR_HEAD",
  "VIZE_CSS_MAP_REQUIRE_SOURCE",
])
  delete observedEnvironment[name];
const sourceIdentity = {
  schema: "ui-stack-observer-v1",
  source: expectedSource,
  tree: expectedTree,
  originalFailedRun: 41183185585,
  originalFailedJob: 124484773805,
  driver: process.env.GITHUB_WORKFLOW_SHA,
  run: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT,
  locks,
  originalCommand,
  tools,
  inheritedLibuvThreadpool: process.env.UV_THREADPOOL_SIZE ?? null,
  historicalLibuvThreadpool:
    "not recorded in original failure; inherited value is preserved without modification",
  versions: Object.fromEntries(toolNames.map((name) => [name, command(name, ["--version"])])),
  stackLimits: readFileSync("/proc/self/limits", "utf8")
    .split("\n")
    .filter((line) => line.startsWith("Max stack size")),
  observer:
    "process-only strace, core enabled with 4GiB per-core ceiling, RUST_BACKTRACE=full; no stack/worker override",
};
writeFileSync(path.join(output, "source.json"), JSON.stringify(sourceIdentity, null, 2) + "\n");
const stdout = await open(path.join(output, "command.stdout"), "w");
const stderr = await open(path.join(output, "command.stderr"), "w");
const start = new Date().toISOString();
const result = await new Promise<{
  status: number | null;
  signal: NodeJS.Signals | null;
  error: string | null;
}>((resolve) => {
  const child = spawn(
    "strace",
    [
      "-f",
      "-tt",
      "-e",
      "trace=process",
      "-o",
      path.join(output, "processes.strace"),
      "bash",
      "--noprofile",
      "--norc",
      "-c",
      originalCommand,
    ],
    {
      cwd: source,
      env: observedEnvironment,
      stdio: ["ignore", stdout.fd, stderr.fd],
    },
  );
  let error: string | null = null;
  child.on("error", (failure) => {
    error = failure.message;
  });
  child.on("close", (status, signal) => resolve({ status, signal, error }));
});
await stdout.close();
await stderr.close();
async function fileIdentity(filename: string): Promise<{ bytes: number; sha256: string }> {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(filename)) hash.update(chunk);
  return { bytes: statSync(filename).size, sha256: hash.digest("hex") };
}
const native = await Promise.all(
  readdirSync(path.join(source, "npm/native"))
    .filter((name) => name.endsWith(".node"))
    .map(async (name) => ({
      name,
      ...(await fileIdentity(path.join(source, "npm/native", name))),
    })),
);
const coreResults = await inspectUiCores(scratch, output);
const capture = {
  ...result,
  start,
  finish: new Date().toISOString(),
  native,
  coreResults,
  stdout: await fileIdentity(path.join(output, "command.stdout")),
  stderr: await fileIdentity(path.join(output, "command.stderr")),
  processTrace: await fileIdentity(path.join(output, "processes.strace")),
  sourceDiffAfter: git("diff", "--name-only", "HEAD"),
  classification:
    result.status === 0
      ? "original symptom not reproduced; no repair claim"
      : "original command failed; inspect actual process/core evidence",
};
writeFileSync(path.join(output, "result.json"), JSON.stringify(capture, null, 2) + "\n");
console.log(JSON.stringify(capture, null, 2));
for (const filename of ["command.stdout", "command.stderr"]) {
  const bytes = readFileSync(path.join(output, filename));
  console.log(
    `${filename}: last 16000 bytes\n${bytes.subarray(Math.max(0, bytes.length - 16000)).toString("utf8")}`,
  );
}
process.exitCode = result.error === null && result.status === 0 ? 0 : 1;
