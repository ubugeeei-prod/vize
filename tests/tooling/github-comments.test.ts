import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  isManagedComment as isBenchmarkManagedComment,
  markerForKey as benchmarkMarkerForKey,
} from "../../tools/benchmarks/scripts/comment-pr.mjs";
import {
  TEST_REPORT_MARKER,
  commentRun,
  isManagedComment as isTestReportManagedComment,
  mayReplace,
  publishTestReport,
  readEnv,
  renderTestReport,
  type RunIdentity,
  type WorkflowJob,
} from "../../tools/support/compat/github/comment-test-report.ts";

function comment(body: string, login = "github-actions[bot]") {
  return {
    body,
    user: {
      login,
      type: "Bot",
    },
  };
}

test("PR benchmark comments only match GitHub Actions comments with the marker as the first line", () => {
  const marker = benchmarkMarkerForKey("abc123");

  assert.equal(isBenchmarkManagedComment(comment(`${marker}\n## PR Benchmark`), marker), true);
  assert.equal(isBenchmarkManagedComment(comment(marker), marker), true);
  assert.equal(
    isBenchmarkManagedComment(comment(`> quoted old report\n${marker}\n## PR Benchmark`), marker),
    false,
  );
  assert.equal(
    isBenchmarkManagedComment(comment(`${marker}\n## PR Benchmark`, "dependabot[bot]"), marker),
    false,
  );
  assert.equal(
    isBenchmarkManagedComment(
      comment("<!-- vize-pr-benchmark:def456 -->\n## PR Benchmark"),
      marker,
    ),
    false,
  );
});

const SHA = "a".repeat(40);
const OTHER_SHA = "b".repeat(40);
const run: RunIdentity = { headSha: SHA, runId: 200, runAttempt: 2 };
const server = "https://github.com/ubugeeei-prod/vize";
const job = (name: string, conclusion: string | null, id: number, failedStep?: number) => ({
  name,
  status: conclusion === null ? "in_progress" : "completed",
  conclusion,
  html_url: `${server}/actions/runs/200/job/${id}`,
  steps: [
    { name: "Set up job", number: 1, conclusion: "success" },
    ...(failedStep === undefined
      ? []
      : [{ name: "Run | the tests", number: failedStep, conclusion: "failure" }]),
  ],
});
const apiJobs = [
  job("fmt-rust", "success", 1),
  job("PR source checks / PR Rust workspace products / Rust tests (Compiler)", "failure", 2, 7),
  job("PR source checks / PR Rust workspace products / Rust tests (Type checker)", "success", 3),
  job("PR source checks / Native runtime (ssr)", "cancelled", 4),
  job("cargo-semver-checks (${{ matrix.crate }})", "skipped", 5),
  job("nix-flake", "skipped", 6),
  job("test-report", null, 9),
];
const jobs: WorkflowJob[] = apiJobs.map((entry) => ({
  name: entry.name,
  status: entry.status,
  conclusion: entry.conclusion,
  url: entry.html_url,
  steps: entry.steps,
}));

test("PR test report comments match only the GitHub Actions comment that starts with the marker", () => {
  const managed = (body: string, login = "github-actions[bot]") =>
    isTestReportManagedComment({ id: 1, login, body });
  assert.equal(managed(`${TEST_REPORT_MARKER}\n## Test report`), true);
  assert.equal(managed(TEST_REPORT_MARKER), true);
  assert.equal(managed(`Prior discussion mentions ${TEST_REPORT_MARKER}\n## Test report`), false);
  assert.equal(managed(`${TEST_REPORT_MARKER}\n## Test report`, "github-actions-user[bot]"), false);
  // Per-head comments from before the restore stay as history.
  assert.equal(managed(`<!-- vize-test-report:${SHA} -->\n## Detailed Test Report`), false);
});

const rustCompiler = "Rust tests (Compiler)";
const rustTypes = "Rust tests (Type checker)";
const testCase = (path: string, name: string, outcome: "passed" | "failed" | "skipped") => ({
  path,
  name,
  outcome,
});
const packets = [
  {
    version: 1 as const,
    job: rustCompiler,
    attempt: 1,
    cases: [
      testCase("crates/vize_armature", "vize_armature a", "passed"),
      testCase("crates/vize_armature", "vize_armature b", "passed"),
      testCase("crates/vize_atelier_sfc", "vize_atelier_sfc compile::x_y", "failed"),
      testCase("crates/vize_atelier_vapor", "vize_atelier_vapor v", "passed"),
      testCase("crates/vize", "vize::integration lint_rules::z", "passed"),
      testCase("crates/vize", "vize::integration build_cli::w", "skipped"),
    ],
  },
  {
    version: 1 as const,
    job: rustTypes,
    attempt: 2,
    cases: [
      testCase("crates/vize_canon", "vize_canon t", "passed"),
      testCase("crates/vize_canon", "vize_canon::integration check::y", "failed"),
    ],
  },
];
const input = {
  jobs,
  packets,
  gate: "failure" as const,
  run,
  serverUrl: "https://github.com",
  repository: "ubugeeei-prod/vize",
  workflow: "Check",
};

test("the test report counts cases by product area and lists each failed case", () => {
  const report = renderTestReport(input);
  assert.match(report, /^## Test report: failed$/m);
  assert.ok(report.includes(`[\`${SHA.slice(0, 12)}\`](${server}/commit/${SHA})`));
  assert.ok(report.includes(`[Check run 200, attempt 2](${server}/actions/runs/200/attempts/2)`));
  assert.ok(report.includes(`gate [test-report](${server}/actions/runs/200/job/9)`));
  assert.match(report, /^\*\*8 test cases: 5 passed, 2 failed, 1 skipped\*\*$/m);
  const areas = report
    .split("\n")
    .filter((line) => /^\| [A-Z*]/.test(line) && !line.startsWith("| Area"));
  assert.deepEqual(areas.slice(0, 6), [
    "| Compiler | 2 | 1 | 0 | 3 |",
    "| Vapor | 1 | 0 | 0 | 1 |",
    "| Type checker | 1 | 1 | 0 | 2 |",
    "| Linter | 1 | 0 | 0 | 1 |",
    "| CLI | 0 | 0 | 1 | 1 |",
    "| **Total** | 5 | 2 | 1 | 8 |",
  ]);
  assert.match(report, /^Counted from 2 test job reports\./m);
  const rustJob = (id: number) => `${server}/actions/runs/200/job/${id}`;
  assert.ok(
    report.includes(
      `- Compiler: vize\\_atelier\\_sfc compile::x\\_y · [${rustCompiler}](${rustJob(2)})\n` +
        `- Type checker: vize\\_canon::integration check::y · [${rustTypes}](${rustJob(3)})`,
    ),
  );
  // Jobs stay collapsed, open only when a job failed.
  assert.match(
    report,
    /^<details open>\n<summary>6 jobs: 2 passed, 1 failed, 2 skipped, 1 other<\/summary>$/m,
  );
  assert.match(
    report,
    /^\| PR source checks \/ PR Rust workspace products \/ Rust tests \| 1 \| 1 \| 0 \| 0 \| 2 \|$/m,
  );
  assert.match(report, /^\| PR source checks \/ Native runtime \| 0 \| 0 \| 0 \| 1 \| 0 \|$/m);
  assert.match(report, /^Not run: cargo-semver-checks, nix-flake$/m);
  assert.ok(report.indexOf("Rust tests |") < report.indexOf("fmt-rust |"));
  assert.ok(
    report.includes(
      `- failure: [PR source checks / PR Rust workspace products / Rust tests (Compiler)](${rustJob(2)}), step [Run \\| the tests](${rustJob(2)}#step:7:1)`,
    ),
  );
  assert.ok(
    report.includes(`- cancelled: [PR source checks / Native runtime (ssr)](${rustJob(4)})`),
  );
  assert.doesNotMatch(report, /\| test-report \|/);
  assert.ok(report.indexOf("| Area |") < report.indexOf("<details"));

  const passed = renderTestReport({
    ...input,
    jobs: jobs.filter(
      (entry) => entry.conclusion !== "failure" && entry.conclusion !== "cancelled",
    ),
    packets: [],
    gate: "success",
  });
  assert.match(passed, /^## Test report: passed$/m);
  assert.match(passed, /^\*\*0 test cases: 0 passed, 0 failed, 0 skipped\*\*$/m);
  assert.match(passed, /^<details>$/m);
  assert.doesNotMatch(passed, /Failed test cases|Failed and unfinished jobs/);
  assert.match(
    renderTestReport({ ...input, packets: null }),
    /^Test case reports could not be downloaded; the \[run\]/m,
  );
  // A packet must come from a job of this run.
  assert.throws(
    () => renderTestReport({ ...input, packets: [{ ...packets[0], job: "Rust tests (Gone)" }] }),
    /0 jobs match case packet Rust tests \(Gone\)/,
  );
  assert.throws(
    () =>
      renderTestReport({ ...input, jobs: jobs.filter((entry) => entry.name !== "test-report") }),
    /no test-report job/,
  );
});

test("a run replaces the comment only for the PR's current head and its latest run attempt", () => {
  const at = (headSha: string, runId: number, runAttempt: number) => ({
    headSha,
    runId,
    runAttempt,
  });
  assert.equal(mayReplace(run, SHA, null), true);
  assert.equal(mayReplace(run, SHA, at(OTHER_SHA, 300, 1)), true);
  assert.equal(mayReplace(run, SHA, at(SHA, 200, 1)), true);
  assert.equal(mayReplace(run, SHA, at(SHA, 200, 2)), true);
  assert.equal(mayReplace(run, SHA, at(SHA, 199, 5)), true);
  // Superseded: the PR moved on, a later attempt or a later run already wrote.
  assert.equal(mayReplace(run, OTHER_SHA, null), false);
  assert.equal(mayReplace(run, OTHER_SHA, at(SHA, 100, 1)), false);
  assert.equal(mayReplace(run, SHA, at(SHA, 200, 3)), false);
  assert.equal(mayReplace(run, SHA, at(SHA, 201, 1)), false);

  const state = `<!-- vize-test-report-run sha=${SHA} run=200 attempt=2 -->`;
  assert.deepEqual(commentRun(`${TEST_REPORT_MARKER}\n${state}\n## Test report`), run);
  assert.equal(commentRun(`${TEST_REPORT_MARKER}\n## Test report`), null);
  assert.equal(commentRun(`${TEST_REPORT_MARKER}\n${state.replace("run=200", "run=x")}`), null);
});

type Call = { method: string; url: string; body?: string };
function fakeGitHub(routes: Record<string, { body: unknown; link?: string }>) {
  const calls: Call[] = [];
  const fetchImpl = async (input: string | URL | Request, init?: RequestInit) => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
    const method = init?.method ?? "GET";
    calls.push({ method, url, body: typeof init?.body === "string" ? init.body : undefined });
    const route = routes[`${method} ${url}`];
    if (!route) return new Response("not found", { status: 404 });
    const headers = new Headers(route.link ? { link: route.link } : {});
    return new Response(JSON.stringify(route.body), { status: 200, headers });
  };
  return { calls, fetchImpl: fetchImpl as typeof fetch };
}

const api = "https://api.github.com/repos/ubugeeei-prod/vize";
const jobsUrl = `${api}/actions/runs/200/jobs?filter=latest&per_page=100`;
const commentsUrl = `${api}/issues/7/comments?per_page=100`;
function environment(directory: string, pr: string, cases = "success") {
  return readEnv({
    GITHUB_API_URL: "https://api.github.com",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_RUN_ATTEMPT: "2",
    GITHUB_RUN_ID: "200",
    GITHUB_SERVER_URL: "https://github.com",
    GITHUB_STEP_SUMMARY: path.join(directory, "summary.md"),
    GITHUB_TOKEN: "token",
    GITHUB_WORKFLOW: "Check",
    TEST_REPORT_GATE: "failure",
    TEST_REPORT_HEAD_SHA: SHA,
    TEST_REPORT_PR_NUMBER: pr,
    TEST_REPORT_CASES: cases,
    TEST_REPORT_CASES_DIR: path.join(directory, "cases"),
  });
}
function inDirectory(check: (directory: string) => Promise<void>) {
  return async () => {
    const directory = mkdtempSync(path.join(tmpdir(), "vize-test-report-"));
    try {
      await check(directory);
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  };
}
// The download keeps one directory per artifact, or none for a single one.
function writePackets(directory: string, written: readonly object[]) {
  written.forEach((entry, index) => {
    const dir = path.join(directory, "cases", `test-cases-${index}`);
    mkdirSync(dir, { recursive: true });
    writeFileSync(path.join(dir, `test-cases-${index}.json`), JSON.stringify(entry));
  });
}
const summaryOf = (directory: string) => readFileSync(path.join(directory, "summary.md"), "utf8");
const jobPages = {
  [`GET ${jobsUrl}`]: {
    body: { total_count: apiJobs.length, jobs: apiJobs.slice(0, 3) },
    link: `<${jobsUrl}&page=2>; rel="next"`,
  },
  [`GET ${jobsUrl}&page=2`]: { body: { total_count: apiJobs.length, jobs: apiJobs.slice(3) } },
};
const state = (headSha: string, runId: number, runAttempt: number) =>
  `${TEST_REPORT_MARKER}\n<!-- vize-test-report-run sha=${headSha} run=${runId} attempt=${runAttempt} -->\n## Test report: passed`;

void test(
  "a same-repository PR gets one comment, updated in place on later runs",
  inDirectory(async (directory) => {
    // Attempt 2 re-ran the type checker job; its attempt 1 packet is stale.
    const stale = {
      ...packets[1],
      attempt: 1,
      cases: [testCase("crates/vize_canon", "old", "failed")],
    };
    writePackets(directory, [packets[0], stale, packets[1]]);
    const created = fakeGitHub({
      ...jobPages,
      [`GET ${commentsUrl}`]: { body: [{ id: 1, user: { login: "someone" }, body: "LGTM" }] },
      [`GET ${api}/pulls/7`]: { body: { head: { sha: SHA } } },
      [`POST ${api}/issues/7/comments`]: { body: { id: 50 } },
    });
    assert.equal(
      await publishTestReport(environment(directory, "7"), created.fetchImpl),
      "Created test report comment 50",
    );
    const post = created.calls.find((call) => call.method === "POST");
    const body = (JSON.parse(post?.body ?? "{}") as { body: string }).body;
    assert.ok(body.startsWith(`${TEST_REPORT_MARKER}\n`));
    assert.deepEqual(commentRun(body), run);
    assert.match(body, /^\*\*8 test cases: 5 passed, 2 failed, 1 skipped\*\*$/m);
    assert.doesNotMatch(body, /: old ·/);
    assert.equal(summaryOf(directory), `${body.split("\n").slice(2).join("\n")}\n`);

    const updated = fakeGitHub({
      ...jobPages,
      [`GET ${commentsUrl}`]: {
        body: [
          { id: 1, user: { login: "someone" }, body: state(SHA, 300, 1) },
          { id: 40, user: { login: "github-actions[bot]" }, body: state(OTHER_SHA, 150, 1) },
        ],
      },
      [`GET ${api}/pulls/7`]: { body: { head: { sha: SHA } } },
      [`PATCH ${api}/issues/comments/40`]: { body: { id: 40 } },
    });
    assert.equal(
      await publishTestReport(environment(directory, "7"), updated.fetchImpl),
      "Updated test report comment 40",
    );
    assert.equal(updated.calls.filter((call) => call.method === "POST").length, 0);
  }),
);

void test(
  "a superseded run keeps the newer comment and still writes its step summary",
  inDirectory(async (directory) => {
    for (const [prHead, existing] of [
      [OTHER_SHA, state(SHA, 100, 1)],
      [SHA, state(SHA, 200, 3)],
    ]) {
      const github = fakeGitHub({
        ...jobPages,
        [`GET ${commentsUrl}`]: {
          body: [{ id: 40, user: { login: "github-actions[bot]" }, body: existing }],
        },
        [`GET ${api}/pulls/7`]: { body: { head: { sha: prHead } } },
      });
      assert.match(
        await publishTestReport(environment(directory, "7"), github.fetchImpl),
        /^Kept test report comment 40: superseded/,
      );
      assert.deepEqual(
        github.calls.filter((call) => call.method !== "GET"),
        [],
      );
    }
    // No job published cases, so the download made no directory.
    assert.match(summaryOf(directory), /\*\*0 test cases: 0 passed, 0 failed, 0 skipped\*\*/);
  }),
);

void test(
  "a fork PR or merge group writes only the step summary",
  inDirectory(async (directory) => {
    const github = fakeGitHub(jobPages);
    assert.match(
      await publishTestReport(environment(directory, "", "failure"), github.fetchImpl),
      /^No PR comment/,
    );
    assert.deepEqual(
      github.calls.map((call) => call.url),
      [jobsUrl, `${jobsUrl}&page=2`],
    );
    assert.match(summaryOf(directory), /^Test case reports could not be downloaded/m);
  }),
);

void test(
  "an oversized report is cut at a line within GitHub's comment limit and points to the summary",
  inDirectory(async (directory) => {
    const failures = Array.from({ length: 2000 }, (_, index) =>
      testCase(
        "crates/vize_canon",
        `vize_canon ${"case_".repeat(10)}${String(index).padStart(4, "0")}`,
        "failed",
      ),
    );
    writePackets(directory, [{ ...packets[1], cases: failures }]);
    const github = fakeGitHub({
      ...jobPages,
      [`GET ${commentsUrl}`]: { body: [] },
      [`GET ${api}/pulls/7`]: { body: { head: { sha: SHA } } },
      [`POST ${api}/issues/7/comments`]: { body: { id: 51 } },
    });
    await publishTestReport(environment(directory, "7"), github.fetchImpl);
    const post = github.calls.find((call) => call.method === "POST");
    const body = (JSON.parse(post?.body ?? "{}") as { body: string }).body;
    const note = `\n\nTruncated: the [run summary](${server}/actions/runs/200/attempts/2) holds the whole report.`;
    assert.ok(body.length <= 65_000, `${body.length}`);
    assert.ok(body.endsWith(note));
    assert.deepEqual(commentRun(body), run);
    // Whole lines only: the last kept entry is complete.
    assert.match(body.slice(0, -note.length).split("\n").at(-1) ?? "", /job\/3\)$/);
    const whole = summaryOf(directory);
    assert.ok(whole.length > 65_000);
    assert.ok(whole.includes("case\\_1999"));
  }),
);

test("the test report refuses inputs it cannot place", () => {
  const valid = {
    GITHUB_API_URL: "https://api.github.com",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_RUN_ATTEMPT: "1",
    GITHUB_RUN_ID: "200",
    GITHUB_SERVER_URL: "https://github.com",
    GITHUB_STEP_SUMMARY: "summary.md",
    GITHUB_TOKEN: "token",
    GITHUB_WORKFLOW: "Check",
    TEST_REPORT_GATE: "success",
    TEST_REPORT_HEAD_SHA: SHA,
    TEST_REPORT_PR_NUMBER: "",
    TEST_REPORT_CASES: "success",
    TEST_REPORT_CASES_DIR: "cases",
  };
  assert.equal(readEnv(valid).pullRequest, null);
  assert.equal(readEnv(valid).casesDir, "cases");
  assert.equal(readEnv({ ...valid, TEST_REPORT_CASES: "failure" }).casesDir, null);
  assert.throws(() => readEnv({ ...valid, TEST_REPORT_CASES: "skipped" }), /TEST_REPORT_CASES/);
  assert.throws(() => readEnv({ ...valid, TEST_REPORT_CASES_DIR: "" }), /TEST_REPORT_CASES_DIR/);
  assert.throws(() => readEnv({ ...valid, TEST_REPORT_GATE: "skipped" }), /TEST_REPORT_GATE/);
  assert.throws(() => readEnv({ ...valid, TEST_REPORT_HEAD_SHA: "main" }), /TEST_REPORT_HEAD_SHA/);
  assert.throws(() => readEnv({ ...valid, TEST_REPORT_PR_NUMBER: "7 " }), /TEST_REPORT_PR_NUMBER/);
  assert.throws(() => readEnv({ ...valid, GITHUB_TOKEN: "" }), /GITHUB_TOKEN/);
});

void test(
  "pagination never sends the token outside the API host",
  inDirectory(async (directory) => {
    const github = fakeGitHub({
      [`GET ${jobsUrl}`]: {
        body: { total_count: 1, jobs: apiJobs.slice(0, 1) },
        link: `<https://example.com/jobs?page=2>; rel="next"`,
      },
    });
    await assert.rejects(
      publishTestReport(environment(directory, ""), github.fetchImpl),
      /outside https:\/\/api\.github\.com/,
    );
    assert.equal(github.calls.length, 1);
  }),
);
