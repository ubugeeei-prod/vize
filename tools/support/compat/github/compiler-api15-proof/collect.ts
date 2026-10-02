import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const [transport, source, output] = process.argv.slice(2);
assert(output && transport && source);
assert(!fs.existsSync(output));
fs.mkdirSync(output, { recursive: true });
const revision = "34b1713bc2156929a6087648e028129823db4cef";
const tree = "969ea15a40444314ed6d30d9acdd222d663d3253";
const artifactId = 11204858271;
const artifactBytes = 45765688;
const artifactDigest = "d7aa4701068bdf1b0463a411ce42fa5aac53a2e86b8166490293ec8dddabec85";
const auditorDigest = "dce248eb39bef94a023fa989699c84491e1ca210b91d526ad5663b8c298ce636";
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const git = (...args: string[]) => execFileSync("git", ["--no-replace-objects", ...args], { cwd: source, encoding: "utf8" }).trimEnd();
assert.equal(git("rev-parse", "HEAD"), revision);
assert.equal(git("rev-parse", "HEAD^{tree}"), tree);
assert.equal(git("status", "--porcelain=v1"), "");
assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
assert.match(process.env.GITHUB_SHA ?? "", /^[a-f0-9]{40}$/);
const token = process.env.GITHUB_TOKEN;
assert(token, "ephemeral Actions read-only token is required");
const api = "https://api.github.com/repos/ubugeeei-prod/vize";
async function get(relative: string) {
  assert(relative.startsWith("/actions/"));
  const response = await fetch(api + relative, { headers: { Authorization: `Bearer ${token}`, Accept: "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28" }, signal: AbortSignal.timeout(60_000) });
  assert.equal(response.ok, true, `read-only Actions response ${response.status}`);
  return Buffer.from(await response.arrayBuffer());
}
function write(file: string, bytes: Uint8Array) {
  const target = path.join(output, file);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, bytes, { flag: "wx" });
  return target;
}
const metadataBytes = await get(`/actions/artifacts/${artifactId}`);
const metadata = JSON.parse(metadataBytes.toString());
assert.equal(metadata.id, artifactId);
assert.equal(metadata.name, "formatter-api-corpus-36952506161-1-test-scripts-full");
assert.equal(metadata.size_in_bytes, artifactBytes);
assert.equal(metadata.digest, `sha256:${artifactDigest}`);
assert.equal(metadata.expired, false);
assert.equal(metadata.workflow_run.id, 36952506161);
assert.equal(metadata.workflow_run.head_sha, revision);
assert.equal(metadata.workflow_run.head_branch, "proof/compiler-history-dd6-20261002");
write("authority/artifact.json", metadataBytes);
const zip = await get(`/actions/artifacts/${artifactId}/zip`);
assert.equal(zip.length, artifactBytes);
assert.equal(hash(zip), artifactDigest);
const zipFile = write("artifact.zip", zip);
const crcTable = Array.from({ length: 256 }, (_, index) => {
  let value = index;
  for (let bit = 0; bit < 8; bit++) value = (value & 1) ? (0xedb88320 ^ (value >>> 1)) : (value >>> 1);
  return value >>> 0;
});
const crc32 = (bytes: Uint8Array) => {
  let value = 0xffffffff;
  for (const byte of bytes) value = crcTable[(value ^ byte) & 0xff] ^ (value >>> 8);
  return (value ^ 0xffffffff) >>> 0;
};
let end = -1;
for (let offset = zip.length - 22; offset >= Math.max(0, zip.length - 65557); offset--) {
  if (zip.readUInt32LE(offset) === 0x06054b50 && offset + 22 + zip.readUInt16LE(offset + 20) === zip.length) { end = offset; break; }
}
assert(end >= 0, "complete ZIP end record");
assert.equal(zip.readUInt16LE(end + 4), 0);
assert.equal(zip.readUInt16LE(end + 6), 0);
let entryCount = zip.readUInt16LE(end + 10);
let centralBytes = zip.readUInt32LE(end + 12);
let cursor = zip.readUInt32LE(end + 16);
const safe64 = (offset: number) => { const value = Number(zip.readBigUInt64LE(offset)); assert(Number.isSafeInteger(value) && value >= 0); return value; };
if (entryCount === 0xffff || centralBytes === 0xffffffff || cursor === 0xffffffff) {
  assert.equal(zip.readUInt32LE(end - 20), 0x07064b50);
  assert.equal(zip.readUInt32LE(end - 16), 0);
  assert.equal(zip.readUInt32LE(end - 4), 1);
  const extended = safe64(end - 12);
  assert.equal(zip.readUInt32LE(extended), 0x06064b50);
  assert.equal(zip.readUInt32LE(extended + 16), 0);
  assert.equal(zip.readUInt32LE(extended + 20), 0);
  assert.equal(safe64(extended + 24), safe64(extended + 32));
  entryCount = safe64(extended + 32); centralBytes = safe64(extended + 40); cursor = safe64(extended + 48);
} else assert.equal(zip.readUInt16LE(end + 8), entryCount);
assert.equal(entryCount, 42);
const centralStart = cursor;
assert(cursor + centralBytes <= end);
const seen = new Set<string>();
const census: { path: string; bytes: number; compressedBytes: number; crc32: string; sha256: string; transported: boolean }[] = [];
let totalBytes = 0;
for (let index = 0; index < entryCount; index++) {
  assert.equal(zip.readUInt32LE(cursor), 0x02014b50);
  const flags = zip.readUInt16LE(cursor + 8), method = zip.readUInt16LE(cursor + 10), crc = zip.readUInt32LE(cursor + 16);
  assert.equal(flags & 1, 0, "encrypted entries are unsupported");
  assert([0, 8].includes(method));
  let compressedBytes = zip.readUInt32LE(cursor + 20), bytes = zip.readUInt32LE(cursor + 24), local = zip.readUInt32LE(cursor + 42);
  const nameBytes = zip.readUInt16LE(cursor + 28), extraBytes = zip.readUInt16LE(cursor + 30), commentBytes = zip.readUInt16LE(cursor + 32);
  const nameBuffer = zip.subarray(cursor + 46, cursor + 46 + nameBytes);
  const name = nameBuffer.toString("utf8");
  assert(nameBuffer.equals(Buffer.from(name)));
  assert(!path.isAbsolute(name) && !name.includes("\\") && !/[\0*?\[\]:]/.test(name) && !name.split("/").some(part => ["", ".", ".."].includes(part)));
  assert(!seen.has(name)); seen.add(name);
  const extraEnd = cursor + 46 + nameBytes + extraBytes;
  assert(extraEnd + commentBytes <= centralStart + centralBytes);
  if (bytes === 0xffffffff || compressedBytes === 0xffffffff || local === 0xffffffff) {
    let extra = cursor + 46 + nameBytes;
    while (extra + 4 <= extraEnd && zip.readUInt16LE(extra) !== 1) extra += 4 + zip.readUInt16LE(extra + 2);
    assert(extra + 4 <= extraEnd && zip.readUInt16LE(extra) === 1);
    const limit = extra + 4 + zip.readUInt16LE(extra + 2); extra += 4;
    for (const field of ["bytes", "compressedBytes", "local"]) {
      if ((field === "bytes" ? bytes : field === "compressedBytes" ? compressedBytes : local) === 0xffffffff) {
        assert(extra + 8 <= limit && limit <= extraEnd); const value = safe64(extra); extra += 8;
        if (field === "bytes") bytes = value; else if (field === "compressedBytes") compressedBytes = value; else local = value;
      }
    }
  }
  assert(bytes <= 128 * 1024 * 1024);
  assert.equal(zip.readUInt32LE(local), 0x04034b50);
  assert.equal(zip.readUInt16LE(local + 8), method);
  const localNameBytes = zip.readUInt16LE(local + 26), localExtraBytes = zip.readUInt16LE(local + 28);
  assert(zip.subarray(local + 30, local + 30 + localNameBytes).equals(nameBuffer));
  assert(local + 30 + localNameBytes + localExtraBytes + compressedBytes <= centralStart);
  const extracted = spawnSync("unzip", ["-p", zipFile, name], { maxBuffer: 128 * 1024 * 1024 });
  assert.equal(extracted.status, 0, extracted.stderr?.toString());
  assert.equal(extracted.stdout.length, bytes);
  assert.equal(crc32(extracted.stdout), crc);
  totalBytes += bytes; assert(totalBytes <= 512 * 1024 * 1024);
  const text = extracted.stdout.toString("utf8");
  const transported = !extracted.stdout.includes(0) && extracted.stdout.equals(Buffer.from(text));
  if (transported) write(`artifact/${name}`, extracted.stdout);
  census.push({ path: name, bytes, compressedBytes, crc32: crc.toString(16).padStart(8, "0"), sha256: hash(extracted.stdout), transported });
  cursor = extraEnd + commentBytes;
}
assert.equal(cursor, centralStart + centralBytes);
const zipTest = spawnSync("unzip", ["-t", zipFile], { encoding: "utf8", maxBuffer: 1024 * 1024 });
assert.equal(zipTest.status, 0, zipTest.stderr);
write("authority/zip-crc-test.log", Buffer.from(zipTest.stdout + zipTest.stderr));
write("authority/zip-census.json", Buffer.from(JSON.stringify({ schema: "vize.compiler.api15.zip-census", version: 1, artifactId, literalZipBytes: zip.length, literalZipSha256: hash(zip), entryCount, totalUncompressedBytes: totalBytes, entries: census.sort((a, b) => a.path.localeCompare(b.path)) }, null, 2) + "\n"));
const runBytes = await get("/actions/runs/36952506161");
const jobsBytes = await get("/actions/runs/36952506161/attempts/1/jobs?per_page=100");
const logBytes = await get("/actions/jobs/110668305314/logs");
const runFile = write("authority/original-run.json", runBytes), jobsFile = write("authority/original-attempt1-jobs.json", jobsBytes), logFile = write("authority/original-test-scripts.log", logBytes);
assert.equal(hash(logBytes), "0eded1ac0a798d219ba6b2fc5672a614b5da45781ff6ea7e764ae9a79913b626");
const frozenSource = "/private/tmp/vize-compiler-history-foundation-main-20261002";
const review = "/private/tmp/vize-compiler-dd6-review-20261002";
assert(!fs.existsSync(frozenSource) && !fs.existsSync(review));
fs.symlinkSync(source, frozenSource, "dir");
fs.mkdirSync(path.join(review, "closure-3"), { recursive: true });
for (const relative of ["package.json", "tests/differential", "tests/_fixtures/differential/compiler", "crates/vize_atelier_sfc/tests/fixtures/fix-history", "crates/vize_atelier_ssr/tests/fixtures/fix-history-next", "crates/vize_atelier_vapor/tests/fixtures/fix-history-next"]) fs.cpSync(path.join(source, relative), path.join(review, "closure-3", relative), { recursive: true });
fs.copyFileSync(path.join(transport, "closure-receipt.json"), path.join(review, "closure-3-receipt.json"));
fs.mkdirSync(path.join(review, "temporary-proof-campaign"));
fs.copyFileSync(path.join(transport, "plan.json"), path.join(review, "temporary-proof-campaign/plan.json"));
const auditor = path.join(transport, "audit.ts");
assert.equal(hash(fs.readFileSync(auditor)), auditorDigest);
const auditFile = path.join(output, "authority/api15-hosted-audit.json");
const audited = spawnSync(process.execPath, [auditor, zipFile, runFile, jobsFile, path.join(output, "authority/artifact.json"), logFile, auditFile], { encoding: "utf8", maxBuffer: 4 * 1024 * 1024 });
write("authority/auditor.log", Buffer.from(audited.stdout + audited.stderr));
assert.equal(audited.status, 0, audited.stderr);
const accepted = JSON.parse(fs.readFileSync(auditFile, "utf8"));
assert.equal(accepted.actualApiCasesAtThisExactSource, 15);
assert.equal(accepted.fullWorkflow.accepted, false);
assert.equal(accepted.toolingJob.accepted, false);
assert.equal(accepted.fullHistoryClosed, false);
assert.equal(accepted.nativeHandled, 0);
write("authority/transport.json", Buffer.from(JSON.stringify({ schema: "vize.compiler.api15.hosted-artifact-transport", version: 1, sourceRevision: revision, sourceTree: tree, diagnosticRevision: process.env.GITHUB_SHA, repository: process.env.GITHUB_REPOSITORY, runId: Number(process.env.GITHUB_RUN_ID), runAttempt: Number(process.env.GITHUB_RUN_ATTEMPT), job: process.env.GITHUB_JOB, artifactId, literalZipBytes: artifactBytes, literalZipSha256: artifactDigest, auditorSha256: auditorDigest, originalRunId: 36952506161, originalAttempt: 1, originalJobId: 110668305314, completeCensusEntries: 42, hostedLiteralZipAndEveryEntryCrcHashVerified: true, hostedSourceBoundAuditAcceptedCases: 15, localReceiverRawZipAndBinaryBytesTransferred: false, completeTextualArtifactEntriesTransferred: census.filter(entry => entry.transported).length, omittedBinaryEntries: census.filter(entry => !entry.transported), fullWorkflowAccepted: false, toolingJobAccepted: false, wholeHistoryClosed: false, nativeHandled: 0, nativeEquivalent: 0, readOnlyRemoteGetMethods: true, cargoOrObserverExecution: false }, null, 2) + "\n"));
console.log(JSON.stringify({ source: revision, hostedAuditedApiCases: 15, zipEntries: 42, fullAccepted: false, nativeHandled: 0 }));
