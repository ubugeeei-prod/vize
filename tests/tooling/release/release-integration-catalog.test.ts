import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  assertPinnedSource,
  integrationEvent,
} from "../../../tools/support/compat/github/release-integration-event.ts";
import { aggregateNeedsResults } from "../../../tools/support/compat/github/require-needs-success.ts";
import { readRepoFile } from "../support/github-workflows.ts";

const base = "b".repeat(40);
const candidate = "c".repeat(40);
const repository = "owner/repo";
const pull = {
  number: 99,
  head: { ref: "release-integration/v1.2.3" },
  body: "<!-- vize-release-pin-source: 42 -->",
};
const event = {
  repository: { full_name: repository },
  number: 99,
  pull_request: pull,
  merge_group: {
    head_ref: `refs/heads/gh-readonly-queue/main/pr-99-${base}`,
    base_ref: "refs/heads/main",
    head_sha: candidate,
    base_sha: base,
  },
};

test("integration planning selects its own PR and queue root while ordinary PRs need no catalog", () => {
  assert.equal(integrationEvent("pull_request", event, repository, candidate), "99");
  assert.equal(
    integrationEvent("merge_group", event, repository, candidate, (number: number) => {
      assert.equal(number, 99);
      return pull;
    }),
    "99",
  );
  const ordinary = { ...pull, head: { ref: "fix/ordinary" }, body: "ordinary" };
  assert.equal(
    integrationEvent("pull_request", { ...event, pull_request: ordinary }, repository, candidate),
    "",
  );
  assert.equal(
    integrationEvent("merge_group", event, repository, candidate, () => ordinary),
    "",
  );
  const source = {
    ...pull,
    draft: true,
    head: { ref: "release/v1.2.3", sha: candidate, repo: { full_name: repository } },
    base: { ref: "main", repo: { full_name: repository } },
    body: `<!-- vize-release-pin: immutable-v1 -->\n<!-- vize-release-pin-head: ${candidate} -->\n<!-- vize-release-pin-cut: ${base} -->`,
  };
  assert.equal(
    integrationEvent("pull_request", { ...event, pull_request: source }, repository, candidate),
    "",
  );
});

test("ordinary batched queue roots succeed while official integrations reject ambiguous bases", () => {
  const batched = { ...event, merge_group: { ...event.merge_group, base_sha: "d".repeat(40) } };
  const ordinary = { ...pull, head: { ref: "fix/ordinary" }, body: "ordinary" };
  assert.equal(
    integrationEvent("merge_group", batched, repository, candidate, () => ordinary),
    "",
  );
  assert.throws(() => integrationEvent("merge_group", batched, repository, candidate, () => pull));
  assert.throws(() =>
    integrationEvent(
      "merge_group",
      { ...batched, merge_group: { ...batched.merge_group, base_sha: "short" } },
      repository,
      candidate,
      () => ordinary,
    ),
  );
});

test("integration planning rejects missing, foreign and ambiguous event identities", () => {
  for (const mutated of [
    { ...event, repository: { full_name: "foreign/repo" } },
    { ...event, number: 98 },
    { ...event, pull_request: { ...pull, head: { ref: "fix/ordinary" } } },
  ]) {
    assert.throws(() => integrationEvent("pull_request", mutated, repository, candidate));
  }
  for (const changes of [
    { head_ref: `refs/heads/gh-readonly-queue/main/pr-98-${base}` },
    { head_ref: `refs/heads/gh-readonly-queue/main/pr-99-${candidate}` },
    { head_ref: "refs/heads/other" },
    { head_sha: base },
    { base_sha: candidate },
    { base_sha: "short" },
    { base_ref: "refs/heads/other" },
  ]) {
    assert.throws(() =>
      integrationEvent(
        "merge_group",
        { ...event, merge_group: { ...event.merge_group, ...changes } },
        repository,
        candidate,
        () => pull,
      ),
    );
  }
  assert.throws(() => integrationEvent("push", event, repository, candidate));
  assert.throws(() => integrationEvent("pull_request", event, repository, "short"));
});

test("required Check report consumes the always-running catalog gate without allowing skips", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "check.yml"));
  const wrapper = workflow.jobs["release-integration-catalog"];
  const catalogWorkflow = parse(
    readRepoFile(".github", "workflows", "release-integration-catalog.yml"),
  );
  const gate = catalogWorkflow.jobs["release-integration-catalog"];
  assert.equal(wrapper.uses, "./.github/workflows/release-integration-catalog.yml");
  assert.equal(wrapper.if, gate.if);
  assert.deepEqual(wrapper.permissions, gate.permissions);
  assert.deepEqual(catalogWorkflow.on, { workflow_call: null });
  assert.equal(
    gate.if,
    "${{ github.event_name == 'pull_request' || github.event_name == 'merge_group' }}",
  );
  assert.deepEqual(gate.permissions, { contents: "read", "pull-requests": "read" });
  assert.ok(workflow.jobs["test-report"].needs.includes("release-integration-catalog"));
  const named = (name: string) => {
    const matches = gate.steps.filter((step: { name?: string }) => step.name === name);
    assert.equal(matches.length, 1, `one ${name} step`);
    return matches[0];
  };
  const select = named("Select the official integration candidate");
  const history = named("Fetch exact integration history");
  const authentication = named(
    "Authenticate metadata and the complete pinned catalog before merge",
  );
  const snapshot = named("Retain complete integration event and current body snapshots");
  const initialCheckout = gate.steps.find(
    (step: { uses?: string; if?: string }) =>
      step.uses === "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd" && !step.if,
  );
  assert.ok(initialCheckout);
  assert.equal(select.id, "candidate");
  assert.equal(select.run, "node tools/support/compat/github/release-integration-event.ts");
  for (const step of gate.steps.filter(
    (step: object) =>
      ![initialCheckout, select, snapshot].includes(step) &&
      !("uses" in step && String(step.uses).startsWith("actions/setup-node@")),
  )) {
    assert.equal(step.if, "${{ steps.candidate.outputs.integration != '' }}");
  }
  assert.equal(history.with["fetch-depth"], 0);
  assert.equal(
    authentication.run,
    'rust-script tools/commands/release/pr.rs check-integration-candidate "$INTEGRATION_PR"',
  );
  for (const result of ["failure", "cancelled", "skipped"]) {
    assert.equal(aggregateNeedsResults({ "release-integration-catalog": { result } }).exitCode, 1);
  }
  assert.equal(
    aggregateNeedsResults({ "release-integration-catalog": { result: "success" } }).exitCode,
    0,
  );
});

test("integration body snapshots survive qualification refusal without running for ordinary PRs", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "release-integration-catalog.yml"));
  const gate = workflow.jobs["release-integration-catalog"];
  const authentication = gate.steps.find(
    (step: { name?: string }) =>
      step.name === "Authenticate metadata and the complete pinned catalog before merge",
  );
  const snapshot = gate.steps.find(
    (step: { name?: string }) =>
      step.name === "Retain complete integration event and current body snapshots",
  );
  assert.ok(gate.steps.indexOf(authentication) < gate.steps.indexOf(snapshot));
  assert.equal(authentication["continue-on-error"], undefined);
  assert.equal(snapshot.if, "${{ always() && steps.candidate.outputs.integration != '' }}");
  assert.equal(snapshot.uses, "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
  assert.deepEqual(snapshot.with, {
    name: "release-integration-body-${{ github.run_id }}-${{ github.run_attempt }}",
    path: "${{ runner.temp }}/vize-release-integration-catalog/",
    "if-no-files-found": "error",
    "retention-days": 30,
  });
  assert.equal(gate["timeout-minutes"], 10);
  assert.deepEqual(gate.permissions, { contents: "read", "pull-requests": "read" });
});

test("pinned source detects main merges, changed custody and malformed markers before qualification", () => {
  const source = {
    number: 9196,
    draft: true,
    merged: false,
    head: { ref: "release/v0.457.0", sha: candidate, repo: { full_name: repository } },
    base: { ref: "main", repo: { full_name: repository } },
    body: `<!-- vize-release-pin: immutable-v1 -->\n<!-- vize-release-pin-head: ${candidate} -->\n<!-- vize-release-pin-cut: ${base} -->`,
  };
  assert.doesNotThrow(() => assertPinnedSource(source, repository));
  for (const changed of [
    { ...source, head: { ...source.head, sha: "d".repeat(40) } },
    { ...source, head: { ...source.head, ref: "fix/ordinary" } },
    { ...source, head: { ...source.head, repo: { full_name: "foreign/repo" } } },
    { ...source, base: { ...source.base, ref: "davinci" } },
    { ...source, base: { ...source.base, repo: { full_name: "foreign/repo" } } },
    { ...source, draft: false },
    { ...source, merged: true },
    { ...source, body: source.body.replace("immutable-v1", "immutable-v2") },
    { ...source, body: source.body.replace(candidate, "short") },
    { ...source, body: source.body.replace(base, candidate) },
    { ...source, body: source.body.replace(`<!-- vize-release-pin-head: ${candidate} -->`, "") },
    { ...source, body: source.body + `\n<!-- vize-release-pin-head: ${candidate} -->` },
  ]) {
    assert.throws(() => assertPinnedSource(changed, repository), /Pinned source/);
  }
  const drift = { ...source, head: { ...source.head, sha: "d".repeat(40) } };
  assert.throws(
    () =>
      integrationEvent(
        "pull_request",
        {
          ...event,
          number: source.number,
          pull_request: drift,
        },
        repository,
        candidate,
      ),
    /Pinned source/,
  );
  assert.throws(
    () =>
      integrationEvent("merge_group", event, repository, candidate, () => ({
        ...drift,
        number: event.number,
      })),
    /Pinned source/,
  );
  assert.doesNotThrow(() =>
    assertPinnedSource(
      {
        ...source,
        body: "Legacy non-pinned release",
        draft: false,
      },
      repository,
    ),
  );
});
