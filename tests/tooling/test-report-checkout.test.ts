import assert from "node:assert/strict";
import { execFile, execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parse } from "yaml";

const root = fileURLToPath(new URL("../../", import.meta.url));
const job = parse(readFileSync(join(root, ".github/workflows/check.yml"), "utf8")).jobs[
  "test-report"
];
const checkoutInputs =
  job.steps.find((step: { uses?: string }) => step.uses?.startsWith("actions/checkout@"))?.with ??
  {};
const runtime = [
  "package.json",
  ...[
    "require-needs-success",
    "comment-test-report",
    "test-cases",
    "test-report-markdown",
    "test-areas",
  ].map((name) => `tools/support/compat/github/${name}.ts`),
  "tools/support/json-value.ts",
].sort();
const gateArgs = job.steps
  .find((step: { id?: string }) => step.id === "gate")
  .run.trim()
  .split(/\s+/)
  .slice(1);
const reportArgs = job.steps
  .find((step: { name?: string }) => step.name === "Report test results")
  .run.trim()
  .split(/\s+/)
  .slice(1);
const sha = "a".repeat(40);

function put(root: string, file: string, value: string) {
  mkdirSync(dirname(join(root, file)), { recursive: true });
  writeFileSync(join(root, file), value);
}
function git(cwd: string, ...args: string[]) {
  return execFileSync("git", args, { cwd, encoding: "utf8", stdio: "pipe" }).trim();
}
function files(cwd: string, prefix = ""): string[] {
  return readdirSync(join(cwd, prefix), { withFileTypes: true })
    .flatMap((entry) => {
      if (entry.name === ".git") return [];
      const name = prefix ? `${prefix}/${entry.name}` : entry.name;
      return entry.isDirectory() ? files(cwd, name) : [name];
    })
    .sort();
}
async function execute(cwd: string, args: string[], env: NodeJS.ProcessEnv) {
  return new Promise<{ code: number; stdout: string; stderr: string }>((resolve) => {
    execFile(
      process.execPath,
      args,
      { cwd, env: { ...process.env, ...env }, encoding: "utf8" },
      (error, stdout, stderr) => resolve({ code: error ? Number(error.code) : 0, stdout, stderr }),
    );
  });
}

await test("the required report runs from its actual checkout closure without product sources", async (t) => {
  const scratch = mkdtempSync(join(tmpdir(), "vize-report-checkout-"));
  t.after(() => rmSync(scratch, { recursive: true, force: true }));
  const origin = join(scratch, "origin");
  mkdirSync(origin);
  git(origin, "init", "-q");
  for (const [key, value] of [
    ["user.name", "CI fixture"],
    ["user.email", "ci@example.invalid"],
    ["commit.gpgsign", "false"],
    ["core.hooksPath", "/dev/null"],
    ["uploadpack.allowFilter", "true"],
  ])
    git(origin, "config", key, value);
  for (const file of runtime) put(origin, file, readFileSync(join(root, file), "utf8"));
  put(origin, "Cargo.toml", "invalid manifest: report imports must never parse product metadata\n");
  put(origin, "crates/unrelated/src/lib.rs", "unrelated product bytes\n");
  git(origin, "add", ".");
  git(origin, "commit", "-qm", "report runtime and unrelated source controls");
  const head = git(origin, "rev-parse", "HEAD");
  function checkout(name: string, selected: boolean) {
    const cwd = join(scratch, name);
    mkdirSync(cwd);
    git(cwd, "init", "-q");
    git(cwd, "remote", "add", "origin", pathToFileURL(origin).href);
    const sparse = selected ? checkoutInputs["sparse-checkout"] : undefined;
    assert.equal(checkoutInputs["fetch-depth"] ?? 1, 1);
    git(
      cwd,
      "fetch",
      "--no-tags",
      "--depth=1",
      ...(sparse ? ["--filter=blob:none"] : []),
      "origin",
      head,
    );
    if (sparse) {
      assert.equal(checkoutInputs["sparse-checkout-cone-mode"], false);
      git(cwd, "config", "core.sparseCheckout", "true");
      put(cwd, ".git/info/sparse-checkout", `${sparse.trim()}\n`);
    }
    git(cwd, "checkout", "-q", "--detach", "FETCH_HEAD");
    return cwd;
  }
  const full = checkout("full", false),
    selected = checkout("selected", true);
  assert.equal(job["timeout-minutes"], 5);
  const mutations = ["success", "failure", "cancelled", "skipped", "unknown", "missing"];
  for (const event of ["pull_request", "merge_group"]) {
    for (const state of mutations)
      await t.test(`${event} preserves the complete ${state} needs gate`, async () => {
        const needs = Object.fromEntries(
          job.needs.map((name: string) => [
            name,
            {
              result:
                name === "instruction-counts" && event === "pull_request" ? "skipped" : "success",
            },
          ]),
        );
        if (state === "missing") delete needs["iteration-plan"].result;
        else if (state !== "success") needs["iteration-plan"].result = state;
        const env = { GITHUB_EVENT_NAME: event, NEEDS_JSON: JSON.stringify(needs) };
        const actual = await execute(selected, gateArgs, env),
          reference = await execute(full, gateArgs, env);
        assert.equal(actual.code, state === "success" ? 0 : 1);
        assert.equal(actual.code, reference.code);
        assert.equal(actual.stdout, reference.stdout);
        assert.equal(
          actual.stderr.replaceAll(selected, "<checkout>"),
          reference.stderr.replaceAll(full, "<checkout>"),
        );
      });
  }
  const posts: unknown[] = [];
  let requests = 0;
  let reportGate = "success";
  const server = createServer(async (request, response) => {
    requests++;
    response.setHeader("Content-Type", "application/json");
    const target = request.url ?? "";
    if (target.includes("/actions/runs/41/jobs"))
      response.end(
        JSON.stringify({
          jobs: [
            {
              name: "Compiler",
              status: "completed",
              conclusion: reportGate,
              html_url: "https://github.example.invalid/run/41/job/1",
              steps: [],
            },
            {
              name: "test-report",
              status: "in_progress",
              conclusion: null,
              html_url: "https://github.example.invalid/run/41/job/2",
              steps: [],
            },
          ],
        }),
      );
    else if (target.includes("/pulls/7")) response.end(JSON.stringify({ head: { sha } }));
    else if (request.method === "POST" && target.includes("/issues/7/comments")) {
      const chunks: Buffer[] = [];
      for await (const chunk of request) chunks.push(chunk);
      posts.push(JSON.parse(Buffer.concat(chunks).toString()));
      response.end(JSON.stringify({ id: 73 }));
    } else if (target.includes("/issues/7/comments")) response.end("[]");
    else {
      response.statusCode = 500;
      response.end(JSON.stringify({ unexpected: target }));
    }
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(
    () =>
      new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      ),
  );
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  const port = address.port;
  const cases = join(scratch, "downloaded-cases");
  mkdirSync(cases);
  put(
    cases,
    "compiler.json",
    JSON.stringify({
      version: 1,
      job: "Compiler",
      attempt: 1,
      cases: ["passed", "failed", "skipped"].map((outcome) => ({
        path: "crates/vize_armature",
        name: `${outcome} case`,
        outcome,
      })),
    }),
  );
  function env(summary: string, gate: string, pr: string): NodeJS.ProcessEnv {
    return {
      GITHUB_API_URL: `http://127.0.0.1:${port}`,
      GITHUB_SERVER_URL: "https://github.example.invalid",
      GITHUB_REPOSITORY: "example/vize",
      GITHUB_WORKFLOW: "Check",
      GITHUB_RUN_ID: "41",
      GITHUB_RUN_ATTEMPT: "1",
      GITHUB_TOKEN: "fixture-token",
      GITHUB_STEP_SUMMARY: summary,
      TEST_REPORT_GATE: gate,
      TEST_REPORT_CASES: "success",
      TEST_REPORT_CASES_DIR: cases,
      TEST_REPORT_HEAD_SHA: sha,
      TEST_REPORT_PR_NUMBER: pr,
    };
  }
  for (const gate of ["success", "failure"])
    for (const pr of ["7", ""])
      await t.test(
        `report ${gate} preserves ${pr ? "PR comment" : "summary-only"} bytes`,
        async () => {
          const summaries = [
            join(scratch, `selected-${gate}-${pr}.md`),
            join(scratch, `full-${gate}-${pr}.md`),
          ];
          reportGate = gate;
          put(
            cases,
            "compiler.json",
            JSON.stringify({
              version: 1,
              job: "Compiler",
              attempt: 1,
              cases: ["passed", gate === "failure" ? "failed" : "passed", "skipped"].map(
                (outcome) => ({
                  path: "crates/vize_armature",
                  name: `${outcome} case`,
                  outcome,
                }),
              ),
            }),
          );
          const before = posts.length;
          const actual = await execute(selected, reportArgs, env(summaries[0], gate, pr));
          const reference = await execute(full, reportArgs, env(summaries[1], gate, pr));
          assert.equal(actual.code, 0, actual.stderr);
          assert.deepEqual(actual, reference);
          assert.equal(readFileSync(summaries[0], "utf8"), readFileSync(summaries[1], "utf8"));
          if (pr) {
            assert.equal(posts.length, before + 2);
            assert.deepEqual(posts[before], posts[before + 1]);
          } else assert.equal(posts.length, before);
        },
      );
  await t.test(
    "a malformed downloaded packet refuses report publication in both checkouts",
    async () => {
      put(cases, "compiler.json", JSON.stringify({ version: 999 }));
      const before = posts.length;
      for (const [name, cwd] of [
        ["selected", selected],
        ["full", full],
      ]) {
        const summary = join(scratch, `${name}-invalid.md`);
        const result = await execute(cwd, reportArgs, env(summary, "success", "7"));
        assert.notEqual(result.code, 0);
        assert.match(result.stderr, /unknown version/);
        assert.equal(existsSync(summary), false);
      }
      assert.equal(posts.length, before);
    },
  );
  await t.test("the configured checkout materializes only the seven report runtime files", () => {
    assert.deepEqual(files(selected), runtime);
    assert.equal(git(selected, "config", "--get", "remote.origin.partialclonefilter"), "blob:none");
    assert.ok(files(full).includes("Cargo.toml"));
  });
  await t.test("a missing report helper fails before summary or API requests", async () => {
    rmSync(join(selected, "tools/support/json-value.ts"));
    const summary = join(scratch, "missing-helper.md"),
      before = requests;
    const result = await execute(selected, reportArgs, env(summary, "success", "7"));
    assert.notEqual(result.code, 0);
    assert.match(result.stderr, /MODULE_NOT_FOUND/);
    assert.equal(existsSync(summary), false);
    assert.equal(requests, before);
  });
});
