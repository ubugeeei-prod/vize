#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { githubApiPages } from "./release-preflight-github.mjs";

/**
 * Conventional Commits breaking markers, matched line by line.
 *
 * Deliberately equivalent to the `grep -E` pattern this script replaced
 * (`^[[:alnum:]_-]+(\([^)]+\))?!:|^BREAKING CHANGE:`) so moving the decision
 * out of workflow shell does not silently widen or narrow "breaking".
 */
const BREAKING_MARKER_PATTERN = /^(?:[A-Za-z0-9_-]+(?:\([^)]+\))?!:|BREAKING CHANGE:)/m;

const COMMIT_SHA_PATTERN = /^[0-9a-f]{40}$/;

/**
 * @param {unknown} value
 * @returns {string}
 */
export function normalizeMarkerText(value) {
  return typeof value === "string" ? value.replace(/\r\n?/g, "\n") : "";
}

/**
 * Marker text of a pull request: title then body, the same assembly the
 * `pull_request` event path has always used.
 *
 * @param {{ body?: unknown; title?: unknown }} pullRequest
 * @returns {string}
 */
export function pullRequestMarkerText(pullRequest) {
  return `${normalizeMarkerText(pullRequest.title)}\n${normalizeMarkerText(pullRequest.body)}\n`;
}

/**
 * @param {unknown} marker
 * @returns {"major" | "none"}
 */
export function releaseTypeForMarker(marker) {
  return BREAKING_MARKER_PATTERN.test(normalizeMarkerText(marker)) ? "major" : "none";
}

/**
 * Pick the pull request a pushed commit is the merge result of.
 *
 * `GET /commits/{sha}/pulls` also lists pull requests that merely *contain* the
 * commit, so only an exact `merge_commit_sha` match proves this push is that
 * pull request's squash (or merge, or rebase) commit.
 *
 * @param {{ pullRequests?: readonly unknown[]; sha: string }} lookup
 * @returns {{ body?: unknown; number?: unknown; title?: unknown } | null}
 */
export function selectSquashedPullRequest({ pullRequests, sha }) {
  const associated = pullRequests ?? [];
  for (const candidate of associated) {
    const pullRequest = /** @type {Record<string, unknown>} */ (candidate);
    if (
      pullRequest != null &&
      pullRequest.merged_at != null &&
      pullRequest.merge_commit_sha === sha
    ) {
      return pullRequest;
    }
  }
  return null;
}

/**
 * @param {{
 *   apiUrl: string;
 *   fetchImpl?: typeof globalThis.fetch;
 *   repository: string;
 *   sha: string;
 *   token: string;
 * }} request
 * @returns {Promise<readonly unknown[]>}
 */
export async function listAssociatedPullRequests({ apiUrl, repository, sha, token, ...options }) {
  if (!COMMIT_SHA_PATTERN.test(sha)) {
    throw new Error(`SemVer marker lookup needs a full commit SHA, got ${sha || "(empty)"}`);
  }
  if (repository === "") throw new Error("GITHUB_REPOSITORY is required to resolve the merged PR");
  if (token === "") throw new Error("GITHUB_TOKEN is required to resolve the merged PR");
  return githubApiPages({
    apiUrl,
    repository,
    token,
    resource: `commits/${sha}/pulls`,
    ...options,
  });
}

/**
 * Resolve the text a SemVer release-type decision is made from.
 *
 * A squash merge rewrites the pull-request body away, so the push event only
 * sees `<title> (#<number>)`. The merged pull request itself is the durable
 * home for the marker, and the commit → pull-request association recovers it.
 *
 * @param {{
 *   commitMessage?: unknown;
 *   eventName: string;
 *   listPullRequestsForCommit?: (sha: string) => Promise<readonly unknown[]>;
 *   pullRequestBody?: unknown;
 *   pullRequestTitle?: unknown;
 *   sha: string;
 * }} event
 * @returns {Promise<{
 *   marker: string;
 *   releaseType: "major" | "none";
 *   source: "commit_message" | "pull_request_event" | "squashed_pull_request";
 * }>}
 */
export async function resolveSemverChangeMarker({
  commitMessage,
  eventName,
  listPullRequestsForCommit,
  pullRequestBody,
  pullRequestTitle,
  sha,
}) {
  if (eventName === "pull_request") {
    const marker = pullRequestMarkerText({ body: pullRequestBody, title: pullRequestTitle });
    return { marker, releaseType: releaseTypeForMarker(marker), source: "pull_request_event" };
  }
  const pullRequests =
    listPullRequestsForCommit == null ? [] : await listPullRequestsForCommit(sha);
  const squashed = selectSquashedPullRequest({ pullRequests, sha });
  if (squashed != null) {
    const marker = pullRequestMarkerText(squashed);
    return { marker, releaseType: releaseTypeForMarker(marker), source: "squashed_pull_request" };
  }
  const marker = normalizeMarkerText(commitMessage);
  return { marker, releaseType: releaseTypeForMarker(marker), source: "commit_message" };
}

/**
 * The pull request a `pull_request` run is checking, straight from the event
 * payload the runner wrote — no shell templating of untrusted title/body text.
 *
 * @param {string} eventPath
 * @returns {{ body?: unknown; title?: unknown }}
 */
export function readEventPullRequest(eventPath) {
  if (eventPath === "") {
    throw new Error("GITHUB_EVENT_PATH is required to read the pull-request SemVer marker");
  }
  const event = /** @type {{ pull_request?: { body?: unknown; title?: unknown } }} */ (
    JSON.parse(fs.readFileSync(eventPath, "utf8"))
  );
  return event.pull_request ?? {};
}

function readHeadCommitMessage() {
  const result = spawnSync("git", ["log", "-1", "--format=%B"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    timeout: 30_000,
  });
  if (result.error != null) throw result.error;
  if (result.status !== 0) {
    const detail = [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
    throw new Error(
      [`git log -1 --format=%B failed with exit ${result.status}`, detail]
        .filter(Boolean)
        .join("\n"),
    );
  }
  return result.stdout;
}

/** @param {Record<string, string>} outputs */
function writeOutputs(outputs) {
  const outputPath = process.env.GITHUB_OUTPUT;
  if (!outputPath) throw new Error("GITHUB_OUTPUT is required for the SemVer change marker");
  fs.appendFileSync(
    outputPath,
    Object.entries(outputs)
      .map(([name, value]) => `${name}=${value}\n`)
      .join(""),
  );
}

/**
 * @param {NodeJS.ProcessEnv} env
 * @returns {Promise<{ marker: string; releaseType: string; source: string }>}
 */
export async function runSemverChangeMarker(env = process.env) {
  const eventName = env.GITHUB_EVENT_NAME ?? "";
  const isPullRequest = eventName === "pull_request";
  const pullRequest = isPullRequest ? readEventPullRequest(env.GITHUB_EVENT_PATH ?? "") : {};
  const resolution = await resolveSemverChangeMarker({
    commitMessage: isPullRequest ? "" : readHeadCommitMessage(),
    eventName,
    listPullRequestsForCommit: isPullRequest
      ? undefined
      : (sha) =>
          listAssociatedPullRequests({
            apiUrl: env.GITHUB_API_URL ?? "https://api.github.com",
            repository: env.GITHUB_REPOSITORY ?? "",
            sha,
            token: env.GITHUB_TOKEN ?? "",
          }),
    pullRequestBody: pullRequest.body,
    pullRequestTitle: pullRequest.title,
    sha: env.GITHUB_SHA ?? "",
  });
  writeOutputs({ "marker-source": resolution.source, "release-type": resolution.releaseType });
  console.log(
    `SemVer release type ${resolution.releaseType} resolved from ${resolution.source} on ${eventName || "(unknown event)"}.`,
  );
  return resolution;
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  runSemverChangeMarker().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  });
}
