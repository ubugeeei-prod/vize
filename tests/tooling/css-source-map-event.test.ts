import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { resolveCssEventSource } from "../native/css-source-map-event.mjs";

type WorkflowStep = { uses?: string; name?: string };

await test("older actual PR heads without CSS targets qualify only their real current-base merge candidate", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-event-source-"));
  const git = (...argv: string[]) =>
    execFileSync("git", argv, {
      cwd: root,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  try {
    git("init", "-b", "main");
    git("config", "user.name", "Fixture");
    git("config", "user.email", "fixture@example.invalid");
    fs.writeFileSync(path.join(root, "existing"), "original\n");
    git("add", "existing");
    git("commit", "-m", "original");
    git("checkout", "-b", "old-pr");
    fs.writeFileSync(path.join(root, "art"), "owned old-head change\n");
    git("add", "art");
    git("commit", "-m", "old PR");
    const literalHead = git("rev-parse", "HEAD");
    git("checkout", "main");
    fs.writeFileSync(path.join(root, "css-target"), "actual newer CSS target\n");
    git("add", "css-target");
    git("commit", "-m", "new base target");
    git("merge", "--no-ff", "old-pr", "-m", "candidate");
    const sha = git("rev-parse", "HEAD"),
      parents = git("rev-list", "--parents", "-n", "1", sha).split(" ").slice(1);
    assert.equal(
      spawnSync("git", ["show", `${literalHead}:css-target`], { cwd: root }).status,
      128,
    );
    assert.equal(git("show", `${sha}:css-target`), "actual newer CSS target");
    const input = {
      event: "pull_request",
      sha,
      head: sha,
      ref: "refs/pull/1/merge",
      literalHead,
      parents,
    };
    assert.deepEqual(resolveCssEventSource(input), {
      role: "pull-request-merge",
      source: sha,
      literalPullRequestHead: literalHead,
      parents,
    });
    for (const changed of [
      { head: literalHead },
      { sha: literalHead, head: literalHead },
      { parents: [] },
      { literalHead: "f".repeat(40) },
      { ref: "refs/heads/old-pr" },
    ])
      assert.throws(() => resolveCssEventSource({ ...input, ...changed }));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
await test("dispatch qualifies exactly its branch/tag source and cannot fall back to another checkout or PR role", () => {
  const sha = "a".repeat(40),
    input = {
      event: "workflow_dispatch",
      sha,
      head: sha,
      ref: "refs/heads/pinned",
      literalHead: "",
      parents: ["b".repeat(40)],
    };
  assert.deepEqual(resolveCssEventSource(input), {
    role: "dispatch-source",
    source: sha,
    literalPullRequestHead: null,
    parents: input.parents,
  });
  assert.equal(resolveCssEventSource({ ...input, ref: "refs/tags/pinned" }).source, sha);
  for (const changed of [
    { head: "c".repeat(40) },
    { sha: "main" },
    { event: "push" },
    { ref: "refs/pull/1/merge" },
    { literalHead: sha },
  ])
    assert.throws(() => resolveCssEventSource({ ...input, ...changed }));
});
await test("the actual dedicated workflow checks event source and retains role proof before any CSS compilation", () => {
  const root = fileURLToPath(new URL("../../", import.meta.url)),
    { parse } = createRequire(import.meta.url)("yaml");
  const workflow = parse(
    fs.readFileSync(path.join(root, ".github/workflows/css-compile-source-maps.yml"), "utf8"),
  );
  assert.equal(workflow.env.VIZE_CSS_MAP_SOURCE_SHA, "${{ github.sha }}");
  assert.equal(
    workflow.env.VIZE_CSS_MAP_PR_HEAD,
    "${{ github.event.pull_request.head.sha || '' }}",
  );
  const steps = workflow.jobs.native.steps,
    checkout = steps.find((step: WorkflowStep) => step.uses?.startsWith("actions/checkout@"));
  assert.equal(checkout.with.ref, "${{ env.VIZE_CSS_MAP_SOURCE_SHA }}");
  assert.equal(checkout.with["fetch-depth"], 2);
  const proof = steps.find(
    (step: WorkflowStep) =>
      step.name === "Bind actual checkout to its PR merge or exact dispatch role",
  );
  assert.deepEqual(Object.keys(proof).sort(), ["name", "run"]);
  assert.equal(
    proof.run,
    "node --test tests/tooling/css-source-map-event.test.ts\nnode tests/native/css-source-map-event.mjs\n",
  );
  assert.ok(
    steps.indexOf(proof) <
      steps.findIndex(
        (step: WorkflowStep) =>
          step.name === "Verify real writer coordinates and complete existing engine contracts",
      ),
  );
  assert.equal(workflow.jobs.native.if, undefined);
  assert.equal(workflow.jobs.native["continue-on-error"], undefined);
  assert.equal(
    workflow.jobs.measurements.if,
    "${{ github.event_name == 'workflow_dispatch' && inputs.measurements }}",
  );
});

await test("a real failed source attempt replaces any retained success before checkout validation", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-event-refusal-"));
  try {
    const output = path.join(directory, "target/css-compile-source-maps/event-source.json");
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(output, JSON.stringify({ status: "qualified", source: "f".repeat(40) }));
    const child = spawnSync(
      process.execPath,
      [fileURLToPath(new URL("../native/css-source-map-event.mjs", import.meta.url))],
      {
        cwd: directory,
        encoding: "utf8",
        env: {
          ...process.env,
          GITHUB_EVENT_NAME: "workflow_dispatch",
          GITHUB_SHA: "a".repeat(40),
          GITHUB_REF: "refs/heads/pinned",
          VIZE_CSS_MAP_SOURCE_SHA: "a".repeat(40),
          VIZE_CSS_MAP_PR_HEAD: "",
        },
      },
    );
    assert.equal(child.status, 1);
    assert.deepEqual(JSON.parse(fs.readFileSync(output, "utf8")), {
      schema: "vize.css.source-map.event-source",
      version: 1,
      status: "attempt",
      event: "workflow_dispatch",
      expectedSource: "a".repeat(40),
      literalPullRequestHead: "",
    });
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
