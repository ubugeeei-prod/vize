/** Authenticate one immutable Release producer before public acceptance. */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

type Json = Record<string, unknown>;

const sha = (value: unknown): string => {
  assert.equal(typeof value, "string");
  assert.match(value as string, /^[0-9a-f]{40}$/u);
  return value as string;
};
const text = (value: unknown): string => {
  assert.equal(typeof value, "string");
  return value as string;
};
const integer = (value: unknown): number => {
  assert.ok(Number.isSafeInteger(value) && Number(value) > 0);
  return Number(value);
};
const object = (value: unknown): Json => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Json;
};

function commandText(executable: string, args: string[], cwd?: string): string {
  return execFileSync(executable, args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
    env: process.env,
  });
}

function commandBytes(executable: string, args: string[]): Buffer {
  return execFileSync(executable, args, {
    maxBuffer: 16 * 1024 * 1024,
    env: process.env,
  });
}

export function validateCandidateTag(
  ref: string,
  expectedCandidate: string,
  expectedSource: string,
  cwd = process.cwd(),
): { candidate: string; source: string } {
  const candidate = commandText(
    "git",
    ["--no-replace-objects", "rev-parse", `${ref}^{commit}`],
    cwd,
  ).trim();
  assert.equal(candidate, sha(expectedCandidate));
  const raw = commandText("git", ["--no-replace-objects", "cat-file", "commit", candidate], cwd);
  const parents = raw
    .split("\n\n", 1)[0]!
    .split("\n")
    .filter((line) => line.startsWith("parent "))
    .map((line) => line.slice(7));
  assert.deepEqual(parents, [sha(expectedSource)]);
  return { candidate, source: parents[0]! };
}

function selectProducerArtifact(
  artifactsValue: unknown,
  latestRunAttempt: number,
): { artifactId: number; attempt: number } {
  const artifacts = object(artifactsValue).artifacts;
  assert.ok(Array.isArray(artifacts));
  const matches = artifacts
    .map(object)
    .map((artifact) => {
      const match = /^release-producer-source-([1-9]\d*)$/u.exec(text(artifact.name));
      return match ? { artifact, attempt: integer(Number(match[1])) } : undefined;
    })
    .filter((row) => row !== undefined)
    .sort((left, right) => left.attempt - right.attempt);
  assert.ok(matches.length > 0);
  for (let index = 1; index < matches.length; index += 1)
    assert.notEqual(matches[index - 1]!.attempt, matches[index]!.attempt);
  const selected = matches.at(-1)!;
  assert.ok(selected.attempt <= latestRunAttempt);
  assert.equal(selected.artifact.expired, false);
  return { artifactId: integer(selected.artifact.id), attempt: selected.attempt };
}

export function validateProducerIdentity(
  request: { version: string; candidate: string; source: string; run: string },
  runValue: unknown,
  artifactsValue: unknown,
  recordValue: unknown,
): { artifactId: number; workflowHead: string; runAttempt: number } {
  assert.match(request.version, /^\d+\.\d+\.\d+$/u);
  const candidate = sha(request.candidate);
  const source = sha(request.source);
  assert.match(request.run, /^[1-9]\d*$/u);

  const run = object(runValue);
  assert.equal(String(integer(run.id)), request.run);
  assert.equal(run.name, `Release ${request.version} @ ${source}`);
  assert.equal(run.path, ".github/workflows/release.yml");
  assert.equal(run.event, "workflow_dispatch");
  assert.equal(run.status, "completed");
  assert.equal(run.conclusion, "success");
  const latestRunAttempt = integer(run.run_attempt);
  const workflowHead = sha(run.head_sha);
  const artifact = selectProducerArtifact(artifactsValue, latestRunAttempt);

  const record = object(recordValue);
  assert.equal(record.schema, "vize.releaseProducerCandidate.v1");
  assert.equal(record.repository, "ubugeeei-prod/vize");
  assert.equal(record.runId, request.run);
  assert.equal(record.runAttempt, artifact.attempt);
  assert.equal(record.workflowHead, workflowHead);
  assert.equal(record.version, request.version);
  assert.equal(record.candidateSha, candidate);
  assert.equal(record.producerSource, source);
  sha(record.nativeManifestGitBlob);
  sha(record.inventoryGitBlob);
  return { artifactId: artifact.artifactId, workflowHead, runAttempt: artifact.attempt };
}

function api(endpoint: string): Json {
  return object(JSON.parse(commandText("gh", ["api", `repos/ubugeeei-prod/vize/${endpoint}`])));
}

function apiBytes(endpoint: string): Buffer {
  return commandBytes("gh", ["api", `repos/ubugeeei-prod/vize/${endpoint}`]);
}

export function authenticate(
  version: string,
  candidate: string,
  source: string,
  producerRun: string,
): void {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
  assert.equal(process.env.GITHUB_REF, "refs/heads/main");
  assert.match(
    process.env.GITHUB_WORKFLOW_REF ?? "",
    /ubugeeei-prod\/vize\/\.github\/workflows\/no-unused-script-vars-public\.yml@refs\/heads\/main$/u,
  );
  validateCandidateTag("FETCH_HEAD", candidate, source);
  const run = api(`actions/runs/${producerRun}`);
  const latestRunAttempt = integer(run.run_attempt);
  const rows: unknown[] = [];
  for (let attempt = 1; attempt <= latestRunAttempt; attempt += 1) {
    const page = api(
      `actions/runs/${producerRun}/artifacts?name=release-producer-source-${attempt}`,
    );
    const artifacts = object(page).artifacts;
    assert.ok(Array.isArray(artifacts) && artifacts.length <= 1);
    rows.push(...artifacts);
  }
  const artifacts = { artifacts: rows };
  const artifactId = selectProducerArtifact(artifacts, latestRunAttempt).artifactId;

  const runnerTemp = realpathSync(text(process.env.RUNNER_TEMP));
  const directory = mkdtempSync(join(runnerTemp, "vize-unused-vars-authority-"));
  try {
    const zip = apiBytes(`actions/artifacts/${artifactId}/zip`);
    const archive = join(directory, "producer.zip");
    writeFileSync(archive, zip, { flag: "wx" });
    const raw = commandBytes("unzip", ["-p", archive, "producer-candidate.json"]);
    const record = object(JSON.parse(raw.toString("utf8")));
    const identity = validateProducerIdentity(
      { version, candidate, source, run: producerRun },
      run,
      artifacts,
      record,
    );
    process.stdout.write(
      `${JSON.stringify({ version, candidate, source, producerRun, ...identity })}\n`,
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

if (process.argv[1] && realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [version, candidate, source, producerRun] = process.argv.slice(2);
  assert.ok(version && candidate && source && producerRun);
  authenticate(version, candidate, source, producerRun);
}
