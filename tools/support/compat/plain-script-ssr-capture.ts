import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { copyFileSync, lstatSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const producer = resolve("producer");
const proof = resolve(process.env.SSR_CAPTURE_PROOF!);
mkdirSync(proof, { recursive: true });
const hash = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
const json = (path: string, value: unknown) =>
  writeFileSync(path, JSON.stringify(value, null, 2) + "\n");
function api(path: string) {
  const result = spawnSync("gh", ["api", "repos/ubugeeei-prod/vize/" + path], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}
const artifact = api("actions/artifacts/12748357555");
assert.equal(artifact.name, "source-cli-40528310601-1");
assert.equal(artifact.workflow_run.id, 40528310601);
assert.equal(artifact.workflow_run.head_sha, "824bda644a79650900c5e87ef45bf9c1ab1310ad");
assert.equal(
  artifact.digest,
  "sha256:1b31bf4a5d422e0164b7b0ba6e78b5421c62f97bb956ec901eab63fb2d707830",
);
assert.equal(artifact.expired, false);
const job = api("actions/jobs/122282680803");
assert.equal(job.run_id, 40528310601);
assert.equal(job.conclusion, "success");
const commit = api("git/commits/3bfe310301328e45bb33ffa56cc3d2fbfe7f3945");
assert.equal(commit.tree.sha, "725f800dde5688b9f9354c99d45ab674de80f64f");
assert(
  commit.parents.some(
    (parent: { sha: string }) => parent.sha === "824bda644a79650900c5e87ef45bf9c1ab1310ad",
  ),
);
const archive = resolve(process.env.SSR_CAPTURE_ARCHIVE!);
const listing = spawnSync("tar", ["-tzf", archive], { encoding: "utf8" });
assert.equal(listing.status, 0, listing.stderr);
const names = [
  "target/ci/vize",
  "target/ci/vize.differential-build.json",
  "target/ci/examples/cli_build_result_oracle",
  "source-cli.json",
];
assert.deepEqual(listing.stdout.trim().split("\n"), names);
const verbose = spawnSync("tar", ["-tvzf", archive], { encoding: "utf8" });
assert.equal(verbose.status, 0, verbose.stderr);
assert.deepEqual(
  verbose.stdout
    .trim()
    .split("\n")
    .map((line) => line[0]),
  ["-", "-", "-", "-"],
);
const stage = join(proof, "original-producer");
mkdirSync(stage);
const extract = spawnSync("tar", ["-xzf", archive, "-C", stage], { encoding: "utf8" });
assert.equal(extract.status, 0, extract.stderr);
const envelopeBytes = readFileSync(join(stage, "source-cli.json"));
const envelope = JSON.parse(envelopeBytes.toString()) as {
  identity: {
    sourceRevision: string;
    sourceTree: string;
    runId: string;
    runAttempt: string;
    platform: string;
    arch: string;
  };
  recipe: string;
  members: { path: string; bytes: number; mode: number; sha256: string }[];
};
assert.equal(envelope.identity.sourceRevision, commit.sha);
assert.equal(envelope.identity.sourceTree, commit.tree.sha);
assert.equal(envelope.identity.runId, "40528310601");
assert.equal(envelope.identity.runAttempt, "1");
assert.equal(envelope.identity.platform, "linux");
assert.equal(envelope.identity.arch, "x64");
assert.equal(envelope.recipe, "cargo build --profile ci -p vize");
assert.deepEqual(
  envelope.members.map((member) => member.path),
  names.slice(0, 3),
);
assert.deepEqual(
  envelope.members.map((member) => member.mode),
  [0o755, 0o644, 0o755],
);
for (const member of envelope.members) {
  const path = join(stage, member.path);
  const stat = lstatSync(path);
  assert(stat.isFile() && !stat.isSymbolicLink());
  assert.equal(stat.size, member.bytes);
  assert.equal(stat.mode & 0o777, member.mode);
  assert.equal(hash(readFileSync(path)), member.sha256);
}
const receiptBytes = readFileSync(join(stage, names[1]!));
const receipt = JSON.parse(receiptBytes.toString());
assert.equal(receipt.schema, "vize.differential.build");
assert.equal(receipt.version, 1);
assert.equal(receipt.recipe, envelope.recipe);
assert.equal(receipt.sourceRevision, commit.sha);
assert.equal(receipt.binaryPath, names[0]);
assert.equal(receipt.binarySha256, envelope.members[0]!.sha256);
const cargo = readFileSync(join(producer, "Cargo.toml"), "utf8");
const workspaceVersion = cargo.match(/\[workspace\.package\][\s\S]*?^version = "([^"]+)"/m);
assert(workspaceVersion);
assert.equal(receipt.cliVersion, "vize " + workspaceVersion[1] + "-dev");
const executable = join(stage, names[0]!);
const version = spawnSync(executable, ["--version"], { encoding: "utf8" });
assert.equal(version.status, 0, version.stderr);
assert.equal(version.stdout.trim(), receipt.cliVersion);
const original = readFileSync(
  join(producer, "tests/_fixtures/differential/compiler/plain-script-import-10903/src/App.vue.txt"),
);
assert.equal(hash(original), "d801e065a254bbffbd8ee5cac7a5d87885139c6bab38671c2a23916e4d780bee");
const args = ["build", "Layout.vue", "--format", "json", "--output", "out", "--ssr", "--no-config"];
const packets: unknown[] = [];
const modules: string[] = [];
for (const name of ["first", "second"]) {
  const project = join(proof, name);
  mkdirSync(project);
  writeFileSync(join(project, "Layout.vue"), original);
  const result = spawnSync(executable, args, {
    cwd: project,
    encoding: "buffer",
    timeout: 120_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr?.toString() || result.error?.message);
  const bytes = readFileSync(join(project, "out/Layout.json"));
  const parsed = JSON.parse(bytes.toString());
  packets.push({
    name,
    executable,
    args,
    cwd: project,
    status: result.status,
    signal: result.signal,
    stdoutBase64: result.stdout.toString("base64"),
    stderrBase64: result.stderr.toString("base64"),
    rawJsonBase64: bytes.toString("base64"),
    rawJsonSha256: hash(bytes),
    result: parsed,
  });
  json(join(proof, "whole-observations.json"), packets);
  assert.deepEqual(Object.keys(parsed), [
    "filename",
    "code",
    "css",
    "errors",
    "warnings",
    "script_lang",
    "macro_artifacts",
  ]);
  assert.equal(parsed.filename, "Layout.vue");
  assert.equal(parsed.script_lang, "ts");
  assert.deepEqual(parsed.errors, []);
  assert.deepEqual(parsed.warnings, []);
  assert.deepEqual(parsed.macro_artifacts, []);
  assert.equal(parsed.css, null);
  assert.equal(typeof parsed.code, "string");
  modules.push(parsed.code);
}
assert.equal(
  modules[0],
  modules[1],
  "two independent original CLI executions must agree byte for byte",
);
writeFileSync(join(proof, "module.expected.txt"), modules[0]!);
copyFileSync(join(stage, "source-cli.json"), join(proof, "original-producer-envelope.json"));
copyFileSync(join(stage, names[1]!), join(proof, "original-build-receipt.json"));
const custody = {
  artifact,
  producerJob: { id: job.id, runId: job.run_id, conclusion: job.conclusion },
  producerCommit: commit,
  originalProducerIdentity: envelope.identity,
  originalProducerEnvelopeSha256: hash(envelopeBytes),
  archiveSha256: hash(readFileSync(archive)),
  binarySha256: envelope.members[0]!.sha256,
  consumerRecipe: {
    source: process.env.GITHUB_SHA,
    runId: process.env.GITHUB_RUN_ID,
    runAttempt: process.env.GITHUB_RUN_ATTEMPT,
  },
  role: "cross-run replay of the original authenticated producer; not source qualification",
};
json(join(proof, "custody.json"), custody);
const packet = {
  inputSha256: hash(original),
  moduleSha256: hash(modules[0]!),
  moduleBase64: Buffer.from(modules[0]!).toString("base64"),
  firstWholeJsonBase64: readFileSync(join(proof, "first/out/Layout.json")).toString("base64"),
  binarySha256: envelope.members[0]!.sha256,
  producerSource: commit.sha,
  producerTree: commit.tree.sha,
  producerRun: "40528310601",
  producerJob: "122282680803",
  artifactId: 12748357555,
  recipeSource: process.env.GITHUB_SHA,
  independentExecutions: 2,
};
json(join(proof, "reference-packet.json"), packet);
console.log("SSR_REFERENCE_PACKET=" + JSON.stringify(packet));
