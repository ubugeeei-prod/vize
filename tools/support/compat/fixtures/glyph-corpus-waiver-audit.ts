import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { auditKnownViolationIssues, loadKnownViolationLedger } from "./glyph-corpus.ts";
import type { KnownViolation } from "./glyph-corpus-waivers.ts";

type Environment = Readonly<Record<string, string | undefined>>;
type Request = (url: string, init: { headers: Record<string, string> }) => Promise<Response>;

interface IssueRecord {
  readonly number: unknown;
  readonly state: unknown;
  readonly title: unknown;
  readonly url: unknown;
  readonly updatedAt: unknown;
}

function parseArguments(argv: readonly string[]): string {
  const [flag, output] = argv;
  if (argv.length !== 2 || flag !== "--output" || output === undefined || output.startsWith("-")) {
    throw new Error("usage: node glyph-corpus-waiver-audit.ts --output <path>");
  }
  return resolve(output);
}

async function resolveGitHubIssue(
  number: number,
  environment: Environment,
  request: Request = fetch,
): Promise<IssueRecord> {
  const repository = environment.GITHUB_REPOSITORY ?? "ubugeeei-prod/vize";
  const apiUrl = environment.GITHUB_API_URL ?? "https://api.github.com";
  const headers: Record<string, string> = {
    Accept: "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
  };
  if (environment.GITHUB_TOKEN) headers.Authorization = `Bearer ${environment.GITHUB_TOKEN}`;
  const response = await request(`${apiUrl}/repos/${repository}/issues/${number}`, { headers });
  if (!response.ok) {
    throw new Error(`tracking Issue #${number} lookup failed with HTTP ${response.status}`);
  }
  const issue = (await response.json()) as Partial<Record<string, unknown>>;
  return {
    number: issue.number,
    state: issue.state,
    title: issue.title,
    url: issue.html_url,
    updatedAt: issue.updated_at,
  };
}

export async function createWaiverIssueAudit(
  entries: readonly KnownViolation[],
  environment: Environment = process.env,
  request: Request = fetch,
) {
  const issues = await auditKnownViolationIssues(entries, (number) =>
    resolveGitHubIssue(number, environment, request),
  );
  const waiverCounts = new Map<number, number>();
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
