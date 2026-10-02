import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

// Preserve complete textual partial evidence on either audit outcome.
// The raw ZIP and omitted binary bytes are never selected for this upload.
const [source, output] = process.argv.slice(2);
assert(source && output && !fs.existsSync(output));
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const status = {
  schema: "vize.compiler.api15.diagnostic-step-status", version: 1,
  repository: process.env.GITHUB_REPOSITORY,
  diagnosticRevision: process.env.GITHUB_SHA,
  runId: Number(process.env.GITHUB_RUN_ID),
  runAttempt: Number(process.env.GITHUB_RUN_ATTEMPT),
  job: process.env.GITHUB_JOB,
  auditStepOutcome: process.env.PROOF_AUDIT_OUTCOME,
  framesStepOutcome: process.env.PROOF_FRAMES_OUTCOME,
  priorJobStatus: process.env.PROOF_JOB_STATUS,
  originalSourceRevision: "34b1713bc2156929a6087648e028129823db4cef",
  originalArtifactId: 11204858271,
  originalZipBytes: 45765688,
  originalZipSha256: "d7aa4701068bdf1b0463a411ce42fa5aac53a2e86b8166490293ec8dddabec85",
  rawZipAndBinaryBytesTransferred: false,
  boundedPartialUploadIsNotArtifactAcceptance: true,
  fullWorkflowAccepted: false, wholeHistoryClosed: false, nativeHandled: 0,
};
assert.equal(status.repository, "ubugeeei-prod/vize");
assert.match(status.diagnosticRevision ?? "", /^[a-f0-9]{40}$/);
assert(Number.isSafeInteger(status.runId) && status.runId > 0);
assert(Number.isSafeInteger(status.runAttempt) && status.runAttempt > 0);
for (const directory of [source, path.join(source, "authority")]) {
  const stat = fs.lstatSync(directory, { throwIfNoEntry: false });
  assert(!stat?.isSymbolicLink(), "partial evidence never follows symlinks before status write");
  assert(!stat || stat.isDirectory());
}
fs.mkdirSync(path.join(source, "authority"), { recursive: true });
fs.writeFileSync(path.join(source, "authority/diagnostic-step-status.json"), JSON.stringify(status, null, 2) + "\n", { flag: "wx" });
const selected: { path: string; bytes: number; sha256: string }[] = [];
let bytes = 0;
function walk(relative: string) {
  const absolute = path.join(source, relative);
  const stat = fs.lstatSync(absolute);
  assert(!stat.isSymbolicLink(), "partial evidence never follows symlinks");
  if (stat.isDirectory()) {
    for (const name of fs.readdirSync(absolute).sort()) {
      assert(![".", ".."].includes(name) && !name.includes("\\"));
      walk(path.posix.join(relative, name));
    }
    return;
  }
  assert(stat.isFile());
  const content = fs.readFileSync(absolute);
  assert(!content.includes(0) && content.equals(Buffer.from(content.toString("utf8"))), "only complete UTF8 textual evidence is uploaded");
  bytes += content.length;
  assert(bytes <= 16 * 1024 * 1024);
  assert(selected.length < 127);
  const destination = path.join(output, relative);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.writeFileSync(destination, content, { flag: "wx" });
  selected.push({ path: relative, bytes: content.length, sha256: hash(content) });
}
for (const root of ["authority", "artifact"]) if (fs.existsSync(path.join(source, root))) walk(root);
const census = Buffer.from(JSON.stringify({ schema: "vize.compiler.api15.bounded-partial-upload", version: 1, source: status, completeSelectedTextBytes: bytes, completeSelectedTextFiles: selected.length, limits: { files: 128, bytes: 16 * 1024 * 1024 }, selected, rawZipAndBinaryBytesTransferred: false, acceptanceClaim: false }, null, 2) + "\n");
assert(bytes + census.length <= 16 * 1024 * 1024);
fs.writeFileSync(path.join(output, "bounded-upload-census.json"), census, { flag: "wx" });
console.log(JSON.stringify({ output, files: selected.length + 1, bytes: bytes + census.length, auditStepOutcome: status.auditStepOutcome, artifactAcceptance: 0 }));
