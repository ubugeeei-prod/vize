import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { inflateRawSync } from "node:zlib";
import { actualTestcase } from "./testcase.ts";

const [specFile, output] = process.argv.slice(2);
assert(specFile && output && !fs.existsSync(output));
const spec = JSON.parse(fs.readFileSync(specFile, "utf8"));
assert.equal(spec.repository, "ubugeeei-prod/vize");
assert.equal(spec.cohort, "pull-request");
assert.equal(spec.artifacts.length, 1);
assert.equal(process.env.GITHUB_REPOSITORY, spec.repository);
const token = process.env.GITHUB_TOKEN;
assert(token, "ephemeral read-only Actions token required");
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const api = `https://api.github.com/repos/${spec.repository}`;
async function get(relative: string) {
  assert(/^\/(?:actions|git)\//.test(relative));
  const response = await fetch(api + relative, {
    headers: { Authorization: `Bearer ${token}`, Accept: "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28" },
    signal: AbortSignal.timeout(30_000),
  });
  assert(response.ok, `read-only response ${response.status}`);
  return Buffer.from(await response.arrayBuffer());
}
const files: { path: string; bytes: number; sha256: string; base64: string }[] = [];
function retain(name: string, bytes: Buffer) {
  assert(!path.isAbsolute(name) && !name.includes("\\") && !name.includes("\0"));
  assert(name.split("/").every(part => part && part !== "." && part !== ".."));
  assert(!files.some(file => file.path === name));
  files.push({ path: name, bytes: bytes.length, sha256: hash(bytes), base64: bytes.toString("base64") });
}
const runBytes = await get(`/actions/runs/${spec.runId}/attempts/${spec.attempt}`);
const run = JSON.parse(runBytes.toString());
assert.equal(run.id, spec.runId);
assert.equal(run.run_attempt, spec.attempt);
assert.equal(run.head_sha, spec.source);
assert.equal(run.head_branch, spec.branch);
assert.equal(run.event, "pull_request");
assert.equal(run.path, ".github/workflows/check.yml");
assert.equal(run.status, "completed");
assert.equal(run.conclusion, "success");
retain("authority/source-run.json", runBytes);
const commitBytes = await get(`/git/commits/${spec.source}`);
assert.equal(JSON.parse(commitBytes.toString()).tree.sha, spec.tree);
retain("authority/source-commit.json", commitBytes);
const candidate = JSON.parse(commitBytes.toString());
assert.deepEqual(candidate.parents.map((parent: { sha: string }) => parent.sha), [spec.base]);
assert.equal(candidate.sha, spec.source);
assert.equal(spec.originalSource, "a04fdf81bad61ad32d1bd96a7cfd816e4796cb70");
const originalBytes = await get(`/git/commits/${spec.originalSource}`);
const original = JSON.parse(originalBytes.toString());
assert.equal(original.tree.sha, "c2c45173fc4ea26571f3583ade1e1b0dad35c99c");
retain("authority/original-pr-source.json", originalBytes);
for (const owned of spec.ownedCaseBlobs) {
  let tree = spec.tree;
  const parts = owned.path.split("/");
  for (const [index, part] of parts.entries()) {
    const treeBytes = await get(`/git/trees/${tree}`);
    const data = JSON.parse(treeBytes.toString());
    assert.equal(data.sha, tree);
    assert.equal(data.truncated, false);
    const matches = data.tree.filter((entry: any) => entry.path === part);
    assert.equal(matches.length, 1);
    retain(`authority/case-${owned.label}-tree-${index}.json`, treeBytes);
    tree = matches[0].sha;
    assert.equal(matches[0].type, index === parts.length - 1 ? "blob" : "tree");
  }
  assert.equal(tree, owned.sha, "actual source-owned Drop body byte-identical to reviewed provider");
}
const jobs: any[] = [];
let total = -1;
for (let page = 1; page <= 10; page++) {
  const bytes = await get(`/actions/runs/${spec.runId}/attempts/${spec.attempt}/jobs?per_page=100&page=${page}`);
  const data = JSON.parse(bytes.toString());
  if (total < 0) total = data.total_count;
  assert.equal(data.total_count, total);
  jobs.push(...data.jobs);
  retain(`authority/jobs-page-${page}.json`, bytes);
  if (jobs.length === total) break;
  assert(data.jobs.length === 100 && jobs.length < total);
}
assert.equal(jobs.length, total, "complete attempt inventory");
assert.equal(new Set(jobs.map(job => job.id)).size, total);
assert(jobs.every(job => job.status === "completed" && ["success", "skipped"].includes(job.conclusion)));
const crcTable = Array.from({ length: 256 }, (_, index) => {
  let value = index;
  for (let bit = 0; bit < 8; bit++) value = value & 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
  return value >>> 0;
});
function crc32(bytes: Uint8Array) {
  let value = 0xffffffff;
  for (const byte of bytes) value = crcTable[(value ^ byte) & 255] ^ (value >>> 8);
  return (value ^ 0xffffffff) >>> 0;
}
function zipEntries(zip: Buffer) {
  let end = -1;
  for (let offset = zip.length - 22; offset >= Math.max(0, zip.length - 65557); offset--) {
    if (zip.readUInt32LE(offset) === 0x06054b50 && offset + 22 + zip.readUInt16LE(offset + 20) === zip.length) { end = offset; break; }
  }
  assert(end >= 0, "complete ZIP end record");
  assert.equal(zip.readUInt16LE(end + 4), 0);
  assert.equal(zip.readUInt16LE(end + 6), 0);
  const count = zip.readUInt16LE(end + 10);
  assert.equal(zip.readUInt16LE(end + 8), count);
  assert(count > 0 && count < 100 && count !== 0xffff);
  const centralBytes = zip.readUInt32LE(end + 12);
  const start = zip.readUInt32LE(end + 16);
  assert.equal(start + centralBytes, end);
  let cursor = start;
  const entries: { path: string; bytes: Buffer; crc32: string; compressedBytes: number }[] = [];
  let totalBytes = 0;
  for (let index = 0; index < count; index++) {
    assert.equal(zip.readUInt32LE(cursor), 0x02014b50);
    const flags = zip.readUInt16LE(cursor + 8), method = zip.readUInt16LE(cursor + 10);
    assert.equal(flags & 1, 0);
    assert([0, 8].includes(method));
    const crc = zip.readUInt32LE(cursor + 16), compressed = zip.readUInt32LE(cursor + 20), size = zip.readUInt32LE(cursor + 24);
    assert(size < 8 * 1024 * 1024 && compressed < zip.length);
    const nameLength = zip.readUInt16LE(cursor + 28), extra = zip.readUInt16LE(cursor + 30), comment = zip.readUInt16LE(cursor + 32);
    const local = zip.readUInt32LE(cursor + 42), nameBytes = zip.subarray(cursor + 46, cursor + 46 + nameLength);
    const name = nameBytes.toString("utf8");
    assert(nameBytes.equals(Buffer.from(name)) && !path.isAbsolute(name) && !name.includes("\\") && !/[\0*?\[\]:]/.test(name));
    assert(name.split("/").every(part => part && part !== "." && part !== ".."));
    assert(!entries.some(entry => entry.path === name));
    assert.equal(zip.readUInt32LE(local), 0x04034b50);
    assert.equal(zip.readUInt16LE(local + 6), flags);
    assert.equal(zip.readUInt16LE(local + 8), method);
    const localName = zip.readUInt16LE(local + 26), localExtra = zip.readUInt16LE(local + 28);
    assert(zip.subarray(local + 30, local + 30 + localName).equals(nameBytes));
    const offset = local + 30 + localName + localExtra;
    assert(offset + compressed <= start);
    const packed = zip.subarray(offset, offset + compressed);
    const bytes = method === 0 ? packed : inflateRawSync(packed, { maxOutputLength: 8 * 1024 * 1024 });
    assert.equal(bytes.length, size);
    assert.equal(crc32(bytes), crc);
    totalBytes += size;
    assert(totalBytes <= 16 * 1024 * 1024);
    entries.push({ path: name, bytes, compressedBytes: compressed, crc32: crc.toString(16).padStart(8, "0") });
    cursor += 46 + nameLength + extra + comment;
    assert(cursor <= start + centralBytes);
  }
  assert.equal(cursor, start + centralBytes);
  return entries;
}
const proofs = [];
for (const expected of spec.artifacts) {
  const matching = jobs.filter(job => job.id === expected.jobId);
  assert.equal(matching.length, 1);
  const job = matching[0];
  assert.equal(job.run_id, spec.runId);
  assert.equal(job.run_attempt, spec.attempt);
  assert.equal(job.head_sha, spec.source);
  assert.equal(job.conclusion, "success");
  assert.equal(job.name, expected.jobName);
  for (const name of ["Verify Rust archive identity", "Run Rust test shard without rebuilding", "Upload Rust shard results"]) {
    assert.equal(job.steps.filter((step: any) => step.name === name && step.conclusion === "success").length, 1);
  }
  const metadataBytes = await get(`/actions/artifacts/${expected.id}`);
  const metadata = JSON.parse(metadataBytes.toString());
  assert.equal(metadata.id, expected.id);
  assert.equal(metadata.name, expected.name);
  assert.equal(metadata.size_in_bytes, expected.bytes);
  assert.equal(metadata.digest, `sha256:${expected.sha256}`);
  assert.equal(metadata.expired, false);
  assert.equal(metadata.workflow_run.id, spec.runId);
  assert.equal(metadata.workflow_run.head_sha, spec.source);
  assert.equal(metadata.workflow_run.head_branch, spec.branch);
  const zip = await get(`/actions/artifacts/${expected.id}/zip`);
  assert.equal(zip.length, expected.bytes);
  assert.equal(hash(zip), expected.sha256);
  const entries = zipEntries(zip);
  const xmls = entries.filter(entry => entry.path === "junit.xml");
  assert.equal(xmls.length, 1, "one authentic complete JUnit");
  const xml = xmls[0].bytes.toString("utf8");
  assert(xmls[0].bytes.equals(Buffer.from(xml)));
  const actual = actualTestcase(xml, expected.case, expected.binary);
  const logBytes = await get(`/actions/jobs/${expected.jobId}/logs`);
  const log = logBytes.toString().replace(/\u001b\[[\d;]*m/g, "");
  const pass = log.split("\n").filter(line => line.includes("PASS") && line.includes(expected.binary) && line.includes(expected.case));
  assert.equal(pass.length, 1, "authentic named worker PASS");
  assert(log.split("\n").some(line => /Z\s+NEXTEST_PROFILE: pr$/.test(line)), "actual original PR-profile runtime envelope");
  const prefix = `artifact-${expected.id}`;
  retain(`${prefix}/service-metadata.json`, metadataBytes);
  retain(`${prefix}/literal.zip`, zip);
  retain(`${prefix}/worker.log`, logBytes);
  for (const entry of entries) retain(`${prefix}/entries/${entry.path}`, entry.bytes);
  proofs.push({ artifactId: expected.id, jobId: expected.jobId, shard: expected.shard, literalZipSha256: hash(zip), literalZipBytes: zip.length,
    entries: entries.map(entry => ({ path: entry.path, bytes: entry.bytes.length, sha256: hash(entry.bytes), crc32: entry.crc32, compressedBytes: entry.compressedBytes })),
    completeJUnitTestcases: actual.totalCases, case: expected.case, binary: expected.binary, testcase: actual.testcase, namedPass: pass[0] });
}
retain("authority/spec.json", fs.readFileSync(specFile));
retain("authority/proof.json", Buffer.from(JSON.stringify({ schema: "vize.embedding-nextest.actual-junit-proof", version: 1, spec, proofs,
  diagnostic: { source: process.env.GITHUB_SHA, runId: process.env.GITHUB_RUN_ID, attempt: process.env.GITHUB_RUN_ATTEMPT },
  literalZipAndEveryEntryCRCVerified: true, originalFullJUnitPreserved: true, originalWorkerExecutedCases: 1,
  testOrProductExecutedByTransport: false, readOnlyRemoteGetMethods: true, mergeQueueAccepted: false }, null, 2) + "\n"));
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(path.join(output, "payload.json"), JSON.stringify({ schema: 1, label: "embedding-nextest-junit-proof", files }), { flag: "wx" });
console.log(JSON.stringify({ originalRun: spec.runId, originalAttempt: spec.attempt, proofs: proofs.map(proof => ({ job: proof.jobId, artifact: proof.artifactId, case: proof.case, junitCases: proof.completeJUnitTestcases })) }));
