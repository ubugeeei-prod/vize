// npm-first: a refused rolling crate budget defers crates.io alone.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { compileFunction } from "node:vm";
import { parse } from "yaml";
import { documentationReleaseNote } from "../../../docs/scripts/generate-contract-metadata.ts";

import {
  cratePublicationDecision,
  cratePublicationNote,
  cratePublicationSummary,
  quotaCommandStatus,
  recordCratePublication,
  type QuotaCommand,
} from "../../../tools/support/release/release/crate_publication.ts";
import {
  CRATE_QUOTA_BUDGET,
  readRollingCrateQuota,
} from "../../../tools/support/release/release/quota.ts";
import { readRepoFile } from "../support/github-workflows.ts";
import { writeFakeGh } from "./release-notes-fixtures.ts";

interface Step {
  id?: string;
  name?: string;
  run?: string;
  env?: Record<string, string>;
}
interface Job {
  if?: string;
  needs?: string | string[];
  outputs?: Record<string, string>;
  environment?: string;
  permissions?: Record<string, string>;
  steps?: Step[];
}
const release = parse(readRepoFile(".github", "workflows", "release.yml")) as {
  jobs: Record<string, Job>;
};

const now = Date.parse("2026-11-03T04:40:56Z");
const previous = "0.657.0";
const target = "0.658.0";
const json = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), { status, headers: { Date: new Date(now).toUTCString() } });

/** `recent` rows inside the window; `withPrevious` false leaves a crates.io gap. */
function registry(recent: number, withPrevious = true, previousStatus = 404): typeof fetch {
  return (async (input: string | URL | Request) => {
    const url = new URL(input instanceof Request ? input.url : input);
    const crate = url.pathname.split("/")[4]!;
    if (!url.pathname.endsWith("/versions")) {
      return previousStatus === 404
        ? json({ errors: [{ detail: "crate version does not exist" }] }, 404)
        : json({ version: {} });
    }
    const versions = Array.from({ length: recent }, (_, i) => ({
      id: i + 1,
      crate,
      num: withPrevious && i === 0 ? previous : `0.600.${i}`,
      created_at: new Date(now - i * 60 * 60_000).toISOString(),
      yanked: false,
      checksum: "a".repeat(64),
    }));
    return json({ versions, meta: { total: versions.length, next_page: null } });
  }) as typeof fetch;
}
const read = (fetchImpl: typeof fetch) =>
  readRollingCrateQuota(["alpha", "beta"], previous, target, null, fetchImpl, () => now);

test("an admitted budget publishes crates; the 41578199563 refusal defers only crates.io", async () => {
  const allowed = await read(registry(CRATE_QUOTA_BUDGET - 1));
  assert.equal(allowed.kind, "allowed");
  assert.deepEqual(cratePublicationDecision(allowed), {
    crates: "publish",
    reason: "the rolling crate quota admits every crate write",
  });

  // Run 41578199563: 19 versions in the window, so the 20th was refused.
  const blocked = await read(registry(CRATE_QUOTA_BUDGET));
  assert.equal(blocked.kind, "blocked");
  assert.ok(blocked.observations.every((row) => !row.allowed && row.proposedCount === 20));
  const deferred = cratePublicationDecision(blocked);
  assert.equal(deferred.crates, "defer");
  assert.equal(
    deferred.reason,
    `crates.io publication deferred by the rolling quota (next eligible ${blocked.nextEligibleAt})`,
  );

  const unavailable = await read((async () => json("rate limited", 429)) as typeof fetch);
  assert.equal(unavailable.kind, "unavailable");
  assert.deepEqual(cratePublicationDecision(unavailable), {
    crates: "defer",
    reason: "crates.io publication deferred: the rolling crate quota could not be established",
  });
});

test("a deferred prior version leaves a gap the next admission accepts only on a fresh 404", async () => {
  const gap = await read(registry(3, false));
  assert.equal(gap.kind, "allowed");
  assert.ok(gap.observations.every((row) => row.priorMissing?.status === 404));
  assert.equal(cratePublicationDecision(gap).crates, "publish");
  // Negative control: an exact-version hit missing from history is inconsistent.
  const inconsistent = await read(registry(3, false, 200));
  assert.equal(inconsistent.kind, "unavailable");
  assert.equal(cratePublicationDecision(inconsistent).crates, "defer");
});

test("only the pre-write guard fails on a refusal; the admission records it", () => {
  const exits = (command: QuotaCommand) =>
    (["allowed", "blocked", "unavailable"] as const).map((kind) =>
      quotaCommandStatus(command, kind),
    );
  assert.deepEqual(exits("quota"), [0, 1, 1], "crate writes never pass a refused budget");
  assert.deepEqual(exits("quota-publication"), [0, 0, 0], "npm never waits for crates.io");
  assert.deepEqual(exits("quota-dry-run"), [0, 0, 1]);
});

test("the admission writes job outputs and a visible step summary line", () => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-crate-deferral-"));
  const saved = { output: process.env.GITHUB_OUTPUT, summary: process.env.GITHUB_STEP_SUMMARY };
  const output = path.join(directory, "output");
  const summary = path.join(directory, "summary");
  process.env.GITHUB_OUTPUT = output;
  process.env.GITHUB_STEP_SUMMARY = summary;
  const deferred = cratePublicationDecision({
    kind: "blocked",
    nextEligibleAt: "2026-11-03T04:51:33.504Z",
  });
  try {
    recordCratePublication(
      target,
      cratePublicationDecision({ kind: "allowed", nextEligibleAt: null }),
    );
    recordCratePublication(target, deferred);
    assert.equal(
      readFileSync(output, "utf8"),
      "crates=publish\ncrates_reason=the rolling crate quota admits every crate write\n" +
        `crates=defer\ncrates_reason=${deferred.reason}\n`,
    );
    assert.equal(
      readFileSync(summary, "utf8"),
      `- crates.io: publishing v${target}\n` +
        "- crates.io: crates.io publication deferred by the rolling quota (next eligible " +
        `2026-11-03T04:51:33.504Z); every other channel still publishes v${target}\n`,
    );
  } finally {
    for (const [key, value] of [
      ["GITHUB_OUTPUT", saved.output],
      ["GITHUB_STEP_SUMMARY", saved.summary],
    ] as const) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
    rmSync(directory, { recursive: true, force: true });
  }
  assert.equal(
    cratePublicationSummary(target, { crates: "publish", reason: "" }),
    `crates.io: publishing v${target}`,
  );
});

test("release notes state a deferral and stay unchanged when crates publish", () => {
  const reason = cratePublicationDecision({
    kind: "blocked",
    nextEligibleAt: "2026-11-03T04:51:33.504Z",
  }).reason;
  assert.equal(cratePublicationNote(`v${target}`, "publish", ""), "");
  assert.equal(
    cratePublicationNote(`v${target}`, "defer", reason),
    `> [!NOTE]\n> ${reason}. Every other channel publishes v${target}; crates.io skips this ` +
      "version, and a later release publishes the crates at its own version.",
  );
  assert.match(
    cratePublicationNote(`v${target}`, "defer", reason),
    /crates\.io publication deferred by the rolling quota/,
  );
  for (const [crates, why] of [
    [undefined, undefined],
    ["", ""],
    ["skip", reason],
    ["defer", undefined],
    ["defer", "anything else"],
    ["defer", `${reason}\n## injected`],
  ] as const)
    assert.throws(() => cratePublicationNote(`v${target}`, crates, why), /crate publication/);
  const notes = release.jobs["github-release"]!.steps!.find(
    (s) => s.name === "Generate release notes",
  );
  assert.equal(notes?.env?.CRATES_PUBLICATION, "${{ needs.publication-quota.outputs.crates }}");
  assert.equal(
    notes?.env?.CRATES_PUBLICATION_REASON,
    "${{ needs.publication-quota.outputs.crates_reason }}",
  );
  const generator = readRepoFile("tools", "support", "release", "github_release_notes.ts");
  assert.ok(
    generator.indexOf("cratePublicationNote(") < generator.indexOf("generateReleaseNotes(api"),
  );
});

test("the real notes command prints the deferral and refuses an unknown decision before any API call", () => {
  const root = mkdtempSync(path.join(tmpdir(), "vize-crate-deferral-notes-"));
  const script = new URL("../../../tools/support/release/github_release_notes.ts", import.meta.url);
  try {
    const git = (...args: string[]) =>
      execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
    git("init", "--quiet");
    git(
      "-c",
      "user.name=F",
      "-c",
      "user.email=f@example.invalid",
      "commit",
      "--allow-empty",
      "-qm",
      "s",
    );
    const sha = git("rev-parse", "HEAD");
    const generated =
      "## What's Changed\n\n* fix by @user in https://github.com/owner/repo/pull/1\n";
    writeFakeGh(root, generated, { commits: [] });
    const output = path.join(root, "notes.md");
    const reason = cratePublicationDecision({
      kind: "blocked",
      nextEligibleAt: "2026-11-03T04:51:33.504Z",
    }).reason;
    const notes = (crates?: string) =>
      spawnSync(process.execPath, [script.pathname, "v0.458.0", sha, output], {
        cwd: root,
        encoding: "utf8",
        env: {
          ...process.env,
          CRATES_PUBLICATION: crates,
          CRATES_PUBLICATION_REASON: reason,
          GITHUB_REPOSITORY: "owner/repo",
          PATH: `${root}:${process.env.PATH ?? ""}`,
        },
      });
    const scope = readRepoFile(".github", "release-notes", "drop-in-scope.md").trim();
    const deferred = notes("defer");
    assert.equal(deferred.status, 0, deferred.stderr);
    assert.equal(
      readFileSync(output, "utf8"),
      `${scope}\n\n${cratePublicationNote("v0.458.0", "defer", reason)}\n\n${generated.trim()}\n\n${documentationReleaseNote()}\n`,
    );
    rmSync(output);
    rmSync(path.join(root, "api.jsonl"));
    const unknown = notes(undefined);
    assert.equal(unknown.status, 1);
    assert.match(unknown.stderr, /crate publication decision must be publish or defer/);
    assert.equal(existsSync(path.join(root, "api.jsonl")), false, "no GitHub call");
    assert.equal(existsSync(output), false, "no notes written");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

/** Evaluates a job's `if`, adding GitHub's implicit success() when it names no status function. */
function runs(id: string, crates: string, resume = ""): boolean {
  const job = release.jobs[id]!;
  const needs: Record<string, { result: string; outputs: Record<string, string> }> = {};
  for (const dependency of [job.needs ?? []].flat())
    needs[dependency] = {
      result: "success",
      outputs:
        dependency === "publication-quota" ? { crates } : { ref: "refs/release-candidates/source" },
    };
  // Every need succeeded here, so an implicit success() always holds.
  const condition = job.if ?? "success()";
  const expression = condition
    .replace(/^\$\{\{\s*|\s*\}\}$/g, "")
    .replaceAll("always()", "true")
    .replaceAll("success()", "true")
    .replaceAll("cancelled()", "false")
    .replace(/needs\.([a-z-]+)\./g, (_, key: string) => `needs["${key}"].`);
  return Boolean(
    compileFunction("return (" + expression + ");", ["needs", "inputs"])(needs, {
      npm_resume_run: resume,
    }),
  );
}

test("a deferral skips only the crate publisher; every other channel and the release proceed", () => {
  const admission = release.jobs["publication-quota"]!;
  const step = admission.steps!.find((s) => s.id === "admission");
  assert.match(step?.run ?? "", /release\/main\.ts quota-publication "\$RELEASE_VERSION"$/);
  assert.deepEqual(admission.outputs, {
    crates: "${{ steps.admission.outputs.crates }}",
    crates_reason: "${{ steps.admission.outputs.crates_reason }}",
  });
  assert.equal(admission.permissions?.["id-token"], undefined);
  const downstream = Object.keys(release.jobs).filter((id) =>
    [release.jobs[id]!.needs ?? []].flat().includes("publication-quota"),
  );
  assert.deepEqual(downstream.sort(), [
    "github-release",
    "publish-crates",
    "publish-npm",
    "publish-open-vsx",
    "publish-open-vsx-platforms",
    "publish-vscode",
    "publish-vscode-platforms",
  ]);
  for (const resume of ["", "41584652683"]) {
    for (const id of [...downstream, "dispatch-verify"]) {
      assert.equal(runs(id, "publish", resume), true, `${id} with crates admitted`);
      assert.equal(
        runs(id, "defer", resume),
        id !== "publish-crates",
        `${id} with crates deferred`,
      );
    }
    // Negative control: a missing decision never publishes crates.
    assert.equal(runs("publish-crates", "", resume), false);
  }
  for (const [id, job] of Object.entries(release.jobs))
    assert.ok(![job.needs ?? []].flat().includes("publish-crates"), `${id} never waits for crates`);
});

test("the crate publisher keeps its trusted identity and strict pre-write recheck", () => {
  const crates = release.jobs["publish-crates"]!;
  assert.equal(crates.environment, "crates-io");
  assert.deepEqual(crates.permissions, { contents: "read", actions: "read", "id-token": "write" });
  const steps = crates.steps!;
  const guard = steps.findIndex((s) => /main\.ts" quota "\$RELEASE_VERSION"$/m.test(s.run ?? ""));
  const publish = steps.findIndex((s) => s.run?.includes("publish_crates/main.ts"));
  assert.ok(guard >= 0 && guard < publish, "the strict guard runs before any crate write");
  const npm = release.jobs["publish-npm"]!;
  assert.equal(npm.environment, "npm");
  assert.match(JSON.stringify(npm.steps), /--provenance/);
  assert.equal(release.jobs["dispatch-verify"]!.environment, "release-operator");
});

test("the release-candidate cut never reads the crate quota; the admission decides once", () => {
  const prepare = JSON.stringify(release.jobs.version!.steps);
  assert.doesNotMatch(prepare, /main\.ts"? quota|RELEASE_QUOTA_EVIDENCE/);
  const main = readRepoFile("tools", "support", "release", "release", "main.ts");
  const body = (from: string, to: string) => main.slice(main.indexOf(from), main.indexOf(to));
  assert.doesNotMatch(body("async function cut(", "function finishCadence("), /releaseQuota\(/);
  assert.doesNotMatch(
    body("async function cadence(", "export async function release("),
    /releaseQuota\(/,
  );
  assert.equal(main.match(/recordCratePublication\(/g)?.length, 1);
  assert.match(main, /return quotaCommandStatus\(command, report\.kind\);/);
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  assert.equal(workflow.match(/quota-publication/g)?.length, 1, "one decision per release");
  assert.equal(workflow.match(/needs\.publication-quota\.outputs\.crates\b/g)?.length, 2);
});
