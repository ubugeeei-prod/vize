import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { posix } from "node:path";
import { zipEntries, readZipEntry } from "../../archive/zip.ts";
import { boundedBytes } from "../release/quota/http.ts";
import { authenticateRelease, type GithubRead } from "./authority.ts";
import { authenticatedBlob } from "./source-admission.ts";
import type { Request, ReleaseAuthority, UnqualifiedCapture } from "./model.ts";
import { discard, type Sink } from "./evidence.ts";
import { array, bytes, gitBlob, object, sha256, sourcePath, text, wholeJson } from "./value.ts";
import {
  requireCaptureJudgment,
  validateBeforeCaptureRecord,
  type CapturePolicyEntry,
} from "./before-capture-record.ts";
export const capturePolicyPath =
  "tools/support/release/human_reports_11252_11253_public/before-capture-policy.json.txt";
export interface CapturedBeforeAdmission {
  kind: "CAPTURE_ONLY";
  capture: UnqualifiedCapture;
  beforeJudgment: Record<string, unknown>;
  accepted: false;
  productQualified: false;
  afterQualified: false;
  issueQualified: false;
  completeInstalledFileSetQualified: false;
  outcomeAssessed: false;
}
export function bindCapturePolicy(
  raw: Buffer,
  request: Request,
  authority: ReleaseAuthority,
): CapturePolicyEntry {
  assert.equal(request.mode, "UNQUALIFIED_AFTER");
  assert.equal(request.release.phase, "after");
  assert.equal(request.release.version, authority.version);
  assert.equal(request.release.releaseRun, authority.run);
  assert.equal(request.publicBefore ?? null, null);
  const rows = request.reviewedController.files.filter((f) => f.path === capturePolicyPath);
  assert.equal(rows.length, 1);
  const row = rows[0]!;
  assert.equal(raw.length, row.bytes);
  assert.equal(sha256(raw), row.sha256);
  assert.equal(gitBlob(raw), row.gitBlob);
  const policy = object(wholeJson(raw));
  assert.equal(policy.schema, "vize.humanReportPublic.beforeCapturePolicy.v1");
  assert.deepEqual(policy.target, {
    version: authority.version,
    releaseRun: authority.run,
    workflowHead: authority.workflowHead,
    source: authority.source,
    candidate: authority.candidate,
    capsuleBlob: request.release.capsuleBlob,
  });
  const matching = array(policy.roles)
    .map(object)
    .filter((r) => r.role === request.role);
  assert.equal(matching.length, 1);
  const entry = matching[0] as unknown as CapturePolicyEntry;
  assert.equal(entry.issue, request.fixture.issue);
  assert.equal(entry.issue, request.release.issue);
  assert.equal(entry.fixtureReceiptSha256, request.fixture.receiptSha256);
  assert.equal(entry.reference.kind, "CAPTURE_ONLY");
  assert.equal(entry.reference.captureMember, "unqualified-capture.json");
  assert.deepEqual(entry.reference, request.capturedBefore);
  requireCaptureJudgment(entry);
  return entry;
}
export function decodeCapturedArchive(
  archive: Buffer,
  entry: CapturePolicyEntry,
): Map<string, Buffer> {
  assert.equal(archive.length, entry.artifact.compressedBytes);
  assert(archive.length <= 12 * 1024 ** 2);
  assert.equal("sha256:" + sha256(archive), entry.reference.archiveDigest);
  const entries = zipEntries(archive);
  assert.equal(entries.length, entry.artifact.entries);
  assert(entries.length > 0 && entries.length <= 4096);
  assert.equal(new Set(entries.map((e) => e.name)).size, entries.length);
  const decoded = entries.reduce((sum, e) => sum + e.size, 0);
  assert.equal(decoded, entry.artifact.decodedBytes);
  assert(decoded <= 96 * 1024 ** 2);
  const members = new Map<string, Buffer>();
  for (const item of entries) {
    assert(!item.name.includes("\\") && !item.name.includes("\0") && !posix.isAbsolute(item.name));
    assert.equal(posix.normalize(item.name), item.name);
    assert(!item.name.split("/").includes(".."));
    assert(((item.externalAttributes >>> 16) & 0xf000) !== 0xa000, "proof symlinks refused");
    assert(
      !/(?:^consumer-|\.(?:node|tgz|vsix|tar\.gz)$)/u.test(item.name),
      "binary/consumer payloads refused",
    );
    const body = readZipEntry(archive, item);
    if (item.isDirectory) continue;
    assert(
      /\.(?:json|txt|jsonl|ts)$/u.test(item.name) ||
        /^sdk\/(?:original|tsrs-supplement)\/[\w./-]+\.d\.(?:cts|mts)$/u.test(item.name),
    );
    members.set(item.name, body);
  }
  const raw = members.get(entry.reference.captureMember);
  assert(raw);
  assert.equal(sha256(raw), entry.reference.captureMemberSha256);
  return members;
}
export async function authenticateCapturedBefore(
  request: Request,
  authority: ReleaseAuthority,
  read: GithubRead,
  sink: Sink = discard,
  transport: typeof fetch = fetch,
): Promise<CapturedBeforeAdmission> {
  const entry = bindCapturePolicy(
      readFileSync(new URL("./before-capture-policy.json.txt", import.meta.url)),
      request,
      authority,
    ),
    ref = entry.reference;
  const run = object(await read(`actions/runs/${ref.controllerRun}`));
  sink("captured-before-run", run);
  assert.equal(String(run.id), ref.controllerRun);
  assert.equal(object(run.repository).full_name, "ubugeeei-prod/vize");
  assert.equal(run.head_sha, ref.controllerHead);
  assert.equal(run.head_branch, "main");
  assert.equal(run.path, entry.artifact.knownWorkflowPath);
  assert.equal(run.event, "workflow_dispatch");
  assert.equal(run.run_attempt, entry.artifact.runAttempt);
  assert.equal(run.status, "completed");
  assert.equal(run.conclusion, "failure");
  const jobs = object(
    await read(`actions/runs/${ref.controllerRun}/jobs?per_page=100&filter=latest`),
  );
  sink("captured-before-jobs", jobs);
  const roster = array(jobs.jobs).map(object);
  assert.equal(jobs.total_count, roster.length);
  assert.equal(roster.length, 2);
  for (const job of roster) {
    assert.equal(job.run_id, run.id);
    assert.equal(job.head_sha, ref.controllerHead);
    assert.equal(job.run_attempt, entry.artifact.runAttempt);
    assert.equal(job.status, "completed");
  }
  const selected = roster.filter((j) => j.name === "public-consumer"),
    contracts = roster.filter((j) => j.name === "contracts");
  assert.equal(selected.length, 1);
  assert.equal(contracts.length, 1);
  assert.equal(selected[0]!.id, entry.artifact.jobId);
  assert.equal(selected[0]!.run_id, run.id);
  assert.equal(selected[0]!.head_sha, ref.controllerHead);
  assert.equal(selected[0]!.status, "completed");
  assert.equal(selected[0]!.conclusion, "failure");
  assert.equal(contracts[0]!.conclusion, "success");
  const manualPacket = object(wholeJson(await authenticatedBlob(ref.manualRequestBlob, read)));
  assert.equal(manualPacket.schema, "vize.humanReportPublic.manualRequest.v1");
  assert.equal(manualPacket.oracleBase64, null);
  const manual = object(manualPacket.request) as unknown as Request;
  assert.equal(manual.reviewedController.head, ref.controllerHead);
  assert.equal(manual.reviewedController.files.length, entry.artifact.sourceFileCount);
  assert.equal(
    new Set(manual.reviewedController.files.map((file) => file.path)).size,
    manual.reviewedController.files.length,
  );
  // The exact policy-pinned original manual Git blob owns the old closed71
  // inventory. Do not apply today's additional DATA seeds to historical e57.
  for (const file of manual.reviewedController.files) {
    sourcePath(file.path);
    const result = object(await read(`contents/${file.path}?ref=${ref.controllerHead}`));
    assert.equal(result.path, file.path);
    assert.equal(result.type, "file");
    assert.equal(result.encoding, "base64");
    const body = bytes(text(result.content).replace(/\n/gu, ""));
    assert.equal(body.length, file.bytes);
    assert.equal(sha256(body), file.sha256);
    assert.equal(gitBlob(body), file.gitBlob);
    assert.equal(result.sha, file.gitBlob);
  }
  const meta = object(await read(`actions/artifacts/${ref.artifactId}`));
  sink("captured-before-artifact", meta);
  assert.equal(meta.id, ref.artifactId);
  assert.equal(meta.name, `human-report-${request.role}-before-${ref.controllerRun}`);
  assert.equal(meta.digest, ref.archiveDigest);
  assert.equal(meta.expired, false);
  assert.equal(meta.size_in_bytes, entry.artifact.compressedBytes);
  const owner = object(meta.workflow_run);
  assert.equal(String(owner.id), ref.controllerRun);
  assert.equal(owner.head_sha, ref.controllerHead);
  assert.equal(owner.head_branch, "main");
  const headers: Record<string, string> = process.env.GITHUB_TOKEN
    ? { Authorization: `Bearer ${process.env.GITHUB_TOKEN}` }
    : {};
  const redirect = await transport(
    `https://api.github.com/repos/ubugeeei-prod/vize/actions/artifacts/${ref.artifactId}/zip`,
    { headers, redirect: "manual", signal: AbortSignal.timeout(30_000) },
  );
  assert([301, 302, 303, 307, 308].includes(redirect.status));
  const target = new URL(text(redirect.headers.get("location")));
  assert.equal(target.protocol, "https:");
  assert.equal(target.username, "");
  assert.equal(target.password, "");
  await redirect.body?.cancel();
  const response = await transport(target, {
    redirect: "error",
    signal: AbortSignal.timeout(60_000),
  });
  assert.equal(response.status, 200);
  const archive = await boundedBytes(response, 12 * 1024 ** 2);
  sink("captured-before-archive", {
    bytes: archive.length,
    sha256: sha256(archive),
    expected: ref.archiveDigest,
    bodyExcluded: true,
  });
  const members = decodeCapturedArchive(archive, entry),
    capture = wholeJson(members.get(ref.captureMember)!) as unknown as UnqualifiedCapture;
  sink("captured-before-raw", capture);
  const release = await authenticateRelease(manual.release, read, sink);
  validateBeforeCaptureRecord(capture, manual, request, release, entry, members);
  return {
    kind: "CAPTURE_ONLY",
    capture,
    beforeJudgment: requireCaptureJudgment(entry),
    accepted: false,
    productQualified: false,
    afterQualified: false,
    issueQualified: false,
    completeInstalledFileSetQualified: false,
    outcomeAssessed: false,
  };
}
