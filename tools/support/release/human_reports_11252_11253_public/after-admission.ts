import assert from "node:assert/strict";
import type { Request, ReleaseAuthority } from "./model.ts";
import type { GithubRead } from "./authority.ts";
import { authenticatedBlob } from "./source-admission.ts";
import { authenticateFixCustody } from "./fix-custody.ts";
import {
  inputPlanSha256,
  inputPlanGitBlob,
  instantiatedInputSha256,
  planAuthorities,
  validateAfterInputs,
} from "./after-plan.ts";
import { array, bytes, gitBlob, hash, object, sha256, text, wholeJson } from "./value.ts";

/** Input-only source admission never supplies or certifies an output oracle. */
export async function authenticateAfterInputs(
  request: Request,
  authority: ReleaseAuthority,
  read: GithubRead,
): Promise<unknown> {
  validateAfterInputs(request);
  const admission = request.sourceAdmission;
  assert(admission);
  hash(admission.commit, 40);
  hash(admission.receiptSha256);
  assert.match(admission.run, /^[1-9]\d*$/u);
  const raw = await authenticatedBlob(admission.receiptBlob, read);
  assert.equal(sha256(raw), admission.receiptSha256);
  const receipt = object(wholeJson(raw));
  assert.equal(receipt.schema, "vize.humanReportPublic.sourceInputAdmission.v1");
  assert.equal(receipt.mode, "UNQUALIFIED_AFTER");
  assert.equal(receipt.role, request.role);
  assert.equal(receipt.fixtureReceiptSha256, request.fixture.receiptSha256);
  assert.equal(receipt.commit, admission.commit);
  assert.equal(String(receipt.run), admission.run);
  assert.equal(receipt.scope, "source-adapted-not-public-target-execution");
  assert.equal(receipt.publicTargetExecuted, false);
  assert.equal(receipt.outcomeAssessed, false);
  for (const key of ["oracleReceiptSha256", "oracles", "outputs", "outcomeExpectation"])
    assert.equal(receipt[key], undefined, "input admission cannot smuggle output authority");
  assert.equal(receipt.inputPlanSha256, inputPlanSha256);
  assert.equal(receipt.inputPlanGitBlob, inputPlanGitBlob);
  assert.equal(
    receipt.instantiatedInputsSha256,
    instantiatedInputSha256(request.role, authority.version),
  );
  assert.deepEqual(receipt.release, {
    version: authority.version,
    releaseRun: authority.run,
    workflowHead: authority.workflowHead,
    source: authority.source,
    candidate: authority.candidate,
    capsuleBlob: request.release.capsuleBlob,
  });
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
  assert(names.length > 0 && new Set(names).size === names.length);
  for (const name of names) {
    const matches = array(jobs.jobs)
      .map(object)
      .filter((j) => j.name === name);
    assert.equal(matches.length, 1);
    assert.equal(matches[0]?.conclusion, "success");
    assert.equal(matches[0]?.head_sha, admission.commit);
  }
  const executed = hash(receipt.executedCommit, 40),
    actual = object(await read(`git/commits/${executed}`));
  if (executed !== admission.commit)
    assert(
      array(actual.parents)
        .map(object)
        .some((p) => p.sha === admission.commit),
    );
  await authenticateFixCustody(fix, executed, executed, read);
  for (const source of planAuthorities()) {
    const file = object(await read(`contents/${text(source.path)}?ref=${text(source.commit)}`));
    assert.equal(file.path, source.path);
    assert.equal(file.encoding, "base64");
    const body = bytes(text(file.content).replace(/\n/gu, ""));
    assert.equal(gitBlob(body), file.sha);
    assert.equal(file.sha, source.gitBlob);
    assert.equal(body.length, source.bytes);
    assert.equal(sha256(body), source.sha256);
  }
  return {
    receipt,
    receiptBlob: admission.receiptBlob,
    receiptSha256: admission.receiptSha256,
    run,
    jobs: names,
    publicTargetExecuted: false,
    outcomeAssessed: false,
  };
}
