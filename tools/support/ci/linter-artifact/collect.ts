import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { zipEntries } from "./zip.ts";

// Temporary read-only transport of actual existing source-built observations.
// The original product never runs here. Complete bytes are not reconstructed.
const [specFile, output] = process.argv.slice(2);
assert(specFile && output && !fs.existsSync(output));
const spec = JSON.parse(fs.readFileSync(specFile, "utf8"));
assert.equal(spec.repository, "ubugeeei-prod/vize");
assert.equal(spec.source, "9c08df5812d24f164254abfa727b02c9d583c41f");
assert.equal(spec.runId, 36967214570);
assert.equal(spec.attempt, 1);
assert.equal(spec.jobId, 110713462355);
assert.equal(spec.artifactId, 11210836312);
assert.equal(
  spec.artifactSha256,
  "709774db6915323ff5a87f14bb564398928e36efef0ea351b7f26e6ca222fb1d",
);
assert.equal(process.env.GITHUB_REPOSITORY, spec.repository);
const token = process.env.GITHUB_TOKEN;
assert(token, "ephemeral read-only Actions token required");
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
async function get(relative: string) {
  assert(/^\/(?:actions|git)\//.test(relative));
  const response = await fetch(`https://api.github.com/repos/${spec.repository}${relative}`, {
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
    },
    signal: AbortSignal.timeout(30_000),
  });
  assert(response.ok, `read-only response ${response.status}`);
  return Buffer.from(await response.arrayBuffer());
}
const files: { path: string; bytes: number; sha256: string; base64: string }[] = [];
function retain(name: string, bytes: Buffer) {
  assert(!path.isAbsolute(name) && !name.includes("\\") && !name.includes("\0"));
  assert(name.split("/").every((part) => part && part !== "." && part !== ".."));
  assert(!files.some((file) => file.path === name));
  assert(bytes.length <= 16 * 1024 * 1024, "complete selected file bound; never truncate");
  files.push({
    path: name,
    bytes: bytes.length,
    sha256: hash(bytes),
    base64: bytes.toString("base64"),
  });
}
const runBytes = await get(`/actions/runs/${spec.runId}/attempts/${spec.attempt}`);
const run = JSON.parse(runBytes.toString());
assert.equal(run.id, spec.runId);
assert.equal(run.run_attempt, spec.attempt);
assert.equal(run.repository.full_name, spec.repository);
assert.equal(run.head_sha, spec.source);
assert.equal(run.head_branch, spec.branch);
assert.equal(run.event, "workflow_dispatch");
assert.equal(run.path, ".github/workflows/check.yml");
assert.equal(run.status, "completed");
assert.equal(run.conclusion, "success");
retain("authority/source-run.json", runBytes);
const commitBytes = await get(`/git/commits/${spec.source}`);
const commit = JSON.parse(commitBytes.toString());
assert.equal(commit.sha, spec.source);
assert.equal(commit.tree.sha, spec.tree);
retain("authority/source-commit.json", commitBytes);
const blobBytes = await get(`/git/blobs/${spec.casesBlob}`);
const blob = JSON.parse(blobBytes.toString());
assert.equal(blob.sha, spec.casesBlob);
assert.equal(blob.encoding, "base64");
const casesBytes = Buffer.from(blob.content, "base64");
assert.equal(hash(casesBytes), spec.casesSha256);
assert.equal(
  createHash("sha1").update(`blob ${casesBytes.length}\0`).update(casesBytes).digest("hex"),
  spec.casesBlob,
);
retain("authority/source-cases-blob.json", blobBytes);
retain("authority/component-name-cases.json", casesBytes);
const jobs: any[] = [];
let total = -1;
for (let page = 1; page <= 10; page++) {
  const bytes = await get(
    `/actions/runs/${spec.runId}/attempts/${spec.attempt}/jobs?per_page=100&page=${page}`,
  );
  const data = JSON.parse(bytes.toString());
  if (total < 0) total = data.total_count;
  assert.equal(data.total_count, total);
  jobs.push(...data.jobs);
  retain(`authority/jobs-page-${page}.json`, bytes);
  if (jobs.length === total) break;
  assert(data.jobs.length === 100 && jobs.length < total);
}
assert.equal(jobs.length, total, "complete original attempt inventory");
assert.equal(new Set(jobs.map((job) => job.id)).size, total);
assert(
  jobs.every(
    (job) => job.status === "completed" && ["success", "skipped"].includes(job.conclusion),
  ),
);
const matching = jobs.filter((job) => job.id === spec.jobId);
assert.equal(matching.length, 1);
const job = matching[0];
assert.equal(job.run_id, spec.runId);
assert.equal(job.run_attempt, spec.attempt);
assert.equal(job.head_sha, spec.source);
assert.equal(job.name, spec.jobName);
assert.equal(job.status, "completed");
assert.equal(job.conclusion, "success");
for (const name of [
  "Build vize CLI",
  "Test release scripts",
  "Run ./.github/actions/upload-formatter-api-corpus-evidence",
])
  assert.equal(
    job.steps.filter((step: any) => step.name === name && step.conclusion === "success").length,
    1,
  );
const metadataBytes = await get(`/actions/artifacts/${spec.artifactId}`);
const metadata = JSON.parse(metadataBytes.toString());
assert.equal(metadata.id, spec.artifactId);
assert.equal(metadata.name, spec.artifactName);
assert.equal(metadata.size_in_bytes, spec.artifactBytes);
assert.equal(metadata.digest, `sha256:${spec.artifactSha256}`);
assert.equal(metadata.expired, false);
assert.equal(metadata.workflow_run.id, spec.runId);
assert.equal(metadata.workflow_run.head_sha, spec.source);
assert.equal(metadata.workflow_run.head_branch, spec.branch);
retain("authority/service-metadata.json", metadataBytes);
const zip = await get(`/actions/artifacts/${spec.artifactId}/zip`);
assert.equal(zip.length, spec.artifactBytes);
assert.equal(hash(zip), spec.artifactSha256);
const entries = zipEntries(zip);
const prefix = "differential/linter-api/";
const required = [
  "component-name-original-capture.json",
  "build-receipt.json",
  "report.json",
  "cargo.jsonl",
  "cargo.stderr.txt",
];
for (const name of required)
  assert.equal(entries.filter((entry) => entry.path === prefix + name).length, 1);
const inventory = entries.map((entry) => {
  const binary = entry.bytes.subarray(0, 4).equals(Buffer.from([0x7f, 69, 76, 70]));
  const selected = entry.path.startsWith(prefix) && !binary;
  if (selected) {
    assert(
      entry.bytes.equals(Buffer.from(entry.bytes.toString("utf8"))),
      "selected source/text must remain lossless UTF8",
    );
    assert(!entry.bytes.includes(0), "no binary may masquerade as selected text");
    retain(entry.path, entry.bytes);
  }
  return {
    path: entry.path,
    bytes: entry.bytes.length,
    sha256: hash(entry.bytes),
    crc32: entry.crc32,
    compressedBytes: entry.compressedBytes,
    selected,
    binary,
  };
});
const binary = inventory.filter((entry) => entry.path === prefix + "lint_history_observer");
assert.equal(binary.length, 1);
assert.equal(binary[0].binary, true);
const receipt = JSON.parse(
  entries.find((entry) => entry.path === prefix + "build-receipt.json")!.bytes.toString(),
);
assert.equal(receipt.source.sourceRevision, spec.source);
assert.equal(receipt.source.sourceTree, spec.tree);
assert.equal(receipt.artifact.sha256, binary[0].sha256);
const capture = JSON.parse(
  entries
    .find((entry) => entry.path === prefix + "component-name-original-capture.json")!
    .bytes.toString(),
);
assert.equal(capture.sourceRevision, spec.source);
assert.deepEqual(capture.buildReceipt, receipt);
assert.equal(capture.rows.length, 8);
retain("authority/original-worker.log", await get(`/actions/jobs/${spec.jobId}/logs`));
retain("authority/spec.json", fs.readFileSync(specFile));
retain(
  "authority/proof.json",
  Buffer.from(
    JSON.stringify(
      {
        schema: "vize.linter-original-artifact.transport-proof",
        version: 1,
        spec,
        literalZipSha256: hash(zip),
        literalZipBytes: zip.length,
        inventory,
        hostBinary: { ...binary[0], transported: false, receiptBound: true },
        originalSourceJob: { id: job.id, conclusion: job.conclusion },
        originalRun: { status: run.status, conclusion: run.conclusion },
        transport: {
          source: process.env.GITHUB_SHA,
          runId: process.env.GITHUB_RUN_ID,
          attempt: process.env.GITHUB_RUN_ATTEMPT,
        },
        claims: {
          literalZipAndEveryEntryCRCVerified: true,
          productExecutedByTransport: false,
          semanticVerificationPerformed: false,
          nativeHandled: 0,
          nativeEquivalent: 0,
          originalWholeRunSucceeded: run.status === "completed" && run.conclusion === "success",
          originalSourceJobSucceeded: true,
          localBinaryTransported: false,
          readOnlyRemoteGetMethods: true,
        },
      },
      null,
      2,
    ) + "\n",
  ),
);
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(
  path.join(output, "payload.json"),
  JSON.stringify({ schema: 1, label: "linter-original-capture-proof", files }),
  { flag: "wx" },
);
console.log(
  JSON.stringify({
    originalRun: spec.runId,
    originalAttempt: spec.attempt,
    sourceJob: job.id,
    artifact: metadata.id,
    literalZipSha256: hash(zip),
    entries: entries.length,
    selected: files.length,
    binarySha256: binary[0].sha256,
    productExecuted: false,
  }),
);
