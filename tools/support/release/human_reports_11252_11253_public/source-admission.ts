import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import type { Oracle, Request } from "./model.ts";
import type { GithubRead } from "./authority.ts";
import { array, bytes, gitBlob, hash, object, sha256, text, wholeJson } from "./value.ts";
import { authenticateFixCustody } from "./fix-custody.ts";
export async function authenticatedBlob(blob: string, read: GithubRead): Promise<Buffer> {
  hash(blob, 40);
  const value = object(await read(`git/blobs/${blob}`));
  assert.equal(value.encoding, "base64");
  const raw = bytes(text(value.content).replace(/\n/gu, ""));
  assert.equal(gitBlob(raw), blob);
  return raw;
}
export async function authenticateSourceAdmission(
  request: Request,
  read: GithubRead,
): Promise<unknown> {
  const admission = request.sourceAdmission;
  assert(admission);
  const receiptRaw = await authenticatedBlob(admission.receiptBlob, read);
  assert.equal(sha256(receiptRaw), admission.receiptSha256);
  const receipt = object(wholeJson(receiptRaw));
  assert.equal(receipt.schema, "vize.humanReportPublic.sourceAdmission.v1");
  assert.equal(receipt.role, request.role);
  assert.equal(receipt.fixtureReceiptSha256, request.fixture.receiptSha256);
  assert.equal(receipt.commit, admission.commit);
  assert.equal(String(receipt.run), admission.run);
  assert.equal(receipt.scope, "source-adapted-not-public-target-execution");
  assert.equal(receipt.publicTargetExecuted, false);
  const fix = request.release.fixes.find((f) => f.issue === request.fixture.issue);
  assert(fix);
  assert.deepEqual(receipt.fix, fix);
  const run = object(await read(`actions/runs/${admission.run}`));
  assert.equal(run.head_sha, admission.commit);
  assert.equal(run.status, "completed");
  assert.equal(run.conclusion, "success");
  assert.equal(object(run.repository).full_name, "ubugeeei-prod/vize");
  assert.equal(run.path, receipt.workflowPath);
  assert.equal(run.event, receipt.event);
  assert(text(receipt.workflowPath).startsWith(".github/workflows/"));
  assert(
    ["pull_request", "merge_group", "workflow_dispatch", "push"].includes(text(receipt.event)),
  );
  const jobs = object(await read(`actions/runs/${admission.run}/jobs?per_page=100&filter=latest`));
  assert.equal(jobs.total_count, array(jobs.jobs).length);
  const names = array(receipt.jobs).map(text);
  assert(names.length > 0);
  assert.equal(new Set(names).size, names.length);
  for (const name of names) {
    const found = array(jobs.jobs)
      .map(object)
      .filter((j) => j.name === name);
    assert.equal(found.length, 1);
    assert.equal(found[0]?.conclusion, "success");
    assert.equal(found[0]?.head_sha, admission.commit);
  }
  const executed = text(receipt.executedCommit);
  hash(executed, 40);
  const actual = object(await read(`git/commits/${executed}`));
  if (executed !== admission.commit)
    assert(
      array(actual.parents)
        .map(object)
        .some((p) => p.sha === admission.commit),
      "declared execution must join actual run head",
    );
  await authenticateFixCustody(fix, executed, executed, read);
  assert(request.oracleOwnerReceipt);
  const ownerRaw = readFileSync(request.oracleOwnerReceipt.path);
  assert.equal(ownerRaw.length, request.oracleOwnerReceipt.bytes);
  assert.equal(sha256(ownerRaw), request.oracleOwnerReceipt.sha256);
  assert.equal(receipt.oracleReceiptSha256, request.oracleOwnerReceipt.sha256);
  const owner = object(wholeJson(ownerRaw));
  assert.equal(owner.role, request.role);
  assert.equal(owner.fixtureReceiptSha256, request.fixture.receiptSha256);
  assert.equal(owner.commit, fix.originalHead);
  for (const command of request.commands) {
    assert(command.oracle);
    await authenticateOracle(command.oracle, fix.originalHead, read);
  }
  return {
    receipt,
    receiptSha256: admission.receiptSha256,
    receiptBlob: admission.receiptBlob,
    run,
    jobs: names,
    publicTargetExecuted: false,
  };
}
async function authenticateOracle(oracle: Oracle, head: string, read: GithubRead): Promise<void> {
  const source = oracle.sourceOwnedBy;
  assert.equal(source.commit, head);
  hash(source.sha256);
  const file = object(await read(`contents/${source.path}?ref=${source.commit}`));
  assert.equal(file.path, source.path);
  assert.equal(file.type, "file");
  const raw = bytes(text(file.content).replace(/\n/gu, ""));
  assert.equal(gitBlob(raw), file.sha);
  assert.equal(sha256(raw), source.sha256);
}
