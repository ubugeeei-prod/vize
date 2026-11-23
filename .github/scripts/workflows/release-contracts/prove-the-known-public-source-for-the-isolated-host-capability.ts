import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
const version = process.env.PUBLIC_VERSION;
const source = process.env.PUBLIC_SOURCE_SHA;
const required = "7c2f6f46172404a8f95335a79ee2b95094f6b7ee";
const repository = "ubugeeei-prod/vize";
assert.equal(process.env.GITHUB_REPOSITORY, repository);
assert.equal(process.env.GITHUB_REF, "refs/heads/test/public-type-admission-release");
assert.equal(process.env.CAPABILITY_PHASE, "type-admission-capability");
assert.equal(version, "0.603.0");
assert.equal(source, "3778a06863bc70bf63b435ae816cca134d2d438e");
assert.equal(process.env.PUBLIC_CANDIDATE, source);
for (const name of ["CAPABILITY_ADMISSION", "CAPABILITY_OBSERVERS", "CAPABILITY_CPP"])
  assert.notEqual(
    process.env[name],
    "true",
    "source-only proof must not select public qualification",
  );
assert.ok(process.env.GH_TOKEN);
const api = async <T>(endpoint: string): Promise<T> => {
  const response = await fetch("https://api.github.com/repos/" + repository + endpoint, {
    headers: {
      Authorization: "Bearer " + process.env.GH_TOKEN,
      Accept: "application/vnd.github+json",
    },
    signal: AbortSignal.timeout(30000),
  });
  assert.equal(response.status, 200, endpoint);
  return (await response.json()) as T;
};
const publishedRelease = await api<{ tag_name: string; draft: boolean; published_at: string }>(
  "/releases/tags/v" + version,
);
const tags = await api<{ name: string; commit: { sha: string } }[]>("/tags?per_page=100");
const tag = tags.find((value: { name: string }) => value.name === "v" + version);
assert.ok(tag);
assert.equal(tag.commit.sha, source);
assert.equal(publishedRelease.tag_name, tag.name);
assert.equal(publishedRelease.draft, false);
assert.ok(publishedRelease.published_at);
const comparison = await api<{
  base_commit: { sha: string };
  merge_base_commit: { sha: string };
  behind_by: number;
  status: string;
}>("/compare/" + required + "..." + source);
assert.equal(comparison.base_commit.sha, required);
assert.equal(comparison.merge_base_commit.sha, required);
assert.equal(comparison.behind_by, 0);
assert.ok(["ahead", "identical"].includes(comparison.status));
const temporary = requiredEnv("RUNNER_TEMP");
const write = (name: string, value: unknown) =>
  fs.writeFileSync(path.join(temporary, name), JSON.stringify(value, null, 2) + "\n", {
    flag: "wx",
  });
write("cpp-source-comparison.json", comparison);
write("cpp-release-identity.json", {
  version,
  sourceSha: source,
  requiredSource: required,
  containsRequiredSource: true,
  comparisonStatus: comparison.status,
  workflowSha: process.env.GITHUB_SHA,
  runId: process.env.GITHUB_RUN_ID,
});
write("public-type-admission-capability-plan.json", {
  schema: "vize.typeAdmission.sourceCapabilityEligibility.v1",
  scope: "source-capability-only",
  coreReleaseEligible: false,
  version,
  sourceSha: source,
  tag,
  publishedRelease,
  comparison,
  workflowSha: process.env.GITHUB_SHA,
  runId: process.env.GITHUB_RUN_ID,
  mapperHost: {
    route: "hosted-content-mapper",
    sdkVersion: "7.1.0-beta",
    workspace: path.join(temporary, "public-type-admission-capability-host"),
    setupReceipt: path.join(
      temporary,
      "public-type-admission-capability-host-proof/setup.json.txt",
    ),
  },
});
