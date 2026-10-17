import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, rmSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";
import {
  requireDownload,
  selectArchive,
  validateArchive,
} from "../../tools/support/compat/github/rust-archive-download.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const context = { run: "123", sha: "a".repeat(40), repository: "owner/repo" };
const artifact = {
  id: 99,
  name: `rust-test-archive-123-${context.sha}`,
  digest: `sha256:${"b".repeat(64)}`,
  expired: false,
  workflow_run: { id: 123, head_sha: context.sha },
};

test("archive selection rejects ambiguous, expired, foreign and missing digest identities", () => {
  assert.deepEqual(selectArchive([artifact], context), { id: 99, digest: artifact.digest });
  assert.throws(() => selectArchive([], context));
  assert.throws(() => selectArchive([artifact, artifact], context));
  for (const changed of [
    null,
    {},
    { ...artifact, workflow_run: null },
    { ...artifact, id: 0 },
    { ...artifact, expired: true },
    { ...artifact, digest: "missing" },
    { ...artifact, name: "another-source" },
    { ...artifact, workflow_run: { id: 124, head_sha: context.sha } },
    { ...artifact, workflow_run: { id: 123, head_sha: "c".repeat(40) } },
  ])
    assert.throws(() => validateArchive(changed, context));
});

test("only a clean first success or a single successful retry can qualify", () => {
  for (const first of ["success", "failure", "cancelled", "skipped", "", undefined]) {
    for (const retry of ["success", "failure", "cancelled", "skipped", "", undefined]) {
      if (
        (first === "success" && retry === "skipped") ||
        (first === "failure" && retry === "success")
      )
        assert.doesNotThrow(() => requireDownload(first, retry));
      else assert.throws(() => requireDownload(first, retry));
    }
  }
});

test("workflow bounds both attempts, pins the same ID and requires cleanup plus every original gate", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "pr-rust-checks.yml"));
  const job = workflow.jobs["pr-rust-shard"];
  const first = job.steps.find((step: { id?: string }) => step.id === "rust-archive-first");
  const retry = job.steps.find((step: { id?: string }) => step.id === "rust-archive-retry");
  assert.equal(
    job.steps.filter((step: { uses?: string }) =>
      step.uses?.startsWith("actions/download-artifact@"),
    ).length,
    2,
  );
  for (const step of [first, retry]) {
    assert.equal(step["timeout-minutes"], 3);
    assert.deepEqual(step.with, {
      "artifact-ids": "${{ steps.rust-archive.outputs.id }}",
      path: ".artifacts/rust-test-archive",
      "digest-mismatch": "error",
    });
  }
  assert.equal(first["continue-on-error"], true);
  assert.equal(retry["continue-on-error"], undefined);
  assert.equal(first.if, "${{ github.event_name == 'merge_group' }}");
  assert.equal(
    retry.if,
    "${{ !cancelled() && steps.rust-archive-first.outcome == 'failure' && steps.rust-archive-clean.outcome == 'success' }}",
  );
  const clean = job.steps.find((step: { id?: string }) => step.id === "rust-archive-clean");
  assert.equal(clean.run, "node tools/support/compat/github/rust-archive-download.ts clean-retry");
  assert.equal(clean.if, "${{ !cancelled() && steps.rust-archive-first.outcome == 'failure' }}");
  assert.ok(job.steps.indexOf(clean) > job.steps.indexOf(first));
  assert.ok(job.steps.indexOf(retry) > job.steps.indexOf(clean));
  const required = job.steps.find(
    (step: { name?: string }) => step.name === "Require completed immutable archive download",
  );
  assert.equal(required.run, "node tools/support/compat/github/rust-archive-download.ts verify");
  assert.equal(required.if, "${{ !cancelled() && github.event_name == 'merge_group' }}");
  assert.equal(required["continue-on-error"], undefined);
  const receipt = job.steps.find(
    (step: { name?: string }) => step.name === "Verify Rust archive identity",
  );
  assert.equal(
    receipt.run,
    "node tools/support/compat/github/rust-test-archive.mjs verify .artifacts/rust-test-archive/tests.tar.zst .artifacts/rust-test-archive/receipt.json",
  );
  assert.ok(job.steps.indexOf(receipt) > job.steps.indexOf(required));
  const run = job.steps.find(
    (step: { name?: string }) => step.name === "Run Rust test cohort without rebuilding",
  );
  assert.ok(
    run.run.includes(
      'cargo nextest run --archive-file .artifacts/rust-test-archive/tests.tar.zst --extract-to "$GITHUB_WORKSPACE" --extract-overwrite --workspace-remap "$GITHUB_WORKSPACE" --profile "$NEXTEST_PROFILE" -E "$NEXTEST_FILTER" --no-tests=pass',
    ),
  );
  assert.equal(job.strategy["fail-fast"], false);
  assert.equal(job["timeout-minutes"], 20);
});

test("actual selector cleans partial files and rejects metadata changes after a successful transfer", async (t) => {
  const workspace = mkdtempSync(join(tmpdir(), "vize-bounded-archive-"));
  t.after(() => rmSync(workspace, { recursive: true, force: true }));
  let current = artifact;
  const server = createServer((request, response) => {
    assert.equal(request.headers.authorization, "Bearer unit-token");
    response.setHeader("content-type", "application/json");
    response.end(
      JSON.stringify(
        request.url?.includes("/runs/") ? { total_count: 1, artifacts: [current] } : current,
      ),
    );
  });
  await new Promise<void>((done) => server.listen(0, "127.0.0.1", done));
  t.after(() => server.close());
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  const env = {
    ...process.env,
    GITHUB_WORKSPACE: workspace,
    GITHUB_RUN_ID: context.run,
    GITHUB_SHA: context.sha,
    GITHUB_REPOSITORY: context.repository,
    GITHUB_API_URL: `http://127.0.0.1:${address.port}`,
    GH_TOKEN: "unit-token",
    GITHUB_OUTPUT: join(workspace, "outputs"),
    RUST_ARCHIVE_ID: "99",
    RUST_ARCHIVE_DIGEST: artifact.digest,
    RUST_ARCHIVE_FIRST_OUTCOME: "failure",
    RUST_ARCHIVE_RETRY_OUTCOME: "success",
  };
  const command = new URL(
    "../../tools/support/compat/github/rust-archive-download.ts",
    import.meta.url,
  );
  const run = (mode: string) =>
    promisify(execFile)(process.execPath, [fileURLToPath(command), mode], { env });
  const destination = join(workspace, ".artifacts/rust-test-archive");
  mkdirSync(destination, { recursive: true });
  writeFileSync(join(destination, "partial"), "incomplete archive");
  await run("select");
  assert.equal(existsSync(destination), false);
  assert.equal(readFileSync(env.GITHUB_OUTPUT, "utf8"), `id=99\ndigest=${artifact.digest}\n`);
  mkdirSync(destination);
  writeFileSync(join(destination, "partial"), "another incomplete archive");
  await run("clean-retry");
  assert.equal(existsSync(destination), false);
  await run("verify");
  current = { ...artifact, digest: `sha256:${"c".repeat(64)}` };
  await assert.rejects(run("verify"), /ID\/digest changed/);
});
