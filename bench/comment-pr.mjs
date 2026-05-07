#!/usr/bin/env node
/**
 * Create or update the PR benchmark comment.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const MARKER = "<!-- vize-pr-benchmark -->";

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (!arg.startsWith("--")) {
      continue;
    }
    const key = arg.slice(2);
    const next = argv[i + 1];
    if (next == null || next.startsWith("--")) {
      args[key] = "true";
    } else {
      args[key] = next;
      i++;
    }
  }
  return args;
}

function requireValue(value, name) {
  if (!value) {
    throw new Error(`Missing ${name}`);
  }
  return value;
}

async function githubRequest(path, options = {}) {
  const token = requireValue(process.env.GITHUB_TOKEN || process.env.GH_TOKEN, "GITHUB_TOKEN");
  const response = await fetch(`https://api.github.com${path}`, {
    ...options,
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
      "X-GitHub-Api-Version": "2022-11-28",
      ...(options.headers ?? {}),
    },
  });

  if (!response.ok) {
    const body = await response.text();
    throw new Error(`GitHub API ${response.status} ${response.statusText}: ${body}`);
  }

  if (response.status === 204) {
    return null;
  }
  return response.json();
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const repo = requireValue(process.env.GITHUB_REPOSITORY, "GITHUB_REPOSITORY");
  const prNumber = requireValue(process.env.PR_NUMBER, "PR_NUMBER");
  const bodyPath = resolve(requireValue(args.body, "--body"));
  const benchmarkBody = readFileSync(bodyPath, "utf8");
  const body = `${MARKER}\n${benchmarkBody}`;

  const comments = await githubRequest(`/repos/${repo}/issues/${prNumber}/comments?per_page=100`);
  const existing = comments.find(
    (comment) => comment.user?.type === "Bot" && comment.body?.includes(MARKER),
  );

  if (existing) {
    await githubRequest(`/repos/${repo}/issues/comments/${existing.id}`, {
      method: "PATCH",
      body: JSON.stringify({ body }),
    });
    console.log(`Updated benchmark comment ${existing.id}`);
  } else {
    const created = await githubRequest(`/repos/${repo}/issues/${prNumber}/comments`, {
      method: "POST",
      body: JSON.stringify({ body }),
    });
    console.log(`Created benchmark comment ${created.id}`);
  }
}

try {
  await main();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
