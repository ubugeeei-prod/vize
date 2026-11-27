import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import {
  isPublished,
  publishDraft,
} from "../../../.github/scripts/workflows/release/publication.ts";
import { readAutomationPrograms } from "../support/automation-source.ts";

const configuration = {
  repository: "owner/repository",
  tag: "v1.2.3",
  token: "test-token",
  api: "https://api.github.test",
};
const release = { id: 42, tag_name: configuration.tag, draft: true };

function github(responses: Array<{ status?: number; body?: unknown }>) {
  const requests: Array<{ url: string; method: string; body: unknown }> = [];
  const fetcher: typeof fetch = async (input, options) => {
    requests.push({
      url: String(input),
      method: options?.method ?? "GET",
      body: options?.body ? JSON.parse(String(options.body)) : null,
    });
    const next = responses.shift();
    assert.ok(next, "unexpected GitHub request");
    return new Response(next.body == null ? null : JSON.stringify(next.body), {
      status: next.status ?? 200,
    });
  };
  return { fetcher, requests };
}

test("inspection distinguishes absent, draft and published releases without writing", async () => {
  for (const [response, expected] of [
    [{ status: 404 }, false],
    [{ body: release }, false],
    [{ body: { ...release, draft: false } }, true],
  ] as const) {
    const api = github([response]);
    assert.equal(await isPublished(configuration, api.fetcher), expected);
    assert.deepEqual(
      api.requests.map((request) => request.method),
      ["GET"],
    );
  }
});

test("API failures and malformed release state cannot authorize mutation", async () => {
  for (const response of [
    { status: 403 },
    { status: 500 },
    { body: { ...release, draft: undefined } },
    { body: { ...release, tag_name: "v9.9.9" } },
  ]) {
    const api = github([response]);
    await assert.rejects(isPublished(configuration, api.fetcher));
    assert.deepEqual(
      api.requests.map((request) => request.method),
      ["GET"],
    );
  }
});

test("publication only clears the draft state and applies the selected latest policy", async () => {
  for (const makeLatest of ["true", "false", "legacy"]) {
    const api = github([{ body: release }, { body: { ...release, draft: false } }]);
    await publishDraft(configuration, "42", makeLatest, api.fetcher);
    assert.deepEqual(
      api.requests.map((request) => [request.method, request.body]),
      [
        ["GET", null],
        ["PATCH", { draft: false, make_latest: makeLatest }],
      ],
    );
    assert.ok(api.requests.every((request) => request.url.endsWith("/releases/42")));
  }
});

test("a published release stays read-only on recovery or a repeated publish step", async () => {
  const api = github([{ body: { ...release, draft: false } }]);
  await publishDraft(configuration, "42", "true", api.fetcher);
  assert.deepEqual(
    api.requests.map((request) => request.method),
    ["GET"],
  );
});

test("publication rejects invalid IDs, unexpected tags and unsuccessful finalization", async () => {
  const unused = github([]);
  await assert.rejects(publishDraft(configuration, "", "true", unused.fetcher));
  await assert.rejects(publishDraft(configuration, "42", "invalid", unused.fetcher));
  assert.deepEqual(unused.requests, []);
  const wrongTag = github([{ body: { ...release, tag_name: "v9.9.9" } }]);
  await assert.rejects(publishDraft(configuration, "42", "true", wrongTag.fetcher));
  assert.equal(wrongTag.requests.length, 1);
  const failed = github([{ body: release }, { status: 500 }]);
  await assert.rejects(publishDraft(configuration, "42", "true", failed.fetcher));
});

test("stable and prerelease assets upload to drafts before publication; published reruns skip writes", () => {
  const workflow = parse(readAutomationPrograms(".github", "workflows", "release.yml"));
  const steps = workflow.jobs["github-release"].steps;
  const inspect = steps.findIndex((step: { id?: string }) => step.id === "release-state");
  const upload = steps.findIndex((step: { id?: string }) => step.id === "draft-release");
  const publish = steps.findIndex(
    (step: { name?: string }) => step.name === "Publish completed draft release",
  );
  assert.ok(inspect >= 0 && upload > inspect && publish > upload);
  assert.equal(steps[upload].with.draft, true);
  assert.equal(steps[upload].with.make_latest, "false");
  assert.equal(steps[upload].with.prerelease, "${{ needs.version.outputs.prerelease }}");
  assert.equal(steps[publish].env.RELEASE_ID, "${{ steps.draft-release.outputs.id }}");
  assert.equal(steps[publish].env.RELEASE_MAKE_LATEST, "${{ needs.version.outputs.make_latest }}");
  for (const step of [steps[upload], steps[publish]]) {
    assert.equal(step.if, "steps.release-state.outputs.published != 'true'");
  }
  // The staged driver survives a checkout of a historical npm-resume tag.
  assert.match(steps[inspect].run, /\$VIZE_CI_FILES\/publication\.ts.*inspect/u);
  assert.match(steps[publish].run, /\$VIZE_CI_FILES\/publication\.ts.*publish/u);
});
