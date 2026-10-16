import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";

const root = process.cwd();
const binary = process.env.ACTIONLINT_BINARY ?? process.argv[2];
assert.ok(binary, "a verified official actionlint binary is required");
const output = path.join(root, "target/workflow-syntax");
mkdirSync(output, { recursive: true });
type Diagnostic = {
  filepath: string;
  line: number;
  column: number;
  kind: string;
  message: string;
};
const revision = (field: string) => {
  const result = spawnSync("git", ["rev-parse", field], { encoding: "utf8", timeout: 10_000 });
  assert.equal(result.status, 0, "actual checked-out source identity is required");
  const value = result.stdout.trim();
  assert.match(value, /^[0-9a-f]{40}$/);
  return value;
};
const source = { head: revision("HEAD"), tree: revision("HEAD^{tree}") };
if (process.env.GITHUB_ACTIONS === "true")
  assert.equal(source.head, process.env.GITHUB_SHA, "workflow source must match this event");

// Released actionlint 1.7.12 predates GitHub's documented concurrency.queue.
// Accept only that precise unsupported-key diagnostic after checking the real
// literal value and its cancellation siblings. All other diagnostics are fatal.
function supportedQueue(error: Diagnostic) {
  if (
    error.kind !== "syntax-check" ||
    error.message !==
      'unexpected key "queue" for "concurrency" section. expected one of "cancel-in-progress", "group"'
  )
    return false;
  assert.ok(!path.isAbsolute(error.filepath) && !error.filepath.split("/").includes(".."));
  const lines = readFileSync(error.filepath, "utf8").split("\n");
  const line = error.line - 1;
  const match = /^( *)queue: (single|max)(?:\s+#.*)?$/.exec(lines[line]);
  if (!match) return false;
  const indent = match[1].length;
  if (indent < 2) return false;
  let parent = line - 1;
  while (parent >= 0) {
    const previous = lines[parent];
    if (previous.trim() && !previous.trimStart().startsWith("#")) {
      if (previous.length - previous.trimStart().length < indent) break;
    }
    parent--;
  }
  if (!new RegExp(`^ {${indent - 2}}concurrency:\\s*(?:#.*)?$`).test(lines[parent])) return false;
  let end = parent + 1;
  while (end < lines.length) {
    const candidate = lines[end];
    if (
      candidate.trim() &&
      !candidate.trimStart().startsWith("#") &&
      candidate.length - candidate.trimStart().length < indent
    )
      break;
    end++;
  }
  if (match[2] === "max") {
    const cancellation = lines
      .slice(parent + 1, end)
      .filter((text) => new RegExp(`^ {${indent}}cancel-in-progress:`).test(text));
    if (cancellation.some((text) => !/cancel-in-progress: false\s*(?:#.*)?$/.test(text)))
      return false;
  }
  return true;
}

function check(files: string[], packet: string): Diagnostic[] {
  const result = spawnSync(
    binary,
    ["-shellcheck=", "-pyflakes=", "-format", "{{json .}}", ...files],
    { cwd: root, encoding: "utf8", timeout: 30_000, maxBuffer: 16 * 1024 * 1024 },
  );
  assert.ok(!result.error && !result.signal, "workflow checker did not finish normally");
  assert.ok(result.status === 0 || result.status === 1, "workflow checker failed to execute");
  const errors: Diagnostic[] = JSON.parse(result.stdout || "[]");
  assert.ok(Array.isArray(errors), "whole workflow diagnostics are required");
  const compatible = errors.filter(supportedQueue);
  const refused = errors.filter((error) => !compatible.includes(error));
  writeFileSync(
    path.join(output, `${packet}.json`),
    JSON.stringify(
      { status: result.status, stderr: result.stderr, errors, compatible, refused },
      null,
      2,
    ) + "\n",
  );
  assert.equal(Boolean(errors.length), result.status === 1, "checker status/diagnostics mismatch");
  return refused;
}

const base = `name: Workflow syntax control
on: workflow_dispatch
jobs:
  context:
    runs-on: ubuntu-latest
    steps:
      - run: "true"
`;
const controls: Array<[string, string, boolean]> = [
  [
    "valid-step-context",
    base.replace(
      '      - run: "true"',
      '      - env:\n          TEMP: ${{ runner.temp }}\n        run: "true"',
    ),
    false,
  ],
  [
    "invalid-job-context",
    base.replace("    steps:", "    env:\n      TEMP: ${{ runner.temp }}\n    steps:"),
    true,
  ],
  [
    "invalid-job-context-with-interpolation",
    base.replace(
      "    steps:",
      "    env:\n      TEMP: output-${{ runner.temp }}/receipt\n    steps:",
    ),
    true,
  ],
  [
    "invalid-top-level-steps-context",
    base.replace("jobs:", "env:\n  VALUE: ${{ steps.prepare.outputs.path }}\njobs:"),
    true,
  ],
  [
    "valid-queue-max",
    base.replace(
      "jobs:",
      "concurrency:\n  group: syntax-control\n  queue: max\n  cancel-in-progress: false\njobs:",
    ),
    false,
  ],
  [
    "invalid-queue-cancellation",
    base.replace(
      "jobs:",
      "concurrency:\n  group: syntax-control\n  queue: max\n  cancel-in-progress: true\njobs:",
    ),
    true,
  ],
  [
    "invalid-queue-value",
    base.replace("jobs:", "concurrency:\n  group: syntax-control\n  queue: unknown\njobs:"),
    true,
  ],
  [
    "valid-queue-retains-other-refusal",
    base
      .replace("jobs:", "concurrency:\n  group: syntax-control\n  queue: max\njobs:")
      .replace("    steps:", "    env:\n      TEMP: ${{ runner.temp }}\n    steps:"),
    true,
  ],
];
const controlDirectory = path.join(output, "controls");
mkdirSync(controlDirectory, { recursive: true });
const activeControls = process.env.WORKFLOW_VALIDATOR_CONTROLS === "true" ? controls : [];
for (const [name, source, mustRefuse] of activeControls) {
  const relative = `target/workflow-syntax/controls/${name}.yml`;
  writeFileSync(relative, source);
  const errors = check([relative], name);
  assert.equal(Boolean(errors.length), mustRefuse, `${name}: actual checker control failed`);
}

const files = readdirSync(".github/workflows")
  .filter((name) => /\.ya?ml$/.test(name))
  .sort()
  .map((name) => `.github/workflows/${name}`);
assert.ok(files.length, "the whole actual workflow inventory is required");
const hash = (file: string) => createHash("sha256").update(readFileSync(file)).digest("hex");
const before = Object.fromEntries(files.map((file) => [file, hash(file)]));
const refused = check(files, "actual-workflows");
assert.deepEqual(Object.fromEntries(files.map((file) => [file, hash(file)])), before);
writeFileSync(
  path.join(output, "source.json"),
  JSON.stringify(
    {
      source,
      githubSha: process.env.GITHUB_SHA ?? null,
      node: process.version,
      checkerBinarySha256: hash(binary),
      files: before,
      controls: activeControls.length,
    },
    null,
    2,
  ) + "\n",
);
for (const error of refused)
  process.stderr.write(`${error.filepath}:${error.line}:${error.column}: ${error.message}\n`);
assert.equal(refused.length, 0, "workflow syntax/context errors refuse qualification");
process.stdout.write(
  `Qualified ${files.length} whole workflows and ${activeControls.length} checker controls.\n`,
);
