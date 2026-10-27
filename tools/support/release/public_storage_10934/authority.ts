import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  asArray, asNumber, asRecord, asString, blob, github, parse, save, sha256,
  REQUIRED_MERGE, REPO,
} from "./io.ts";

export interface ProviderReceipt {
  target: string;
  receipt: Record<string, unknown>;
  image: { bytes: number; sha256: string };
  buildToken: string;
}
export interface Authority {
  version: string;
  candidate: string;
  source: string;
  workflowHead: string;
  producerRun: number;
  providers: ProviderReceipt[];
  npmSRI: Record<string, unknown>;
  authorityBlob: string;
  authoritySha256: string;
}
function integer(value: unknown): number {
  const number = asNumber(value);
  assert.ok(Number.isSafeInteger(number) && number > 0);
  return number;
}
function commit(value: unknown): string {
  const sha = asString(value);
  assert.match(sha, /^[a-f0-9]{40}$/);
  return sha;
}
export async function loadAuthority(
  version: string, candidate: string, authorityBlob: string, evidence: string,
): Promise<Authority> {
  const raw = await blob(authorityBlob);
  fs.writeFileSync(path.join(evidence, "authority.raw.json"), raw);
  const authority = asRecord(parse(raw));
  assert.equal(authority.schema, "vize.publicStorageReleaseAuthority.v1");
  assert.equal(authority.version, version);
  assert.equal(authority.tagSourceSha, candidate);
  assert.equal(authority.required10940MergeSha, REQUIRED_MERGE);
  const source = commit(authority.producerSource);
  const producer = asRecord(authority.producerEvidence);
  const producerRun = integer(producer.runId);
  const workflowHead = commit(producer.workflowHead);
  assert.equal(producer.origin, "original-actions-release-producer-artifacts; rawreceiptbytesunchanged");
  const candidateApi = asRecord(await github("git/commits/" + candidate));
  const parents = asArray(candidateApi.parents);
  assert.equal(parents.length, 1);
  assert.equal(asRecord(parents[0]).sha, source);
  const ancestry = asRecord(await github("compare/" + REQUIRED_MERGE + "..." + source + "?per_page=1"));
  assert.equal(asRecord(ancestry.merge_base_commit).sha, REQUIRED_MERGE);
  assert.equal(asRecord(ancestry.base_commit).sha, REQUIRED_MERGE);
  assert.ok(asString(asRecord(asRecord(ancestry.base_commit).commit).message)
    .includes("Co-authored-by: Masaki Kawaguchi <m-kawaguchi@binc.jp>"));
  assert.ok(ancestry.status === "ahead" || ancestry.status === "identical");
  let tag = asRecord(asRecord(await github("git/ref/tags/v" + version)).object);
  if (tag.type === "tag") {
    tag = asRecord(asRecord(await github("git/tags/" + commit(tag.sha))).object);
  }
  assert.equal(tag.type, "commit");
  assert.equal(tag.sha, candidate);
  const runApi = asRecord(await github("actions/runs/" + producerRun));
  assert.equal(runApi.head_sha, workflowHead);
  assert.equal(runApi.head_branch, "main");
  assert.equal(runApi.path, ".github/workflows/release.yml");
  assert.equal(asRecord(runApi.repository).full_name, REPO);
  save(path.join(evidence, "release-identity.json"), {
    version, candidate, source, workflowHead, producerRun, candidateApi,
    ancestry: { status: ancestry.status, mergeBase: asRecord(ancestry.merge_base_commit).sha },
    producerApi: runApi,
  });
  const componentBlob = commit(producer.componentBlob);
  const componentRaw = await blob(componentBlob);
  fs.writeFileSync(path.join(evidence, "producer-component.raw.json"), componentRaw);
  const component = asRecord(parse(componentRaw));
  assert.equal(component.schema, "vize.originalActionsReleaseProducerReceipts.v1");
  assert.equal(component.version, version);
  assert.equal(component.producerRun, producerRun);
  assert.equal(component.producerSource, source);
  assert.equal(component.workflowHead, workflowHead);
  assert.equal(component.candidate, candidate);
  const originals = asArray(component.receipts).map((value) => asRecord(value));
  assert.equal(originals.length, 8);
  const receipts = asArray(authority.receipts).map((value) => asRecord(value));
  assert.equal(receipts.length, 8);
  const targets = new Set<string>();
  const providers: ProviderReceipt[] = [];
  for (const record of receipts) {
    const bytes = Buffer.from(asString(record.rawBase64), "base64");
    const digest = sha256(bytes);
    assert.equal(record.sha256, digest);
    const original = originals.filter((item) => item.sha256 === digest);
    assert.equal(original.length, 1);
    const originalItem = original[0];
    assert.ok(originalItem);
    assert.equal(originalItem.rawBase64, record.rawBase64);
    assert.deepEqual(originalItem.origin, record.origin);
    const receipt = asRecord(parse(bytes));
    assert.equal(receipt.schema, "vize.platformVsix.v1");
    assert.equal(receipt.version, version);
    assert.equal(receipt.source, candidate);
    const target = asString(receipt.target);
    assert.match(target, /^[a-z0-9-]+$/);
    assert.equal(targets.has(target), false);
    targets.add(target);
    const provider = asRecord(receipt.provider);
    assert.equal(provider.schema, "vize.vscodeNativeProvider.v1");
    assert.equal(provider.source, candidate);
    assert.equal(provider.releaseVersion, version);
    assert.equal(provider.targetPlatform, target);
    const images = asArray(provider.files).map((value) => asRecord(value))
      .filter((file) => asString(file.path).endsWith(".node"));
    assert.equal(images.length, 1);
    const image = images[0];
    assert.ok(image);
    assert.equal(image.path, provider.nodeFile);
    const origin = asRecord(record.origin);
    assert.equal(origin.kind, "original-actions-producer-artifact");
    assert.equal(origin.runId, producerRun);
    assert.equal(origin.workflowHead, workflowHead);
    assert.equal(origin.member, "vize-" + target + ".vsix.receipt.json");
    const artifactId = integer(origin.artifactId);
    assert.equal(origin.artifactApi, "https://api.github.com/repos/" + REPO + "/actions/artifacts/" + artifactId);
    const artifact = asRecord(await github("actions/artifacts/" + artifactId));
    assert.equal(artifact.expired, false);
    const producerIdentity = asRecord(artifact.workflow_run);
    assert.equal(producerIdentity.id, producerRun);
    assert.equal(producerIdentity.head_sha, workflowHead);
    save(path.join(evidence, "artifact-" + artifactId + ".json"), artifact);
    fs.writeFileSync(path.join(evidence, "provider-" + target + ".raw.json"), bytes);
    const bytesCount = integer(image.bytes);
    const imageSha = asString(image.sha256);
    assert.match(imageSha, /^[a-f0-9]{64}$/);
    providers.push({
      target, receipt, image: { bytes: bytesCount, sha256: imageSha },
      buildToken: asString(provider.nativeBuildToken),
    });
  }
  assert.equal(targets.size, 8);
  assert.equal(providers.filter((item) => item.target === "linux-x64").length, 1);
  save(path.join(evidence, "authority-validation.json"), {
    authorityBlob, authoritySha256: sha256(raw), componentBlob,
    componentSha256: sha256(componentRaw), version, candidate, source,
    workflowHead, producerRun, requiredMerge: REQUIRED_MERGE,
    targets: [...targets], rawReceiptsUnchanged: true,
  });
  return {
    version, candidate, source, workflowHead, producerRun, providers,
    npmSRI: asRecord(authority.npmSRI), authorityBlob, authoritySha256: sha256(raw),
  };
}
