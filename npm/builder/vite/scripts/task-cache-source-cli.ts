import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

assert.equal(process.platform, "linux", "source CLI custody is for Linux x64");
assert.equal(process.arch, "x64");
const artifactId = required("VIZE_TASK_CACHE_SOURCE_ARTIFACT_ID", /^\d+$/u);
const archiveSha = required("VIZE_TASK_CACHE_SOURCE_ZIP_SHA256", /^[a-f0-9]{64}$/u);
const binarySha = required("VIZE_TASK_CACHE_SOURCE_BINARY_SHA256", /^[a-f0-9]{64}$/u);
const sourceSha = required("VIZE_TASK_CACHE_SOURCE_SHA", /^[a-f0-9]{40}$/u);
assert.ok(!process.env.VIZE_TASK_CACHE_PUBLIC_VERSION, "public acceptance never overrides its CLI");
const repository = required("GITHUB_REPOSITORY", /^[\w.-]+\/[\w.-]+$/u);

function required(name: string, pattern: RegExp): string {
  const value = process.env[name];
  assert.ok(value && pattern.test(value), `invalid ${name}`);
  return value;
}

function record(value: unknown): asserts value is Record<string, unknown> {
  assert.ok(typeof value === "object" && value !== null && !Array.isArray(value));
}

function github(endpoint: string): unknown {
  const result = spawnSync("gh", ["api", `repos/${repository}/${endpoint}`], {
    encoding: "utf8",
    maxBuffer: 4 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout) as unknown;
}

function output(name: string, value: string): void {
  assert.ok(!/[\r\n]/u.test(value));
  const destination = process.env.GITHUB_OUTPUT;
  assert.ok(destination);
  fs.appendFileSync(destination, `${name}=${value}\n`);
}

function textMember(archive: string, member: string): string {
  const result = spawnSync("tar", ["-xOf", archive, member], {
    encoding: "utf8",
    maxBuffer: 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}

function fileMember(archive: string, member: string, destination: string): void {
  // Extract one named member to an owned file. Archive paths never choose a
  // destination, and tar never extracts links or other archive entries.
  const fd = fs.openSync(destination, "wx");
  try {
    const result = spawnSync("tar", ["-xOf", archive, member], {
      encoding: "utf8",
      stdio: ["ignore", fd, "pipe"],
    });
    assert.equal(result.status, 0, result.stderr);
  } finally {
    fs.closeSync(fd);
  }
}

if (process.argv[2] === "inspect") {
  const artifact = github(`actions/artifacts/${artifactId}`);
  record(artifact);
  assert.equal(String(artifact.id), artifactId);
  assert.equal(artifact.name, `persistent-batch-storage-${sourceSha}`);
  assert.equal(artifact.digest, `sha256:${archiveSha}`);
  assert.equal(artifact.expired, false);
  record(artifact.workflow_run);
  assert.equal(typeof artifact.workflow_run.repository_id, "number");
  assert.equal(artifact.workflow_run.repository_id, artifact.workflow_run.head_repository_id);
  assert.equal(typeof artifact.workflow_run.id, "number");
  assert.ok(
    typeof artifact.workflow_run.head_sha === "string" &&
      /^[a-f0-9]{40}$/u.test(artifact.workflow_run.head_sha),
  );
  const producer = github(`actions/runs/${String(artifact.workflow_run.id)}`);
  record(producer);
  assert.equal(producer.id, artifact.workflow_run.id);
  assert.equal(producer.head_sha, artifact.workflow_run.head_sha);
  assert.equal(producer.status, "completed");
  assert.equal(producer.conclusion, "success");
  const directory = fs.mkdtempSync(
    path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "vize-cache-cli-"),
  );
  fs.writeFileSync(
    path.join(directory, "producer.json"),
    JSON.stringify({ artifact, producer }, null, 2),
  );
  output("directory", directory);
  output("run-id", String(artifact.workflow_run.id));
} else {
  assert.equal(process.argv[2], "extract");
  const directory = fs.realpathSync(required("VIZE_TASK_CACHE_SOURCE_DIRECTORY", /./u));
  const archive = path.join(directory, "artifact/persistent-storage-evidence.tgz");
  assert.equal(textMember(archive, "persistent-storage-evidence/source.sha").trim(), sourceSha);
  const checksum = textMember(archive, "persistent-storage-evidence/cli.sha256");
  assert.equal(checksum.trim().split(/\s+/u)[0], binarySha);
  const packed = path.join(directory, "source-cli.tgz");
  fileMember(archive, "persistent-storage-evidence/source-cli-linux-x64.tgz", packed);
  const binary = path.join(directory, "vize");
  fileMember(packed, "vize", binary);
  const hash = createHash("sha256");
  for await (const chunk of fs.createReadStream(binary)) hash.update(chunk);
  assert.equal(hash.digest("hex"), binarySha, "actual contained source CLI bytes");
  fs.chmodSync(binary, 0o755);
  const receipt = path.join(directory, "verified-cli.json");
  fs.writeFileSync(
    receipt,
    JSON.stringify(
      {
        sourceSha,
        binary: { path: binary, sha256: binarySha, bytes: fs.statSync(binary).size, profile: "ci" },
        archive: { artifactId, zipSha256: archiveSha, digestMismatch: "error" },
        originalChecksum: checksum,
        custody: JSON.parse(
          fs.readFileSync(path.join(directory, "producer.json"), "utf8"),
        ) as unknown,
      },
      null,
      2,
    ),
  );
  output("binary", binary);
  output("receipt", receipt);
}
