import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readZipEntry, zipEntries } from "../../archive/zip.ts";
import { boundedBytes } from "../release/quota/http.ts";
import { authenticateRelease, type GithubRead } from "./authority.ts";
import { validateBeforeRecord } from "./before-record.ts";
import { authenticateController } from "./controller-authority.ts";
import { authenticatedBlob } from "./source-admission.ts";
import { discard, type Sink } from "./evidence.ts";
import type { Bundle, Request } from "./model.ts";
import { hash, object, sha256, text, wholeJson } from "./value.ts";
export async function authenticatePublicBefore(
  request: Request,
  github: GithubRead,
  sink: Sink = discard,
): Promise<unknown> {
  const expected = request.publicBefore;
  assert(expected);
  hash(expected.controllerHead, 40);
  hash(expected.bundleMemberSha256);
  assert.match(expected.controllerRun, /^[1-9]\d*$/u);
  assert(Number.isSafeInteger(expected.artifactId) && expected.artifactId > 0);
  assert.match(expected.archiveDigest, /^sha256:[a-f0-9]{64}$/u);
  const run = object(await github(`actions/runs/${expected.controllerRun}`));
  assert.equal(run.head_sha, expected.controllerHead);
  assert.equal(run.status, "completed");
  assert.equal(run.conclusion, "success");
  assert.equal(run.path, ".github/workflows/published-human-reports-11252-11253.yml");
  assert.equal(run.event, "workflow_dispatch");
  const meta = object(await github(`actions/artifacts/${expected.artifactId}`));
  assert.equal(meta.digest, expected.archiveDigest);
  assert.equal(meta.expired, false);
  const owner = object(meta.workflow_run);
  assert.equal(String(owner.id), expected.controllerRun);
  assert.equal(owner.head_sha, expected.controllerHead);
  assert(
    Number.isSafeInteger(meta.size_in_bytes) &&
      Number(meta.size_in_bytes) > 0 &&
      Number(meta.size_in_bytes) <= 8 * 1024 * 1024,
  );
  const headers: Record<string, string> = process.env.GITHUB_TOKEN
    ? { Authorization: `Bearer ${process.env.GITHUB_TOKEN}` }
    : {};
  const redirect = await fetch(
    `https://api.github.com/repos/ubugeeei-prod/vize/actions/artifacts/${expected.artifactId}/zip`,
    { headers, redirect: "manual", signal: AbortSignal.timeout(30_000) },
  );
  assert([301, 302, 303, 307, 308].includes(redirect.status));
  const target = new URL(text(redirect.headers.get("location")));
  assert.equal(target.protocol, "https:");
  assert.equal(target.username, "");
  assert.equal(target.password, "");
  await redirect.body?.cancel();
  const response = await fetch(target, { redirect: "error", signal: AbortSignal.timeout(60_000) });
  assert.equal(response.status, 200);
  const archive = await boundedBytes(response, 8 * 1024 * 1024);
  assert.equal(archive.length, meta.size_in_bytes);
  assert.equal("sha256:" + createHash("sha256").update(archive).digest("hex"), meta.digest);
  const entries = zipEntries(archive),
    matches = entries.filter((entry) => entry.name === "bundle.json");
  assert.equal(matches.length, 1);
  assert(matches[0] && matches[0].size <= 8 * 1024 * 1024);
  const raw = readZipEntry(archive, matches[0]);
  assert.equal(sha256(raw), expected.bundleMemberSha256);
  const bundle = wholeJson(raw) as Bundle;
  sink("archived-before-bundle", bundle);
  assert.equal(bundle.controller.head, expected.controllerHead);
  assert.equal(bundle.controller.run, expected.controllerRun);
  const controller = await authenticateController(
    bundle.controller,
    expected.reviewedController,
    github,
    true,
  );
  const release = await authenticateRelease(bundle.request.release, github, sink);
  const judgment = object(wholeJson(await authenticatedBlob(expected.judgmentBlob, github)));
  assert.equal(judgment.schema, "vize.humanReportPublic.beforeJudgment.v1");
  assert.equal(judgment.role, request.role);
  assert.equal(judgment.fixtureReceiptSha256, request.fixture.receiptSha256);
  assert.equal(judgment.archiveDigest, expected.archiveDigest);
  assert.equal(judgment.bundleMemberSha256, expected.bundleMemberSha256);
  assert.equal(judgment.sourceOwnedTargetOutputsClaimed, false);
  assert.deepEqual(judgment.oracles, expected.oracles);
  validateBeforeRecord(bundle, request, release);
  const verdicts = entries.filter((e) => e.name === "verdict.json");
  assert.equal(verdicts.length, 1);
  const verdict = object(wholeJson(readZipEntry(archive, verdicts[0]!)));
  assert.equal(verdict.status, "OBSERVED_PUBLIC_BEFORE_ONLY");
  for (const key of [
    "accepted",
    "productQualified",
    "afterQualified",
    "issueQualified",
    "knownBugReproductionJudged",
    "sourceBeforeTransferred",
  ])
    assert.equal(verdict[key], false);
  for (const entry of entries)
    assert(
      !/(?:^consumer-|\.(?:node|tgz|vsix|tar\.gz)$)/u.test(entry.name),
      "binary/public consumer payloads must not be in the proof artifact",
    );
  for (const install of bundle.installs)
    for (const sdk of release.sdk.filter((v) => v.path !== "package.json")) {
      const members = entries.filter(
        (e) => e.name === `sdk/${install.kind}/${sdk.path.replaceAll("/", "--")}`,
      );
      assert.equal(members.length, 1);
      const body = readZipEntry(archive, members[0]!);
      assert.equal(body.length, sdk.bytes);
      assert.equal(sha256(body), sdk.sha256);
    }
  return {
    controllerRun: expected.controllerRun,
    controllerHead: expected.controllerHead,
    artifactId: expected.artifactId,
    archiveDigest: expected.archiveDigest,
    bundleMemberSha256: expected.bundleMemberSha256,
    observations: bundle.observations,
    controller,
    release,
    judgmentScope: "public-recorded-before-packets-not-source-owned-target-output",
    scope:
      "Original 0.627.0 public-before vector only; no after acceptance or source-before substitution",
  };
}
