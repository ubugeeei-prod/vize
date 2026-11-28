// Runs from the staged workflow revision, including when an old tag is resumed.
import assert from "node:assert/strict";
import { appendFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type Configuration = { repository: string; tag: string; token: string; api: string };
type Release = { id: number; tag_name: string; draft: boolean };
type Request = typeof fetch;

async function request(
  configuration: Configuration,
  endpoint: string,
  fetcher: Request,
  body?: object,
): Promise<Response> {
  return fetcher(`${configuration.api}/repos/${configuration.repository}/${endpoint}`, {
    method: body == null ? "GET" : "PATCH",
    headers: {
      accept: "application/vnd.github+json",
      authorization: `Bearer ${configuration.token}`,
      "content-type": "application/json",
      "x-github-api-version": "2022-11-28",
    },
    body: body == null ? undefined : JSON.stringify(body),
    signal: AbortSignal.timeout(30_000),
  });
}

async function releaseResponse(response: Response, tag: string): Promise<Release> {
  assert.ok(response.ok, `GitHub release request failed: HTTP ${response.status}`);
  const release = (await response.json()) as Release;
  assert.ok(Number.isSafeInteger(release.id) && release.id > 0, "invalid release ID");
  assert.equal(release.tag_name, tag, "release tag does not match the selected candidate");
  assert.equal(typeof release.draft, "boolean", "release draft state is required");
  return release;
}

/** Published releases are read-only, including npm recovery and job reruns. */
export async function isPublished(
  configuration: Configuration,
  fetcher: Request = fetch,
): Promise<boolean> {
  const response = await request(
    configuration,
    `releases/tags/${encodeURIComponent(configuration.tag)}`,
    fetcher,
  );
  if (response.status === 404) return false;
  return !(await releaseResponse(response, configuration.tag)).draft;
}

/** Called only after the draft action successfully uploads every selected asset. */
export async function publishDraft(
  configuration: Configuration,
  id: string,
  makeLatest: string,
  fetcher: Request = fetch,
): Promise<void> {
  assert.match(id, /^[1-9][0-9]*$/u, "draft release ID is required");
  assert.ok(["true", "false", "legacy"].includes(makeLatest), "invalid latest-release policy");
  const endpoint = `releases/${id}`;
  const release = await releaseResponse(
    await request(configuration, endpoint, fetcher),
    configuration.tag,
  );
  assert.equal(String(release.id), id, "release identity changed");
  if (!release.draft) return;
  const published = await releaseResponse(
    await request(configuration, endpoint, fetcher, { draft: false, make_latest: makeLatest }),
    configuration.tag,
  );
  assert.equal(published.id, release.id, "release identity changed during publication");
  assert.equal(published.draft, false, "release remains a draft");
}

async function main(): Promise<void> {
  const { GITHUB_REPOSITORY, RELEASE_TAG, GH_TOKEN } = process.env;
  assert.ok(
    GITHUB_REPOSITORY && RELEASE_TAG && GH_TOKEN,
    "release repository, tag and token are required",
  );
  const configuration = {
    repository: GITHUB_REPOSITORY,
    tag: RELEASE_TAG,
    token: GH_TOKEN,
    api: process.env.GITHUB_API_URL ?? "https://api.github.com",
  };
  if (process.argv[2] === "inspect") {
    assert.ok(process.env.GITHUB_OUTPUT, "GitHub step outputs are required");
    const published = await isPublished(configuration);
    appendFileSync(process.env.GITHUB_OUTPUT, `published=${published}\n`);
    console.log(
      published
        ? `${RELEASE_TAG} is already published; keep it unchanged`
        : `${RELEASE_TAG} can upload to a draft`,
    );
    return;
  }
  assert.equal(process.argv[2], "publish", "expected inspect or publish");
  await publishDraft(
    configuration,
    process.env.RELEASE_ID ?? "",
    process.env.RELEASE_MAKE_LATEST ?? "",
  );
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  await main();
}
