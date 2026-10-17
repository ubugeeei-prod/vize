import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { comparisonBase } from "../../tools/support/compat/github/comparison-base.ts";
import { changedPaths } from "../../tools/support/compat/github/plan-source-checks.ts";

const eventBase = "1".repeat(40);
const newBase = "2".repeat(40);
const prHead = "3".repeat(40);
const head = "4".repeat(40);
const input = {
  event: "pull_request",
  eventBase,
  prHead,
  head,
  checkedOutHead: head,
  commit: `tree ${"5".repeat(40)}\nparent ${newBase}\nparent ${prHead}\n\nMerge candidate`,
};

void test("only the verified two-parent tested merge replaces a stale event base", () => {
  assert.deepEqual(comparisonBase(input), {
    base: newBase,
    reason: "verified pull request merge parent",
  });
  for (const candidate of [
    { ...input, checkedOutHead: newBase },
    { ...input, prHead: newBase },
    { ...input, commit: `tree ${newBase}\n\nNonmerge` },
    { ...input, commit: `parent ${newBase}\n\nSingle parent` },
    { ...input, commit: `parent ${newBase}\nparent ${prHead}\nparent ${head}\n\nOctopus` },
    { ...input, commit: `parent malformed\nparent ${prHead}\n\nInvalid header` },
  ]) {
    assert.deepEqual(comparisonBase(candidate), {
      base: eventBase,
      reason: "unverified merge: conservative event base",
    });
  }
  assert.deepEqual(comparisonBase({ ...input, event: "merge_group", prHead: undefined }), {
    base: eventBase,
    reason: "merge group event base",
  });
});

void test("comparison identity rejects incomplete SHAs and unrecognized planning contexts", () => {
  for (const field of ["eventBase", "head", "checkedOutHead", "prHead"]) {
    assert.throws(() => comparisonBase({ ...input, [field]: "short" }), /Expected full/);
    assert.throws(() => comparisonBase({ ...input, [field]: undefined }), /Expected full/);
  }
  assert.throws(() => comparisonBase({ ...input, event: "push" }), /Unexpected comparison/);
});

void test("the real comparison step reuses present parents and fetches only missing validated bases", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-comparison-base-"));
  const origin = join(directory, "origin");
  const git = (cwd: string, ...args: string[]) =>
    execFileSync("git", args, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
  try {
    const workflow = parse(readFileSync(resolve(".github/workflows/pr-source-checks.yml"), "utf8"));
    const steps = workflow.jobs["pr-source-plan"].steps;
    assert.equal(
      steps.find((step: { uses?: string }) => step.uses?.startsWith("actions/checkout@"))?.with?.[
        "fetch-depth"
      ],
      2,
    );
    const command = steps.find((step: { id?: string }) => step.id === "comparison")?.run;
    assert.equal(typeof command, "string");
    git(directory, "init", "-q", "-b", "main", origin);
    git(origin, "config", "user.name", "CI Test");
    git(origin, "config", "user.email", "ci@example.invalid");
    const script = "tools/support/compat/github/comparison-base.ts";
    mkdirSync(dirname(join(origin, script)), { recursive: true });
    copyFileSync(resolve(script), join(origin, script));
    writeFileSync(join(origin, "base.md"), "base\n");
    git(origin, "add", ".");
    git(origin, "commit", "-qm", "base");
    const staleBase = git(origin, "rev-parse", "HEAD");
    git(origin, "checkout", "-qb", "feature");
    writeFileSync(join(origin, "feature.mjs"), "export const selected = true;\n");
    git(origin, "add", ".");
    git(origin, "commit", "-qm", "feature");
    const feature = git(origin, "rev-parse", "HEAD");
    git(origin, "checkout", "-q", "main");
    writeFileSync(join(origin, "release.json"), '{"version":"0.429.1"}\n');
    git(origin, "add", ".");
    git(origin, "commit", "-qm", "release advanced after event payload");
    const testedBase = git(origin, "rev-parse", "HEAD");
    git(origin, "merge", "--no-ff", "-qm", "tested merge", "feature");
    const testedMerge = git(origin, "rev-parse", "HEAD");
    const cases = [
      {
        name: "present-parent",
        depth: 2,
        event: "pull_request",
        prHead: feature,
        eventBase: staleBase,
        base: testedBase,
        fetch: false,
      },
      {
        name: "missing-parent",
        depth: 1,
        event: "pull_request",
        prHead: feature,
        eventBase: staleBase,
        base: testedBase,
        fetch: true,
      },
      {
        name: "unverified-merge",
        depth: 2,
        event: "pull_request",
        prHead: staleBase,
        eventBase: staleBase,
        base: staleBase,
        fetch: true,
      },
      {
        name: "queue-event-base",
        depth: 2,
        event: "merge_group",
        eventBase: staleBase,
        base: staleBase,
        fetch: true,
      },
      {
        name: "zero-base",
        depth: 2,
        event: "merge_group",
        eventBase: "0".repeat(40),
        base: "0".repeat(40),
        fetch: false,
      },
      {
        name: "invalid-base",
        depth: 2,
        event: "merge_group",
        eventBase: "short",
        fetch: false,
        failure: true,
      },
      {
        name: "unfetchable-base",
        depth: 2,
        event: "merge_group",
        eventBase,
        base: eventBase,
        fetch: true,
        failure: true,
      },
    ];
    for (const entry of cases) {
      const shallow = join(directory, entry.name);
      git(directory, "clone", "-q", `--depth=${entry.depth}`, `file://${origin}`, shallow);
      if (entry.base && !/^0+$/.test(entry.base)) {
        const present = spawnSync("git", ["cat-file", "-e", `${entry.base}^{commit}`], {
          cwd: shallow,
        });
        assert.equal(present.status === 0, !entry.fetch, entry.name);
      }
      const output = join(directory, `${entry.name}.outputs`);
      const trace = join(directory, `${entry.name}.trace`);
      const result = spawnSync(
        "/bin/bash",
        ["--noprofile", "--norc", "-eo", "pipefail", "-c", command],
        {
          cwd: shallow,
          encoding: "utf8",
          env: {
            ...process.env,
            PATH: `${dirname(process.execPath)}:${process.env.PATH ?? ""}`,
            GIT_TRACE: trace,
            GITHUB_EVENT_NAME: entry.event,
            EVENT_BASE_SHA: entry.eventBase,
            GITHUB_SHA: testedMerge,
            PR_HEAD_SHA: entry.prHead,
            GITHUB_OUTPUT: output,
            GITHUB_STEP_SUMMARY: join(directory, `${entry.name}.summary`),
          },
        },
      );
      assert.equal(result.status === 0, !entry.failure, `${entry.name}: ${result.stderr}`);
      assert.equal(
        (readFileSync(trace, "utf8").match(/built-in: git fetch /g) ?? []).length,
        entry.fetch ? 1 : 0,
        entry.name,
      );
      if (entry.failure) continue;
      assert.equal(readFileSync(output, "utf8"), `base=${entry.base}\n`, entry.name);
      if (!/^0+$/.test(entry.base!)) {
        assert.deepEqual(
          changedPaths(entry.base!, testedMerge, shallow),
          entry.base === testedBase ? ["feature.mjs"] : ["feature.mjs", "release.json"],
          entry.name,
        );
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
