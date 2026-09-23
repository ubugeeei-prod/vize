import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { auditKnownViolationIssues, loadKnownViolationLedger } from "./glyph-corpus.mjs";

function parseArguments(argv) {
  if (argv.length !== 2 || argv[0] !== "--output" || argv[1].startsWith("-")) {
    throw new Error("usage: node glyph-corpus-waiver-audit.mjs --output <path>");
  }
  return resolve(argv[1]);
}

async function resolveGitHubIssue(number, environment, request = fetch) {
  const repository = environment.GITHUB_REPOSITORY ?? "ubugeeei-prod/vize";
  const apiUrl = environment.GITHUB_API_URL ?? "https://api.github.com";
  const headers = {
    Accept: "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
  };
  if (environment.GITHUB_TOKEN) headers.Authorization = `Bearer ${environment.GITHUB_TOKEN}`;
  const response = await request(`${apiUrl}/repos/${repository}/issues/${number}`, { headers });
  if (!response.ok) {
    throw new Error(`tracking Issue #${number} lookup failed with HTTP ${response.status}`);
  }
  const issue = await response.json();
  return {
    number: issue.number,
    state: issue.state,
    title: issue.title,
    url: issue.html_url,
    updatedAt: issue.updated_at,
  };
}

export async function createWaiverIssueAudit(entries, environment = process.env, request = fetch) {
  const issues = await auditKnownViolationIssues(entries, (number) =>
    resolveGitHubIssue(number, environment, request),
  );
  const waiverCounts = new Map();
  for (const entry of entries) {
    waiverCounts.set(entry.trackingIssue, (waiverCounts.get(entry.trackingIssue) ?? 0) + 1);
  }
  return {
    schema: "vize.glyphCorpusWaiverIssueAudit",
    version: 1,
    repository: environment.GITHUB_REPOSITORY ?? "ubugeeei-prod/vize",
    sourceCommit: environment.GITHUB_SHA ?? null,
    generatedAt: new Date().toISOString(),
    waiverCount: entries.length,
    issues: issues.map((issue) => ({
      ...issue,
      state: issue.state.toUpperCase(),
      waiverCount: waiverCounts.get(issue.number),
    })),
  };
}

async function main() {
  const output = parseArguments(process.argv.slice(2));
  const artifact = await createWaiverIssueAudit(loadKnownViolationLedger());
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, `${JSON.stringify(artifact, null, 2)}\n`);
  process.stdout.write(
    `audited ${artifact.waiverCount} formatter waiver(s) across ${artifact.issues.length} open Issue(s)\n`,
  );
}

const isMain = process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isMain) {
  main().catch((error) => {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  });
}
