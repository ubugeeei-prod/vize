// Plan, prepare and verify the focused original-project LSP campaign
// (.github/workflows/lsp-focused-projects.yml). The packet replay itself lives
// in lsp-focused-lifecycle.ts.
// usage: node lsp-focused-campaign.ts plan | dependencies | verify-resources | refuse
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

type Json = null | boolean | number | string | readonly Json[] | { readonly [key: string]: Json };
type JsonObject = { readonly [key: string]: Json };

/** JSON with two-space indentation and ASCII-only escapes, as the original receipts were written. */
export function receiptJson(value: unknown): string {
  const escape = (character: string): string =>
    Array.from(
      { length: character.length },
      (_, index) => `\\u${character.charCodeAt(index).toString(16).padStart(4, "0")}`,
    ).join("");
  return `${JSON.stringify(value, null, 2).replace(/[^\0-\x7f]/gu, escape)}\n`;
}

function object(value: unknown, label: string): JsonObject {
  assert.ok(typeof value === "object" && value !== null && !Array.isArray(value), label);
  return value as JsonObject;
}

function list(value: unknown, label: string): readonly Json[] {
  assert.ok(Array.isArray(value), label);
  return value as readonly Json[];
}

function text(value: unknown, label: string): string {
  assert.ok(typeof value === "string", label);
  return value;
}

const readJson = (file: string): unknown => JSON.parse(fs.readFileSync(file, "utf8"));
const sha256 = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
const git = (...args: string[]): string => execFileSync("git", args, { encoding: "utf8" }).trim();

function environment(name: string): string {
  const value = process.env[name];
  assert.ok(value !== undefined, `${name} is required`);
  return value;
}

function campaign(): string {
  return environment("CAMPAIGN_DIR");
}

interface PlannedProject extends JsonObject {
  readonly id: string;
  readonly revision: string;
  readonly shardIndex: number;
}

interface Plan {
  readonly sourceRevision: string;
  readonly shardCount: number;
  readonly preparationTrace: boolean;
  readonly projects: readonly PlannedProject[];
}

function readPlan(out: string): Plan {
  const plan = object(readJson(path.join(out, "plan.json")), "plan.json");
  return {
    sourceRevision: text(plan.sourceRevision, "plan.sourceRevision"),
    shardCount: Number(plan.shardCount),
    preparationTrace: plan.preparationTrace === true,
    projects: list(plan.projects, "plan projects").map((entry) => {
      const project = object(entry, "plan project");
      return {
        ...project,
        id: text(project.id, "project.id"),
        revision: text(project.revision, "project.revision"),
        shardIndex: Number(project.shardIndex),
      };
    }),
  };
}

const pins: Readonly<Record<string, string>> = {
  varlet: "1007097bde3f6c06326e0057ac6a7cf4ab7ff519",
  "primevue-volt": "d4374cb7c1267f35eba7cee5d0a266f50ca8ec84",
  "vue3-antd-admin": "64c03db76710e70b85aedf40a9ca87101946d70d",
  misskey: "810faa8e5db4bbf9d6d408a61df148559340ee99",
};
const scopes: Readonly<Record<string, readonly string[]>> = {
  "original-four": Object.keys(pins),
  "residual-two": ["vue3-antd-admin", "misskey"],
  "misskey-only": ["misskey"],
};

/** Freeze the exact source, registry selection, and fixture pins. */
function plan(): void {
  const root = process.cwd();
  const out = path.join(root, campaign());
  fs.mkdirSync(path.dirname(out), { recursive: true });
  fs.mkdirSync(out);
  const expected = environment("EXPECTED_SOURCE");
  assert.match(expected, /^[0-9a-f]{40}$/u);
  assert.equal(process.env.GITHUB_REF, "refs/heads/main");
  assert.ok(git("rev-parse", "HEAD") === expected && expected === process.env.GITHUB_SHA);
  assert.equal(process.env.GITHUB_WORKFLOW_SHA, expected);
  const scope = environment("PACKET_SCOPE");
  const selected = scopes[scope];
  assert.ok(selected, `unknown packet scope ${scope}`);
  const registryBytes = fs.readFileSync(
    path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"),
  );
  const registry = list(
    object(JSON.parse(registryBytes.toString("utf8")), "registry").projects,
    "projects",
  ).map((entry) => object(entry, "registry project"));
  const projects: JsonObject[] = [];
  for (const [identifier, revision] of Object.entries(pins)) {
    const matches = registry.flatMap((project, index) =>
      project.id === identifier ? [{ project, index }] : [],
    );
    assert.equal(matches.length, 1, identifier);
    const [{ project, index } = { project: {}, index: -1 }] = matches;
    assert.equal(project.revision, revision);
    assert.ok(list(project.coverage, "coverage").includes("lsp"));
    const fixturePath = text(project.fixturePath, "project.fixturePath");
    assert.equal(git("ls-tree", "HEAD", fixturePath).split(/\s+/u)[2], revision);
    if (selected.includes(identifier)) projects.push({ ...project, shardIndex: index });
  }
  assert.deepEqual(
    projects.map((project) => project.id),
    selected,
  );
  const sourceBlobs: Record<string, string> = {};
  for (const source of [
    "crates/vize_canon/src",
    "crates/vize_maestro/src",
    "tests/tooling/real-project-lsp.test.ts",
    "tests/tooling/support",
  ])
    sourceBlobs[source] = git("rev-parse", `HEAD:${source}`);
  fs.writeFileSync(
    path.join(out, "plan.json"),
    receiptJson({
      sourceRevision: expected,
      sourceTree: git("rev-parse", "HEAD^{tree}"),
      workflowRevision: process.env.GITHUB_WORKFLOW_SHA,
      registrySha256: sha256(registryBytes),
      shardCount: registry.length,
      projects,
      packetScope: scope,
      preparationTrace: process.env.VIZE_TRACE_EDITOR_PREPARATION === "1",
      sourceBlobs,
      scope:
        "selected complete original Matrix LSP packets only; no full-Matrix or paired-performance claim",
      timingPolicy:
        "elapsed times include optional trace and raw recording overhead; no matched performance acceptance",
    }),
  );
  const fixtures = projects.map((project) => text(project.fixturePath, "project.fixturePath"));
  const update = spawnSync(
    "git",
    ["submodule", "update", "--init", "--depth", "1", "--", ...fixtures],
    {
      stdio: "inherit",
    },
  );
  if (update.error) throw update.error;
  assert.equal(update.status, 0, "git submodule update failed");
  for (const project of projects)
    assert.equal(
      git("-C", text(project.fixturePath, "project.fixturePath"), "rev-parse", "HEAD"),
      project.revision,
    );
}

/** Prepare the original pinned project dependencies, retaining every raw outcome. */
function dependencies(): void {
  const out = campaign();
  const { projects, shardCount } = readPlan(out);
  const results: JsonObject[] = [];
  for (const project of projects) {
    const directory = path.join(out, project.id);
    fs.mkdirSync(directory);
    const command = [
      "rust-script",
      "tools/commands/fixtures/typecheck-dependency-prepare.rs",
      "--output-dir",
      directory,
      "--shard-count",
      String(shardCount),
      "--shard-index",
      String(project.shardIndex),
      "--timeout-ms",
      "600000",
    ];
    const started = performance.now();
    const stdout = fs.openSync(path.join(directory, "prepare.stdout"), "w");
    const stderr = fs.openSync(path.join(directory, "prepare.stderr"), "w");
    let status: number | null;
    try {
      const [file = "", ...args] = command;
      const result = spawnSync(file, args, { stdio: ["inherit", stdout, stderr] });
      if (result.error) throw result.error;
      status = result.status;
    } finally {
      fs.closeSync(stdout);
      fs.closeSync(stderr);
    }
    results.push({
      id: project.id,
      command,
      exitStatus: status,
      durationMs: performance.now() - started,
    });
    fs.writeFileSync(path.join(out, "dependencies.json"), receiptJson(results));
  }
  assert.ok(
    results.every((result) => result.exitStatus === 0),
    "raw dependency failures retained",
  );
}

function resolved(file: string): string {
  try {
    return fs.realpathSync(file);
  } catch {
    return path.resolve(file);
  }
}

/** Verify trace resource custody independently of packet acceptance. */
function verifyResources(): void {
  const out = campaign();
  const plan = readPlan(out);
  const results = list(readJson(path.join(out, "sessions.json")), "sessions").map((entry) =>
    object(entry, "session"),
  );
  assert.equal(plan.preparationTrace, true);
  assert.deepEqual(
    results.map((result) => result.id),
    plan.projects.map((project) => project.id),
  );
  assert.ok(results.every((result) => result.resourceExitStatus === 0));
  const server = resolved(environment("VIZE_LSP_BIN"));
  const provider = resolved(environment("CORSA_PATH"));
  for (const project of plan.projects) {
    const directory = path.join(out, project.id);
    const resource = object(
      readJson(path.join(directory, "resource-summary.json")),
      "resource summary",
    );
    assert.equal(resource.sourceRevision, plan.sourceRevision);
    assert.equal(
      resource.samplerSha256,
      sha256(fs.readFileSync(text(resource.samplerSource, "resource.samplerSource"))),
    );
    const lines = fs
      .readFileSync(path.join(directory, "resource-samples.jsonl"), "utf8")
      .split("\n");
    if (lines.at(-1) === "") lines.pop();
    const samples = lines.map((line) => object(JSON.parse(line), "resource sample"));
    assert.ok(samples.length === resource.rawSamples && samples.length > 0);
    const executables = new Set(
      samples.flatMap((sample) =>
        list(sample.processes, "processes").map((entry) => object(entry, "process").executable),
      ),
    );
    assert.ok(executables.has(server));
    assert.ok(executables.has(provider));
  }
}

/** Refuse missing selected projects, stale binaries, or failed preparation. */
function refuse(): void {
  const out = campaign();
  const plan = readPlan(out);
  const receipt = object(readJson("target/ci/vize.differential-build.json"), "build receipt");
  assert.ok(
    receipt.sourceRevision === plan.sourceRevision &&
      plan.sourceRevision === process.env.EXPECTED_SOURCE,
  );
  for (const kind of ["dependencies", "sessions"]) {
    const results = list(readJson(path.join(out, `${kind}.json`)), kind).map((entry) =>
      object(entry, kind),
    );
    assert.deepEqual(
      results.map((result) => result.id),
      plan.projects.map((project) => project.id),
    );
    assert.ok(results.every((result) => result.exitStatus === 0));
    if (kind === "sessions") assert.ok(results.every((result) => result.wireExitStatus === 0));
  }
  for (const project of plan.projects) {
    const report = object(
      readJson(path.join(out, project.id, "lsp-lifecycle-summary.json")),
      "report",
    );
    assert.equal(report.commitSha, plan.sourceRevision);
    const evidence = object(object(report.evidence, "evidence").vizeBinary, "vizeBinary");
    assert.equal(evidence.sha256, receipt.binarySha256);
    assert.deepEqual(report.shard, { count: plan.shardCount, index: project.shardIndex });
    const projects = list(report.projects, "report projects").map((entry) =>
      object(entry, "report project"),
    );
    const [only] = projects;
    assert.ok(projects.length === 1 && only?.id === project.id);
    assert.equal(only.revision, project.revision);
    assert.equal(only.status, "ok");
  }
  const diff = spawnSync("git", ["diff", "--exit-code", "--ignore-submodules=all", "HEAD"], {
    stdio: "inherit",
  });
  if (diff.error) throw diff.error;
  assert.equal(diff.status, 0, "tracked files changed");
}

const commands: Readonly<Record<string, () => void>> = {
  plan,
  dependencies,
  "verify-resources": verifyResources,
  refuse,
};

if (import.meta.main) {
  const command = commands[process.argv[2] ?? ""];
  assert.ok(
    command,
    "usage: lsp-focused-campaign.ts plan | dependencies | verify-resources | refuse",
  );
  command();
}
