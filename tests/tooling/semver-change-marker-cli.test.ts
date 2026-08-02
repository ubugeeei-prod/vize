import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { once } from "node:events";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  listAssociatedPullRequests,
  readEventPullRequest,
} from "../../tools/github/semver-change-marker.mjs";
import { root } from "./support/github-workflows.ts";
import {
  breakingPullRequest,
  previousSha,
  squashCommitMessage,
  squashSha,
} from "./support/semver-change-marker.ts";

test("associated pull requests come from the commit's pull-request association", async () => {
  const requestedUrls: string[] = [];
  const pullRequests = await listAssociatedPullRequests({
    apiUrl: "https://api.github.test",
    fetchImpl: async (url: URL) => {
      requestedUrls.push(String(url));
      return new Response(JSON.stringify([breakingPullRequest]), { status: 200 });
    },
    repository: "owner/repository",
    sha: squashSha,
    token: "secret",
  });

  assert.deepEqual(requestedUrls, [
    `https://api.github.test/repos/owner/repository/commits/${squashSha}/pulls?per_page=100&page=1`,
  ]);
  assert.deepEqual(pullRequests, [breakingPullRequest]);

  await assert.rejects(
    listAssociatedPullRequests({
      apiUrl: "https://api.github.test",
      repository: "owner/repository",
      sha: "main",
      token: "secret",
    }),
    new Error("SemVer marker lookup needs a full commit SHA, got main"),
  );
});

test("the pull-request marker is read from the runner's event payload", () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-semver-marker-"));
  const eventPath = path.join(workspace, "event.json");

  fs.writeFileSync(eventPath, JSON.stringify({ pull_request: breakingPullRequest }));
  assert.deepEqual(readEventPullRequest(eventPath), breakingPullRequest);

  fs.writeFileSync(eventPath, JSON.stringify({ after: squashSha, before: previousSha }));
  assert.deepEqual(readEventPullRequest(eventPath), {});

  assert.throws(
    () => readEventPullRequest(""),
    new Error("GITHUB_EVENT_PATH is required to read the pull-request SemVer marker"),
  );
  fs.rmSync(workspace, { force: true, recursive: true });
});

function runGit(cwd: string, args: string[]): string {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  return result.stdout.trim();
}

// The stub API server shares this process's event loop, so the script has to
// run asynchronously — a blocking spawn would deadlock against its own request.
async function runMarkerScript(workspace: string, env: Record<string, string>): Promise<string> {
  const outputPath = path.join(workspace, "github-output.txt");
  fs.writeFileSync(outputPath, "");
  const child = spawn(
    process.execPath,
    [path.join(root, "tools", "github", "semver-change-marker.mjs")],
    {
      cwd: workspace,
      env: { ...process.env, ...env, GITHUB_OUTPUT: outputPath },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  let stderr = "";
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk: string) => {
    stderr += chunk;
  });
  const [exitCode] = (await once(child, "exit")) as [number | null];
  assert.equal(exitCode, 0, stderr.trim());
  return fs.readFileSync(outputPath, "utf8");
}

async function withPullRequestApi(
  pullRequests: unknown[],
  run: (apiUrl: string, requestedPaths: string[]) => Promise<void>,
): Promise<void> {
  const requestedPaths: string[] = [];
  const server = http.createServer((request, response) => {
    requestedPaths.push(request.url ?? "");
    response.writeHead(200, { "content-type": "application/json" });
    response.end(JSON.stringify(pullRequests));
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address();
  assert.ok(address != null && typeof address !== "string");
  try {
    await run(`http://127.0.0.1:${address.port}`, requestedPaths);
  } finally {
    server.close();
    await once(server, "close");
  }
}

test("the workflow entry point recovers a squashed marker end to end", async () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-semver-marker-push-"));
  runGit(workspace, ["init", "-q", "--initial-branch=main"]);
  fs.writeFileSync(path.join(workspace, "relief.rs"), "pub struct ElementNode;\n");
  runGit(workspace, ["add", "relief.rs"]);
  runGit(workspace, [
    "-c",
    "user.name=Vize",
    "-c",
    "user.email=vize@example.com",
    "commit",
    "--no-verify",
    "-qm",
    squashCommitMessage(breakingPullRequest, []),
  ]);
  const headSha = runGit(workspace, ["rev-parse", "HEAD"]);
  const pushEnv = {
    GITHUB_EVENT_NAME: "push",
    GITHUB_REPOSITORY: "owner/repository",
    GITHUB_SHA: headSha,
    GITHUB_TOKEN: "secret",
  };

  await withPullRequestApi(
    [{ ...breakingPullRequest, merge_commit_sha: headSha }],
    async (apiUrl, requestedPaths) => {
      assert.equal(
        await runMarkerScript(workspace, { ...pushEnv, GITHUB_API_URL: apiUrl }),
        "marker-source=squashed_pull_request\nrelease-type=major\n",
      );
      assert.deepEqual(requestedPaths, [
        `/repos/owner/repository/commits/${headSha}/pulls?per_page=100&page=1`,
      ]);
    },
  );

  await withPullRequestApi([], async (apiUrl) => {
    assert.equal(
      await runMarkerScript(workspace, { ...pushEnv, GITHUB_API_URL: apiUrl }),
      "marker-source=commit_message\nrelease-type=none\n",
    );
  });

  fs.rmSync(workspace, { force: true, recursive: true });
});
