import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { asArray, asRecord, asString } from "../json-value.ts";
import { SOURCE_ADMISSION } from "./public_type_admission/source-admission.ts";

export type Api = (endpoint: string, payload?: Record<string, unknown>) => Promise<unknown>;
const repository = "ubugeeei-prod/vize";
const required = SOURCE_ADMISSION.requiredSource;
const releaseQuery = `query($cursor:String) {
  repository(owner:"ubugeeei-prod",name:"vize") {
    releases(first:100,after:$cursor,orderBy:{field:CREATED_AT,direction:DESC}) {
      nodes { tagName isDraft publishedAt }
      pageInfo { hasNextPage endCursor }
    }
  }
}`;
function timestamp(value: unknown): number {
  const parsed = Date.parse(asString(value));
  assert.ok(Number.isFinite(parsed), "publication timestamp");
  return parsed;
}
function published(value: unknown, tag: string): Record<string, unknown> {
  const record = asRecord(value);
  assert.equal(record.tag_name, tag);
  assert.equal(record.draft, false);
  timestamp(record.published_at);
  return record;
}
function comparison(value: unknown, target: string): Record<string, unknown> {
  const record = asRecord(value);
  assert.equal(asRecord(record.base_commit).sha, required);
  assert.equal(
    record.html_url,
    "https://github.com/" + repository + "/compare/" + required + "..." + target,
  );
  assert.ok(["ahead", "identical", "behind", "diverged"].includes(asString(record.status)));
  return record;
}
function contains(record: Record<string, unknown>): boolean {
  return (
    (record.status === "ahead" || record.status === "identical") &&
    asRecord(record.merge_base_commit).sha === required &&
    record.behind_by === 0
  );
}
export async function buildEligibility(
  version: string,
  candidate: string,
  api: Api,
  phase = "after",
  source = candidate,
) {
  assert.equal(phase, "after", "type admission is an explicit after-only request");
  assert.equal(candidate, source, "candidate must match the immutable public tag source");
  assert.match(version, /^0\.[0-9]+\.0$/u);
  assert.match(candidate, /^[0-9a-f]{40}$/u);
  for (const excluded of SOURCE_ADMISSION.explicitlyExcludedVersions)
    assert.notEqual(version, excluded, "the required source is absent from this publication");
  assert.ok(
    Number(version.split(".")[1]) > 603,
    "qualification follows the immutable 0.603 baseline",
  );
  const tagName = "v" + version;
  const pull = asRecord(await api("/pulls/10950"));
  assert.equal(pull.number, 10950);
  assert.equal(pull.merge_commit_sha, required);
  const merged = timestamp(pull.merged_at);
  const release = published(await api("/releases/tags/" + tagName), tagName);
  const targetTime = timestamp(release.published_at);
  assert.ok(targetTime >= merged);
  const prior = new Map<string, Record<string, unknown>>();
  for (const excluded of SOURCE_ADMISSION.explicitlyExcludedVersions) {
    const name = "v" + excluded;
    const record = published(await api("/releases/tags/" + name), name);
    assert.ok(timestamp(record.published_at) < merged);
    prior.set(name, record);
  }
  const baseline = prior.get("v0.603.0");
  assert.ok(baseline);
  const baselineTime = timestamp(baseline.published_at);
  let cursor: string | null = null;
  let targetSeen = false;
  for (;;) {
    const result = asRecord(await api("/graphql", { query: releaseQuery, variables: { cursor } }));
    assert.deepEqual(result.errors ?? [], [], "release enumeration must complete");
    const page = asRecord(asRecord(asRecord(result.data).repository).releases);
    for (const item of asArray(page.nodes)) {
      const record = asRecord(item),
        name = asString(record.tagName);
      if (record.isDraft === true || record.publishedAt === null) continue;
      const at = timestamp(record.publishedAt);
      if (name === tagName) {
        assert.equal(at, targetTime);
        targetSeen = true;
      } else if (/^v0\.[0-9]+\.0$/u.test(name) && at >= baselineTime && at < targetTime) {
        if (!prior.has(name)) prior.set(name, published(await api("/releases/tags/" + name), name));
      }
    }
    const info = asRecord(page.pageInfo);
    assert.equal(typeof info.hasNextPage, "boolean");
    if (!info.hasNextPage) break;
    const next = asString(info.endCursor);
    assert.ok(next.length > 0 && next !== cursor, "release enumeration must advance");
    cursor = next;
  }
  assert.ok(targetSeen, "the published tag must be present in the complete release listing");
  const needed = new Set([tagName, ...prior.keys()]);
  const tags = new Map<string, Record<string, unknown>>();
  for (let page = 1; needed.size > 0; page++) {
    const entries = asArray(await api("/tags?per_page=100&page=" + page));
    for (const item of entries) {
      const record = asRecord(item),
        name = asString(record.name);
      if (!needed.has(name)) continue;
      assert.match(asString(asRecord(record.commit).sha), /^[0-9a-f]{40}$/u);
      tags.set(name, record);
      needed.delete(name);
    }
    if (entries.length < 100) break;
  }
  assert.equal(needed.size, 0, "every publication requires its actual repository tag record");
  const tag = tags.get(tagName);
  assert.ok(tag);
  assert.equal(asRecord(tag.commit).sha, candidate);
  const sourceComparison = comparison(
    await api("/compare/" + required + "..." + candidate),
    candidate,
  );
  assert.ok(contains(sourceComparison), "the release must contain the actual merged source");
  const priorPublicationChecks = [];
  for (const [name, publication] of prior) {
    const previousTag = tags.get(name);
    assert.ok(previousTag);
    const sha = asString(asRecord(previousTag.commit).sha);
    const compared = comparison(await api("/compare/" + required + "..." + sha), sha);
    assert.equal(
      contains(compared),
      false,
      name + " already contains the change; this is not the first publication",
    );
    priorPublicationChecks.push({ publication, tag: previousTag, sourceComparison: compared });
  }
  return {
    schema: "vize.template-ref.first-containing-release.v1",
    version,
    firstContainingTag: tagName,
    pullRequest: 10950,
    publicationConfirmed: true,
    sourceSha: candidate,
    requiredSource: required,
    sourceComparison,
    tag,
    firstContainingPublication: release,
    priorPublicationChecks,
    sourceHistory: { mergedAt: pull.merged_at, mergeCommit: required, baseline: "v0.603.0" },
  };
}
async function main(): Promise<void> {
  const [version, candidate, source, phase, output] = process.argv.slice(2);
  assert.ok(version && candidate && source && phase && output);
  const token = process.env.GH_TOKEN;
  assert.ok(token, "only the ancestry proof step needs GH_TOKEN");
  assert.equal(process.env.GITHUB_REPOSITORY, repository);
  const api: Api = async (endpoint, payload) => {
    const url =
      endpoint === "/graphql"
        ? "https://api.github.com/graphql"
        : "https://api.github.com/repos/" + repository + endpoint;
    const response = await fetch(url, {
      method: payload ? "POST" : "GET",
      headers: {
        Authorization: "Bearer " + token,
        Accept: "application/vnd.github+json",
        "Content-Type": "application/json",
      },
      ...(payload ? { body: JSON.stringify(payload) } : {}),
      signal: AbortSignal.timeout(30_000),
    });
    assert.equal(response.status, 200, endpoint);
    return (await response.json()) as unknown;
  };
  const eligibility = await buildEligibility(version, candidate, api, phase, source);
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, JSON.stringify(eligibility, null, 2) + "\n", { flag: "wx" });
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  void main().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
