import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

function pullRequestMarker(pullRequest) {
  if (typeof pullRequest?.title !== "string") {
    throw new Error("Pull request title is missing from the GitHub event");
  }
  return `${pullRequest.title}\n${typeof pullRequest.body === "string" ? pullRequest.body : ""}`;
}

function pushCommitMarker(event) {
  const messages = Array.isArray(event.commits)
    ? event.commits
        .map((commit) => commit?.message)
        .filter((message) => typeof message === "string")
    : [];
  const headMessage = event.head_commit?.message;
  if (typeof headMessage === "string" && !messages.includes(headMessage)) {
    messages.push(headMessage);
  }
  if (messages.length === 0) {
    throw new Error("Push event contains no commit message for the SemVer fallback");
  }
  return messages.join("\n");
}

async function mergedPullRequestForCommit({ apiUrl, fetchImpl, repository, sha, token }) {
  if (!repository || !sha || !token) {
    throw new Error("GITHUB_REPOSITORY, GITHUB_SHA, and GITHUB_TOKEN are required for push events");
  }
  const encodedRepository = repository.split("/").map(encodeURIComponent).join("/");
  const response = await fetchImpl(
    `${apiUrl}/repos/${encodedRepository}/commits/${encodeURIComponent(sha)}/pulls`,
    {
      headers: {
        Accept: "application/vnd.github+json",
        Authorization: `Bearer ${token}`,
        "X-GitHub-Api-Version": "2022-11-28",
      },
    },
  );
  if (!response.ok) {
    throw new Error(`GitHub associated-pulls request failed with HTTP ${response.status}`);
  }
  const associated = await response.json();
  if (!Array.isArray(associated)) {
    throw new Error("GitHub associated-pulls response is not an array");
  }
  const exactMerged = associated.filter(
    (pullRequest) => pullRequest?.merge_commit_sha === sha && pullRequest.merged_at,
  );
  if (exactMerged.length > 1) {
    throw new Error(`Commit ${sha} has multiple exact merged pull requests`);
  }
  return exactMerged[0] ?? null;
}

export async function resolveSemverChangeMarker({
  apiUrl = "https://api.github.com",
  event,
  eventName,
  fetchImpl = globalThis.fetch,
  repository,
  sha,
  token,
}) {
  if (eventName === "pull_request") {
    return pullRequestMarker(event.pull_request);
  }
  if (eventName !== "push") {
    throw new Error(`Unsupported SemVer event: ${eventName}`);
  }

  const pullRequest = await mergedPullRequestForCommit({
    apiUrl,
    fetchImpl,
    repository: repository || event.repository?.full_name,
    sha: sha || event.after,
    token,
  });
  return pullRequest ? pullRequestMarker(pullRequest) : pushCommitMarker(event);
}

async function main() {
  const [outputPath] = process.argv.slice(2);
  if (!outputPath || !process.env.GITHUB_EVENT_PATH) {
    throw new Error(
      "Usage: node tools/github/semver-change-marker.mjs <output-path> with GITHUB_EVENT_PATH",
    );
  }
  const event = JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH, "utf8"));
  const marker = await resolveSemverChangeMarker({
    apiUrl: process.env.GITHUB_API_URL,
    event,
    eventName: process.env.GITHUB_EVENT_NAME,
    repository: process.env.GITHUB_REPOSITORY,
    sha: process.env.GITHUB_SHA,
    token: process.env.GITHUB_TOKEN,
  });
  fs.writeFileSync(outputPath, `${marker}\n`);
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  await main();
}
