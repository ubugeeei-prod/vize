import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import {
  BUILD_RECIPE,
  validateBuildReceipt,
} from "../../../../tests/differential/build-receipt.ts";
import { COMPILER_ARGV } from "../../../../tests/differential/compiler-dialect.ts";

const producer = {
  source: "3e67833be5b3ccd85b871a2bfa2d05683e77c3a4",
  tree: "64de3a23acdf285826f84c434139c3f74c0fcf74",
  run: 41271259992,
  attempt: 1,
  job: 124744341561,
  artifact: 13070015036,
  artifactName: "source-cli-41271259992-1",
  artifactBytes: 61_081_688,
  artifactDigest: "sha256:3e38b0bbe9a48f2b54188bea75177cf9e43609223f92953903144803b957a207",
} as const;
const root = process.cwd();
const sourceRoot = path.join(root, "reference-source");
const temp = process.env.RUNNER_TEMP;
assert.ok(temp);
const evidence = path.join(temp, "sfc-entry-reference-evidence");
const archiveRoot = path.join(temp, "sfc-entry-reference-runtime");
const stage = path.join(temp, "sfc-entry-reference-image");
const sha = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const git = (...args: string[]) => execFileSync("git", args, { encoding: "utf8" }).trim();
function record(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}
function save(name: string, value: unknown): void {
  fs.writeFileSync(path.join(evidence, name), `${JSON.stringify(value, null, 2)}\n`);
}
async function api(endpoint: string): Promise<Record<string, unknown>> {
  assert.ok(process.env.GH_TOKEN);
  const response = await fetch(`https://api.github.com/repos/ubugeeei-prod/vize/${endpoint}`, {
    headers: {
      Authorization: `Bearer ${process.env.GH_TOKEN}`,
      Accept: "application/vnd.github+json",
    },
  });
  const bytes = await response.text();
  assert.ok(Buffer.byteLength(bytes) < 2 * 1024 * 1024);
  fs.writeFileSync(path.join(evidence, `${endpoint.replaceAll("/", "-")}.json`), bytes);
  assert.equal(response.status, 200);
  return record(JSON.parse(bytes));
}
function controller(): void {
  fs.mkdirSync(evidence, { recursive: true });
  assert.match(process.env.SOURCE_SHA ?? "", /^[a-f0-9]{40}$/);
  assert.equal(git("rev-parse", "HEAD"), process.env.SOURCE_SHA);
  assert.equal(process.env.GITHUB_SHA, process.env.SOURCE_SHA);
  assert.equal(git("-C", sourceRoot, "rev-parse", "HEAD"), producer.source);
  assert.equal(git("-C", sourceRoot, "rev-parse", "HEAD^{tree}"), producer.tree);
  save("controller.json", {
    source: process.env.SOURCE_SHA,
    tree: git("rev-parse", "HEAD^{tree}"),
    run: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
    node: process.version,
    producer,
    scope:
      "Existing source CLI, one valid authored SFC SSR control; no negative-input, candidate, native-parity or published acceptance.",
  });
}
async function authenticate(): Promise<void> {
  controller();
  const artifact = await api(`actions/artifacts/${producer.artifact}`);
  const run = await api(`actions/runs/${producer.run}`);
  const job = await api(`actions/jobs/${producer.job}`);
  assert.equal(artifact.id, producer.artifact);
  assert.equal(artifact.name, producer.artifactName);
  assert.equal(artifact.size_in_bytes, producer.artifactBytes);
  assert.equal(artifact.digest, producer.artifactDigest);
  assert.equal(artifact.expired, false);
  assert.equal(record(artifact.workflow_run).id, producer.run);
  assert.equal(record(artifact.workflow_run).head_sha, producer.source);
  assert.equal(run.head_sha, producer.source);
  assert.equal(run.run_attempt, producer.attempt);
  assert.equal(run.event, "merge_group");
  assert.equal(run.status, "completed");
  assert.equal(run.conclusion, "success");
  assert.equal(job.id, producer.job);
  assert.equal(job.run_id, producer.run);
  assert.equal(
    job.name,
    "PR source checks / Affected tooling CLI / Build original full-suite CLI and SFC oracle",
  );
  assert.equal(job.status, "completed");
  assert.equal(job.conclusion, "success");
  save("authenticated.json", { producer, controller: process.env.SOURCE_SHA });
}
function capture(): void {
  controller();
  assert.deepEqual(JSON.parse(fs.readFileSync(path.join(evidence, "authenticated.json"), "utf8")), {
    producer,
    controller: process.env.SOURCE_SHA,
  });
  const archive = path.join(archiveRoot, "source-cli.tar.gz");
  const members = [
    "target/ci/vize",
    "target/ci/vize.differential-build.json",
    "target/ci/examples/cli_build_result_oracle",
  ];
  const expectedMembers = [...members, "source-cli.json"];
  assert.deepEqual(
    execFileSync("tar", ["-tzf", archive], { encoding: "utf8" }).trim().split("\n"),
    expectedMembers,
  );
  assert.deepEqual(
    execFileSync("tar", ["-tvzf", archive], { encoding: "utf8" })
      .trim()
      .split("\n")
      .map((line) => line[0]),
    ["-", "-", "-", "-"],
  );
  fs.mkdirSync(stage, { recursive: false });
  execFileSync("tar", ["-xzf", archive, "-C", stage]);
  const envelopeBytes = fs.readFileSync(path.join(stage, "source-cli.json"));
  const envelope = record(JSON.parse(envelopeBytes.toString("utf8")));
  const identity = record(envelope.identity);
  assert.equal(identity.sourceRevision, producer.source);
  assert.equal(identity.sourceTree, producer.tree);
  assert.equal(identity.runId, String(producer.run));
  assert.equal(identity.runAttempt, String(producer.attempt));
  assert.equal(identity.platform, "linux");
  assert.equal(identity.arch, "x64");
  assert.equal(envelope.recipe, BUILD_RECIPE);
  assert.equal(
    envelope.oracleRecipe,
    "cargo build --profile ci -p vize_atelier_sfc --example cli_build_result_oracle",
  );
  assert.ok(Array.isArray(envelope.members));
  const declared = envelope.members.map((member: unknown) => record(member));
  assert.deepEqual(
    declared.map((member) => member.path),
    members,
  );
  for (const [member, mode] of [
    [members[0], 0o755],
    [members[1], 0o644],
    [members[2], 0o755],
  ] as const) {
    assert.ok(member);
    const physical = path.join(stage, member);
    const stat = fs.lstatSync(physical);
    assert.ok(stat.isFile() && !stat.isSymbolicLink());
    const expected = declared.find((entry) => entry.path === member);
    assert.ok(expected);
    assert.equal(stat.mode & 0o777, mode);
    assert.equal(expected.mode, mode);
    assert.equal(stat.size, expected.bytes);
    assert.equal(sha(fs.readFileSync(physical)), expected.sha256);
  }
  const image = path.join(stage, "target/ci/vize");
  const imageSha256 = sha(fs.readFileSync(image));
  const cargo = git("-C", sourceRoot, "show", "HEAD:Cargo.toml");
  const version = cargo.match(/\[workspace\.package\][\s\S]*?^version = "([^"]+)"/m)?.[1];
  assert.ok(version);
  const buildBytes = fs.readFileSync(path.join(stage, "target/ci/vize.differential-build.json"));
  validateBuildReceipt(JSON.parse(buildBytes.toString("utf8")), {
    sourceRevision: producer.source,
    binaryPath: "target/ci/vize",
    binarySha256: imageSha256,
    cliVersion: `vize ${version}-dev`,
  });
  fs.writeFileSync(path.join(evidence, "producer-envelope.json"), envelopeBytes);
  fs.writeFileSync(path.join(evidence, "producer-build.json"), buildBytes);
  const controls = [
    {
      id: "sfc-entry-validation-11262-11263",
      file: "tests/_fixtures/differential/compiler/sfc-entry-validation-11262-11263/Layout.vue.txt",
      sha256: "7ec65634a398549830e6f1dc6b770f268cb4423c53e539f1746a911dc7057660",
    },
  ];
  const observations: Array<{
    control: (typeof controls)[number];
    output: Buffer | null;
    error: Error | undefined;
    status: number | null;
    signal: NodeJS.Signals | null;
  }> = [];
  for (const control of controls) {
    const input = fs.readFileSync(path.join(root, control.file));
    assert.equal(sha(input), control.sha256);
    const workspace = path.join(temp!, `sfc-entry-reference-work-${control.id}`);
    fs.mkdirSync(workspace, { recursive: false });
    fs.writeFileSync(path.join(workspace, "Layout.vue"), input);
    const result = spawnSync(image, COMPILER_ARGV, {
      cwd: workspace,
      timeout: 30_000,
      maxBuffer: 4 * 1024 * 1024,
    });
    const resultPath = path.join(workspace, "out/Layout.json");
    const output = fs.existsSync(resultPath) ? fs.readFileSync(resultPath) : null;
    const packet = {
      id: control.id,
      controller: process.env.SOURCE_SHA,
      producer,
      imageSha256,
      inputFile: control.file,
      sourceBase64: input.toString("base64"),
      inputSha256: sha(input),
      argv: COMPILER_ARGV,
      status: result.status,
      signal: result.signal,
      stdoutBase64: result.stdout?.toString("base64") ?? "",
      stderrBase64: result.stderr?.toString("base64") ?? "",
      error: result.error
        ? { name: result.error.name, message: result.error.message, stack: result.error.stack }
        : null,
      wholeJsonBase64: output?.toString("base64") ?? null,
    };
    save(`${control.id}.json`, packet);
    assert.equal(sha(fs.readFileSync(image)), imageSha256);
    assert.deepEqual(fs.readFileSync(path.join(workspace, "Layout.vue")), input);
    observations.push({
      control,
      output,
      error: result.error,
      status: result.status,
      signal: result.signal,
    });
  }
  // The complete process packet exists before any process-status or result-shape assertion.
  for (const { control, output, error, status, signal } of observations) {
    assert.equal(error, undefined);
    assert.equal(status, 0);
    assert.equal(signal, null);
    assert.ok(output);
    const whole = record(JSON.parse(output.toString("utf8")));
    assert.deepEqual(Object.keys(whole), [
      "filename",
      "code",
      "css",
      "errors",
      "warnings",
      "script_lang",
      "macro_artifacts",
    ]);
    assert.equal(whole.filename, "Layout.vue");
    assert.equal(whole.script_lang, "js");
    assert.equal(whole.css, null);
    assert.deepEqual(whole.errors, []);
    assert.deepEqual(whole.warnings, []);
    assert.deepEqual(whole.macro_artifacts, []);
    assert.equal(typeof whole.code, "string");
    const code = whole.code;
    assert.ok(typeof code === "string" && code.length > 0);
    fs.writeFileSync(path.join(evidence, `${control.id}.module.expected.txt`), code);
  }
  save("receipt.json", {
    controller: process.env.SOURCE_SHA,
    consumerRun: process.env.GITHUB_RUN_ID,
    producer,
    archiveSha256: sha(fs.readFileSync(archive)),
    producerEnvelopeSha256: sha(envelopeBytes),
    buildReceiptSha256: sha(buildBytes),
    imageSha256,
    transport:
      "Pinned actions/download-artifact validates the pre-authenticated artifact ZIP digest with digest-mismatch:error; no image is uploaded by this consumer.",
    cases: controls,
  });
}

try {
  if (process.argv[2] === "authenticate") await authenticate();
  else if (process.argv[2] === "capture") capture();
  else throw new Error("expected authenticate or capture");
} catch (error) {
  fs.mkdirSync(evidence, { recursive: true });
  save(
    "failure.json",
    error instanceof Error
      ? { name: error.name, message: error.message, stack: error.stack }
      : { value: String(error) },
  );
  throw error;
}
