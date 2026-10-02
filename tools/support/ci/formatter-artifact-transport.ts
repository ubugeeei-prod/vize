import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { gzipSync } from "node:zlib";

// Read-only transport of an actual existing Actions artifact. Never execute a
// product, construct observations, change baselines or grant semantic credit.
const root = path.resolve(process.argv[2]);
const sourceHead = "3290e828a45c0e0e4ef474ca5420aef7dbac2e32";
const number = (name: string) => {
  const value = Number(process.env[name]);
  assert(Number.isSafeInteger(value) && value > 0, `invalid ${name}`);
  return value;
};
const runId = number("FORMATTER_SOURCE_RUN");
const attempt = number("FORMATTER_SOURCE_ATTEMPT");
const jobId = number("FORMATTER_SOURCE_JOB");
const artifactId = number("FORMATTER_ARTIFACT_ID");
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const api = async (endpoint: string) => {
  const response = await fetch(`https://api.github.com/repos/ubugeeei-prod/vize/${endpoint}`, {
    headers: {
      Authorization: `Bearer ${process.env.GH_TOKEN}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
    },
    redirect: "error",
  });
  assert.equal(response.status, 200, `metadata request failed: ${response.status}`);
  return await response.json();
};
const [run, job, artifact]: any[] = await Promise.all([
  api(`actions/runs/${runId}/attempts/${attempt}`),
  api(`actions/jobs/${jobId}`),
  api(`actions/artifacts/${artifactId}`),
]);
assert.equal(run.id, runId);
assert.equal(run.run_attempt, attempt);
assert.equal(run.repository.full_name, "ubugeeei-prod/vize");
assert.equal(run.head_sha, sourceHead);
assert.equal(run.head_branch, "gh-readonly-queue/main/pr-7433-451c97d69565a20b44851080de743d537134f473");
assert.equal(run.event, "workflow_dispatch");
assert.equal(run.path, ".github/workflows/check.yml");
assert.equal(run.status, "completed", "whole source workflow must be terminal before transport");
assert.equal(job.id, jobId);
assert.equal(job.run_id, runId);
assert.equal(job.run_attempt, attempt);
assert.equal(job.head_sha, sourceHead);
assert.equal(job.name, "test-scripts");
assert.equal(job.status, "completed");
assert.equal(artifact.id, artifactId);
assert.equal(artifact.workflow_run.id, runId);
assert.equal(artifact.workflow_run.head_sha, sourceHead);
assert.equal(artifact.expired, false);
assert.equal(artifact.name, `formatter-api-corpus-${runId}-${attempt}-test-scripts-full`);
const names = [
  "script",
  "prepared",
  "literal",
  "literal-extra",
  "vue-version",
  "capture",
  "capture-extra",
  "capture-final",
];
const expected = [
  "build-receipt.json",
  "cargo.jsonl",
  "cargo.stderr.txt",
  "cli-history-report.json",
  ...names.map((name) => (name === "script" ? "report" : name + "-report") + ".json"),
].map((name) => "differential/formatter-api/" + name);
expected.push("ci/vize.differential-build.json", "differential/formatter.json");
const selected = new Set(expected.filter((name) => fs.existsSync(path.join(root, name))));
const safePath = (relative: string) => {
  assert(!path.isAbsolute(relative) && !relative.includes("\\") && !relative.includes("\0"));
  assert(relative.split("/").every((part) => part && part !== "." && part !== ".."));
  return path.join(root, relative);
};
const inventory: { path: string; bytes: number; selected: boolean }[] = [];
const walk = (relative: string) => {
  const directory = relative ? safePath(relative) : root;
  if (!fs.existsSync(directory)) return;
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const name = relative ? `${relative}/${entry.name}` : entry.name;
    assert(!entry.isSymbolicLink(), "artifact symlinks are not transported");
    if (entry.isDirectory()) walk(name);
    else {
      assert(entry.isFile());
      inventory.push({
        path: name,
        bytes: fs.statSync(safePath(name)).size,
        selected: selected.has(name),
      });
      assert(inventory.length <= 8192, "complete artifact inventory bound");
    }
  }
};
walk("");
assert(selected.size > 0 && selected.size <= 128, "transport file count bound");
const files = [...selected].sort().map((relative) => {
  const file = safePath(relative);
  assert(fs.lstatSync(file).isFile());
  const bytes = fs.readFileSync(file);
  assert(bytes.length <= 16 * 1024 * 1024, "transport single-file bound");
  return {
    path: relative,
    bytes: bytes.length,
    sha256: digest(bytes),
    base64: bytes.toString("base64"),
  };
});
const bindings: any[] = [];
const bindingErrors: string[] = [];
for (const file of files.filter((file) => file.path.endsWith(".json"))) {
  try {
    const value = JSON.parse(Buffer.from(file.base64, "base64").toString());
    const revision = value.sourceRevision ?? value.source?.sourceRevision;
    if (revision !== undefined) assert.equal(revision, sourceHead, `source mismatch: ${file.path}`);
    bindings.push({ path: file.path, schema: value.schema, sourceRevision: revision ?? null });
  } catch (error) {
    bindingErrors.push(`${file.path}: ${String(error)}`);
  }
}
const observerPath = "differential/formatter-api/formatter_observe";
const binaryHashes: any[] = [];
if (fs.existsSync(safePath(observerPath))) {
  assert(fs.lstatSync(safePath(observerPath)).isFile());
  const bytes = fs.readFileSync(safePath(observerPath));
  binaryHashes.push({
    path: observerPath,
    bytes: bytes.length,
    sha256: digest(bytes),
    transported: false,
  });
  const receiptFile = files.find(
    (file) => file.path === "differential/formatter-api/build-receipt.json",
  );
  if (receiptFile) {
    try {
      assert.equal(
        JSON.parse(Buffer.from(receiptFile.base64, "base64").toString()).artifact.sha256,
        digest(bytes),
      );
    } catch (error) {
      bindingErrors.push(`frozen observer binary: ${String(error)}`);
    }
  }
}
const payload = Buffer.from(
  JSON.stringify({
    schema: "vize.formatter.actual-artifact.transport",
    version: 1,
    source: {
      runId,
      attempt,
      jobId,
      artifactId,
      sourceHead,
      artifactName: artifact.name,
      authenticatedArtifactDigest: artifact.digest,
      sourceRunStatus: run.status,
      sourceRunConclusion: run.conclusion,
      sourceJobConclusion: job.conclusion,
    },
    metadata: { run, job, artifact },
    selection:
      "Complete selected protected queue3290 formatter API/CLI evidence, including standalone CI CLI receipt and ordinary shared formatter raw report. The current uploader preserves differential/** and ci/vize.differential-build.json under common ancestor target. Complete extracted artifact path/size inventory is retained; unrelated product files receive inventory-only transport with no byte or semantic credit. Observer binary is independently rehashed without execution or binary transport. Missing current expected files remain explicit; no data is reconstructed.",
    files,
    inventory: inventory.sort((a, b) => a.path.localeCompare(b.path)),
    binaryHashes,
    bindings,
    bindingErrors,
    missingExpectedFiles: expected.filter((name) => !selected.has(name)),
    claims: {
      productExecutedByTransport: false,
      semanticVerificationPerformed: false,
      wholeFixesClosed: 0,
      nativeHandled: 0,
      nativeEquivalent: 0,
      currentConsumerAcceptance: false,
      literalZipDigestRecomputed: false,
    },
  }),
);
assert(payload.length <= 32 * 1024 * 1024, "transport JSON bound");
const gzip = gzipSync(payload, { level: 9 });
assert(gzip.length <= 2 * 1024 * 1024, "transport gzip bound; split explicitly, never truncate");
const chunks = gzip.toString("base64").match(/.{1,12000}/g) ?? [];
console.log(
  `VIZE_FORMATTER_TRANSPORT_BEGIN ${JSON.stringify({ version: 1, sourceHead, runId, attempt, jobId, artifactId, chunks: chunks.length, files: files.length, jsonBytes: payload.length, jsonSha256: digest(payload), gzipBytes: gzip.length, gzipSha256: digest(gzip) })}`,
);
for (const [index, chunk] of chunks.entries())
  console.log(`VIZE_FORMATTER_TRANSPORT_CHUNK ${index} ${chunk}`);
console.log("VIZE_FORMATTER_TRANSPORT_END");
