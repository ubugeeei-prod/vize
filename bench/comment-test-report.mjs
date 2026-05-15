#!/usr/bin/env node
/**
 * Create or update a detailed PR test report comment for a workflow run.
 */

import { appendFileSync, readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const MARKER_NAME = "vize-test-report";
const MAX_COMMENT_LENGTH = 64_000;

const AREA_ORDER = new Map([
  ["JS / TS", 0],
  ["Rust", 1],
  ["E2E / VRT", 2],
  ["VRT", 3],
  ["E2E", 4],
  ["Infra", 5],
  ["Other", 6],
]);

const FAILURE_RESULTS = new Set(["failure", "timed_out", "startup_failure", "action_required"]);
const ACTIVE_RESULTS = new Set(["in_progress", "queued", "requested", "waiting", "pending"]);

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

function markerForKey(key) {
  const trimmed = key?.trim();
  if (!trimmed) {
    return `<!-- ${MARKER_NAME} -->`;
  }
  if (trimmed.includes("-->") || /[\r\n]/.test(trimmed)) {
    throw new Error("Invalid test report comment key");
  }
  return `<!-- ${MARKER_NAME}:${trimmed} -->`;
}

function parseNextLink(linkHeader) {
  if (!linkHeader) {
    return null;
  }
  for (const part of linkHeader.split(",")) {
    const match = /<([^>]+)>;\s*rel="next"/.exec(part);
    if (match) {
      return match[1];
    }
  }
  return null;
}

async function githubFetch(path, options = {}) {
  const token = requireValue(process.env.GITHUB_TOKEN || process.env.GH_TOKEN, "GITHUB_TOKEN");
  const url = path.startsWith("https://") ? path : `https://api.github.com${path}`;
  const response = await fetch(url, {
    ...options,
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
      "X-GitHub-Api-Version": "2022-11-28",
      ...options.headers,
    },
  });

  if (!response.ok) {
    const body = await response.text();
    throw new Error(`GitHub API ${response.status} ${response.statusText}: ${body}`);
  }

  if (response.status === 204) {
    return { data: null, headers: response.headers };
  }
  return { data: await response.json(), headers: response.headers };
}

async function githubRequest(path, options = {}) {
  const { data } = await githubFetch(path, options);
  return data;
}

async function githubRequestPages(path, getItems) {
  const items = [];
  let nextPath = path;
  while (nextPath) {
    const { data, headers } = await githubFetch(nextPath);
    items.push(...getItems(data));
    nextPath = parseNextLink(headers.get("link"));
  }
  return items;
}

function normalizeResult(entity) {
  return entity.conclusion || entity.status || "unknown";
}

function resultRank(result) {
  const normalized = result.toLowerCase();
  if (FAILURE_RESULTS.has(normalized)) {
    return 0;
  }
  if (normalized === "cancelled") {
    return 1;
  }
  if (ACTIVE_RESULTS.has(normalized)) {
    return 2;
  }
  if (normalized === "skipped") {
    return 3;
  }
  if (normalized === "success" || normalized === "completed") {
    return 4;
  }
  return 2;
}

function aggregateResult(results) {
  if (results.length === 0) {
    return "unknown";
  }
  return results.reduce((worst, result) =>
    resultRank(result) < resultRank(worst) ? result : worst,
  );
}

function durationMs(startedAt, completedAt) {
  if (!startedAt || !completedAt) {
    return null;
  }
  const started = Date.parse(startedAt);
  const completed = Date.parse(completedAt);
  if (!Number.isFinite(started) || !Number.isFinite(completed) || completed < started) {
    return null;
  }
  return completed - started;
}

function formatDuration(ms) {
  if (ms == null) {
    return "-";
  }
  const totalSeconds = Math.max(0, Math.round(ms / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  if (hours > 0) {
    return `${hours}h ${minutes}m ${seconds}s`;
  }
  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }
  return `${seconds}s`;
}

function formatTimestamp(value) {
  if (!value) {
    return "-";
  }
  return new Date(value)
    .toISOString()
    .replace("T", " ")
    .replace(/\.\d{3}Z$/, " UTC");
}

function escapeTable(value) {
  return String(value ?? "")
    .replace(/\|/g, "\\|")
    .replace(/\r?\n/g, "<br>");
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function classifyJob(job) {
  const name = job.name.toLowerCase();
  if (name === "check-js") {
    return {
      area: "JS / TS",
      focus: "workspace checks, script tests, editor extension packaging, JS package builds",
    };
  }
  if (name === "fmt-rust") {
    return { area: "Rust", focus: "cargo fmt" };
  }
  if (name === "clippy-and-test") {
    return { area: "Rust", focus: "cargo clippy and cargo test" };
  }
  if (name === "coverage") {
    return { area: "Rust", focus: "coverage summary" };
  }
  if (name === "playground-test") {
    return { area: "E2E / VRT", focus: "playground browser tests and visual snapshots" };
  }
  if (name === "nix-flake") {
    return { area: "Infra", focus: "nix flake check" };
  }
  if (name.includes("vrt")) {
    return { area: "VRT", focus: "visual regression testing" };
  }
  if (name.includes("e2e") || name.includes("playwright")) {
    return { area: "E2E", focus: "end-to-end testing" };
  }
  return { area: "Other", focus: "workflow job" };
}

function isReportJob(job) {
  return job.name === "test-report" || job.name.startsWith("test-report ");
}

function compareJobs(a, b) {
  const areaA = classifyJob(a).area;
  const areaB = classifyJob(b).area;
  const orderA = AREA_ORDER.get(areaA) ?? AREA_ORDER.get("Other");
  const orderB = AREA_ORDER.get(areaB) ?? AREA_ORDER.get("Other");
  if (orderA !== orderB) {
    return orderA - orderB;
  }
  return Date.parse(a.started_at || "") - Date.parse(b.started_at || "");
}

function resultCounts(jobs) {
  const counts = new Map();
  for (const job of jobs) {
    const result = normalizeResult(job);
    counts.set(result, (counts.get(result) ?? 0) + 1);
  }
  return [...counts.entries()]
    .sort(([a], [b]) => resultRank(a) - resultRank(b) || a.localeCompare(b))
    .map(([result, count]) => `${count} ${result}`)
    .join(", ");
}

function groupByArea(jobs) {
  const groups = new Map();
  for (const job of jobs) {
    const classification = classifyJob(job);
    const entry = groups.get(classification.area) ?? [];
    entry.push({ job, classification });
    groups.set(classification.area, entry);
  }
  return [...groups.entries()].sort(([a], [b]) => {
    const orderA = AREA_ORDER.get(a) ?? AREA_ORDER.get("Other");
    const orderB = AREA_ORDER.get(b) ?? AREA_ORDER.get("Other");
    return orderA - orderB;
  });
}

function totalDuration(jobs) {
  return jobs.reduce((sum, job) => sum + (durationMs(job.started_at, job.completed_at) ?? 0), 0);
}

function buildAreaSummary(jobs) {
  const lines = [
    "### Area Summary",
    "",
    "| Area | Jobs | Result | Runner Time |",
    "| --- | ---: | --- | ---: |",
  ];

  for (const [area, entries] of groupByArea(jobs)) {
    const areaJobs = entries.map((entry) => entry.job);
    const result = aggregateResult(areaJobs.map(normalizeResult));
    lines.push(
      `| ${escapeTable(area)} | ${areaJobs.length} | ${escapeTable(result)} | ${formatDuration(
        totalDuration(areaJobs),
      )} |`,
    );
  }

  return lines;
}

function buildJobOverview(jobs) {
  const lines = [
    "### Job Overview",
    "",
    "| Area | Job | Focus | Result | Duration | Log |",
    "| --- | --- | --- | --- | ---: | --- |",
  ];

  for (const job of jobs) {
    const classification = classifyJob(job);
    const result = normalizeResult(job);
    const duration = formatDuration(durationMs(job.started_at, job.completed_at));
    const log = job.html_url ? `[open](${job.html_url})` : "-";
    lines.push(
      `| ${escapeTable(classification.area)} | ${escapeTable(job.name)} | ${escapeTable(
        classification.focus,
      )} | ${escapeTable(result)} | ${duration} | ${log} |`,
    );
  }

  return lines;
}

function buildInventorySummary(inventory) {
  if (!inventory) {
    return [];
  }

  const lines = [
    "### Test Inventory",
    "",
    `Total tracked cases: **${inventory.totalCases}** across **${inventory.totalFiles}** files.`,
    "",
    "| Area | Files | Cases |",
    "| --- | ---: | ---: |",
  ];

  for (const area of inventory.areas ?? []) {
    lines.push(`| ${escapeTable(area.area)} | ${area.files} | ${area.cases} |`);
  }

  lines.push("");
  lines.push("<details>");
  lines.push("<summary>Files</summary>");
  lines.push("");
  lines.push("| Area | File | Cases |");
  lines.push("| --- | --- | ---: |");
  for (const group of inventory.groups ?? []) {
    lines.push(`| ${escapeTable(group.area)} | \`${escapeTable(group.file)}\` | ${group.count} |`);
  }
  lines.push("</details>");
  lines.push("");
  lines.push(
    "Full per-test names are written to the workflow summary and uploaded as an artifact.",
  );

  return lines;
}

function buildStepDetails(jobs) {
  const lines = ["### Step Details", ""];

  for (const job of jobs) {
    const classification = classifyJob(job);
    const result = normalizeResult(job);
    const duration = formatDuration(durationMs(job.started_at, job.completed_at));
    const shouldOpen = resultRank(result) <= resultRank("cancelled");
    lines.push(`<details${shouldOpen ? " open" : ""}>`);
    lines.push(
      `<summary>${escapeHtml(
        `${classification.area} / ${job.name}: ${result}, ${duration}`,
      )}</summary>`,
    );
    lines.push("");
    if (job.html_url) {
      lines.push(`[Open job log](${job.html_url})`);
      lines.push("");
    }
    lines.push("| # | Step | Result | Duration | Started | Completed |");
    lines.push("| ---: | --- | --- | ---: | --- | --- |");
    for (const step of job.steps ?? []) {
      const stepResult = normalizeResult(step);
      const stepDuration = formatDuration(durationMs(step.started_at, step.completed_at));
      lines.push(
        `| ${step.number ?? "-"} | ${escapeTable(step.name)} | ${escapeTable(
          stepResult,
        )} | ${stepDuration} | ${escapeTable(formatTimestamp(step.started_at))} | ${escapeTable(
          formatTimestamp(step.completed_at),
        )} |`,
      );
    }
    lines.push("</details>");
    lines.push("");
  }

  return lines;
}

function truncateComment(body) {
  if (body.length <= MAX_COMMENT_LENGTH) {
    return body;
  }
  const suffix = [
    "",
    "",
    `Comment truncated at ${MAX_COMMENT_LENGTH} characters. Open the workflow run for the full job log.`,
  ].join("\n");
  return `${body.slice(0, MAX_COMMENT_LENGTH - suffix.length)}${suffix}`;
}

export function buildComment({ jobs, workflowName, runUrl, runId, runAttempt, sha, inventory }) {
  const reportJobs = jobs.filter((job) => !isReportJob(job)).sort(compareJobs);
  const shortSha = sha ? sha.slice(0, 12) : "unknown";
  const overall = aggregateResult(reportJobs.map(normalizeResult));
  const startedAt = reportJobs
    .map((job) => job.started_at)
    .filter(Boolean)
    .sort()[0];
  const completedAt = reportJobs
    .map((job) => job.completed_at)
    .filter(Boolean)
    .sort()
    .at(-1);

  const lines = [
    "## Detailed Test Report",
    "",
    `Commit: \`${shortSha}\``,
    `Workflow: ${runUrl ? `[${workflowName} #${runId}](${runUrl})` : `${workflowName} #${runId}`}`,
    `Attempt: \`${runAttempt || "1"}\``,
    `Overall: **${overall}**${reportJobs.length ? ` (${resultCounts(reportJobs)})` : ""}`,
    `Wall Time: ${formatDuration(durationMs(startedAt, completedAt))}`,
    "",
    ...buildAreaSummary(reportJobs),
    "",
    ...buildInventorySummary(inventory),
    "",
    ...buildJobOverview(reportJobs),
    "",
    ...buildStepDetails(reportJobs),
  ];

  return truncateComment(lines.join("\n"));
}

async function commentOnPr({ repo, prNumber, marker, body }) {
  const comments = await githubRequestPages(
    `/repos/${repo}/issues/${prNumber}/comments?per_page=100`,
    (data) => data,
  );
  const existing = comments.find(
    (comment) => comment.user?.type === "Bot" && comment.body?.includes(marker),
  );

  if (existing) {
    await githubRequest(`/repos/${repo}/issues/comments/${existing.id}`, {
      method: "PATCH",
      body: JSON.stringify({ body }),
    });
    console.log(`Updated test report comment ${existing.id}`);
  } else {
    const created = await githubRequest(`/repos/${repo}/issues/${prNumber}/comments`, {
      method: "POST",
      body: JSON.stringify({ body }),
    });
    console.log(`Created test report comment ${created.id}`);
  }
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const repo = requireValue(process.env.GITHUB_REPOSITORY, "GITHUB_REPOSITORY");
  const prNumber = requireValue(args["pr-number"] ?? process.env.PR_NUMBER, "PR_NUMBER");
  const runId = requireValue(
    args["run-id"] ?? process.env.TEST_REPORT_RUN_ID ?? process.env.GITHUB_RUN_ID,
    "GITHUB_RUN_ID",
  );
  const runAttempt = args["run-attempt"] ?? process.env.TEST_REPORT_RUN_ATTEMPT;
  const workflowName = args["workflow-name"] ?? process.env.TEST_REPORT_WORKFLOW_NAME ?? "Check";
  const sha = args.sha ?? process.env.TEST_REPORT_HEAD_SHA ?? process.env.GITHUB_SHA;
  const runUrl =
    args["run-url"] ??
    process.env.TEST_REPORT_RUN_URL ??
    (process.env.GITHUB_SERVER_URL && process.env.GITHUB_REPOSITORY
      ? `${process.env.GITHUB_SERVER_URL}/${process.env.GITHUB_REPOSITORY}/actions/runs/${runId}`
      : "");
  const marker = markerForKey(args["comment-key"] ?? process.env.TEST_REPORT_COMMENT_KEY ?? sha);
  const jobs = await githubRequestPages(
    `/repos/${repo}/actions/runs/${runId}/jobs?per_page=100`,
    (data) => data.jobs ?? [],
  );
  const inventory = args.inventory ? JSON.parse(readFileSync(args.inventory, "utf8")) : null;
  const report = buildComment({ jobs, workflowName, runUrl, runId, runAttempt, sha, inventory });
  const body = `${marker}\n${report}`;

  if (args.summary) {
    appendFileSync(args.summary, `${report}\n`);
  }

  await commentOnPr({ repo, prNumber, marker, body });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
