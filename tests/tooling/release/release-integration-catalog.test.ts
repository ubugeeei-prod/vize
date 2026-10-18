import assert from "node:assert/strict";
import { test } from "node:test";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
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
  const source = parse(readRepoFile(".github/workflows/pr-source-checks.yml"));
  const wrapper = source.jobs["release-integration-catalog"];
  const catalogWorkflow = parse(
    readRepoFile(".github", "workflows", "release-integration-catalog.yml"),
  );
  const gate = catalogWorkflow.jobs["release-integration-catalog"];
  assert.equal(wrapper.uses, "./.github/workflows/release-integration-catalog.yml");
  assert.equal(wrapper.if, gate.if);
  assert.deepEqual(wrapper.permissions, gate.permissions);
  assert.deepEqual(Object.keys(catalogWorkflow.on.workflow_call.outputs).sort(), [
    "integration",
    "product-qualification",
    "qualification",
  ]);
  assert.equal(
    gate.if,
    "${{ github.event_name == 'pull_request' || github.event_name == 'merge_group' }}",
  );
  assert.deepEqual(gate.permissions, {
    contents: "read",
    "pull-requests": "read",
    actions: "read",
  });
  assert.ok(workflow.jobs["test-report"].needs.includes("pr-source-checks"));
  assert.ok(source.jobs["source-report"].needs.includes("release-integration-catalog"));
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
      !("id" in step && ["qualification", "products"].includes(String(step.id))) &&
      !("uses" in step && String(step.uses).startsWith("actions/setup-node@")),
  )) {
    assert.equal(step.if, "${{ steps.candidate.outputs.integration != '' }}");
  }
  assert.equal(history.with["fetch-depth"], 0);
  assert.ok(
    authentication.run.includes(
      'rust-script tools/commands/release/pr.rs check-integration-candidate "$INTEGRATION_PR" > "$directory/candidate-context.json"',
    ),
  );
  for (const result of ["failure", "cancelled", "skipped"]) {
    assert.equal(
      aggregateNeedsResults({
        "release-integration-catalog": { result },
        "ordinary-candidate": { result: "success" },
        "auxiliary-q-check": { result: "success" },
      }).exitCode,
      1,
    );
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
  assert.equal(gate["timeout-minutes"], 60);
  assert.deepEqual(gate.permissions, {
    contents: "read",
    "pull-requests": "read",
    actions: "read",
  });
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
  assert.doesNotThrow(() =>
    assertPinnedSource({ ...source, body: source.body.replaceAll("\n", "\r\n") }, repository),
  );
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

const mergeGroup = "${{ github.event_name == 'merge_group' }}";
const originalNeeds = [
  "iteration-plan",
  "fmt-rust",
  "check-js",
  "security-audit",
  "node-engine-compat",
  "check-vize-apps",
  "pr-source-checks",
  "instruction-counts",
  "level-dependency-direction",
];

function qualificationWorkflows() {
  return {
    check: parse(readRepoFile(".github", "workflows", "check.yml")),
    catalog: parse(readRepoFile(".github", "workflows", "release-integration-catalog.yml")),
    source: parse(readRepoFile(".github/workflows/pr-source-checks.yml")),
  };
}

function assertRequiredQualification({
  check,
  catalog,
  source,
}: ReturnType<typeof qualificationWorkflows>) {
  const wrapper = source.jobs["release-integration-catalog"];
  const gate = catalog.jobs["release-integration-catalog"];
  const report = check.jobs["test-report"];
  const permissions = { contents: "read", "pull-requests": "read", actions: "read" };
  for (const value of [
    wrapper.permissions,
    gate.permissions,
    catalog.permissions,
    report.permissions,
  ]) {
    assert.deepEqual(value, permissions);
  }
  assert.deepEqual(report.needs, originalNeeds);
  assert.equal(
    report.if,
    "${{ always() && (github.event_name == 'pull_request' || github.event_name == 'merge_group') }}",
  );
  assert.equal(gate["timeout-minutes"], 60);
  assert.deepEqual(gate.outputs, {
    integration: "${{ steps.candidate.outputs.integration }}",
    qualification: "${{ steps.qualification.outputs.qualification }}",
    "product-qualification": "${{ steps.products.outputs.product-qualification }}",
  });
  for (const name of ["integration", "qualification", "product-qualification"]) {
    assert.equal(
      catalog.on.workflow_call.outputs[name].value,
      `\${{ jobs.release-integration-catalog.outputs.${name} }}`,
    );
  }
  const auth = gate.steps.find((step: { name?: string }) =>
    step.name?.startsWith("Authenticate metadata"),
  );
  const wait = gate.steps.find((step: { id?: string }) => step.id === "qualification");
  assert.ok(gate.steps.indexOf(auth) < gate.steps.indexOf(wait));
  assert.equal(auth.if, "${{ steps.candidate.outputs.integration != '' }}");
  assert.equal(
    wait.if,
    "${{ github.event_name == 'merge_group' && steps.candidate.outputs.integration != '' }}",
  );
  assert.ok(
    auth.run.includes(
      'check-integration-candidate "$INTEGRATION_PR" > "$directory/candidate-context.json"',
    ),
  );
  assert.ok(
    wait.run.includes(
      'integration.ts --wait < "$directory/candidate-context.json" > "$directory/qualification-q.json"',
    ),
  );
  assert.ok(wait.run.includes("Buffer.byteLength(raw) <= 16384"));
  assert.ok(wait.run.includes("fs.appendFileSync(process.env.GITHUB_OUTPUT"));
  assert.equal(
    wait.env.VIZE_RELEASE_INTEGRATION_EVIDENCE,
    "${{ runner.temp }}/vize-release-integration-catalog/qualification-observations.jsonl",
  );
  const original = report.steps.find(
    (step: { name?: string }) => step.name === "Require every needed job to succeed",
  );
  const select = report.steps.find((step: { id?: string }) => step.id === "final-candidate");
  const verify = report.steps.find((step: { name?: string }) =>
    step.name?.startsWith("Freshly verify"),
  );
  assert.equal(
    original.run,
    "node tools/support/compat/github/require-needs-success.ts --check-report",
  );
  assert.equal(original.env.NEEDS_JSON, "${{ toJSON(needs) }}");
  assert.ok(report.steps.indexOf(original) < report.steps.indexOf(select));
  assert.ok(report.steps.indexOf(select) < report.steps.indexOf(verify));
  assert.equal(select.run, "node tools/support/compat/github/release-integration-event.ts");
  for (const step of [auth, wait, original, select, verify]) {
    assert.equal(step["continue-on-error"], undefined);
  }
  assert.equal(select.if, mergeGroup);
  assert.equal(verify.if, mergeGroup);
  assert.equal(verify.env.CURRENT_INTEGRATION, "${{ steps.final-candidate.outputs.integration }}");
  assert.equal(verify.env.CATALOG_INTEGRATION, "${{ needs.pr-source-checks.outputs.integration }}");
  assert.equal(
    verify.env.QUALIFICATION_JSON,
    "${{ needs.pr-source-checks.outputs.qualification }}",
  );
  assert.ok(verify.run.includes("assert.equal(selected, catalog"));
  assert.ok(verify.run.includes("assert.equal(raw, ''"));
  assert.ok(verify.run.includes("raw.length > 0 && Buffer.byteLength(raw) <= 16384"));
  assert.ok(
    verify.run.includes(
      'integration-release.ts --verify < "$directory/qualification.json" > "$directory/final-qualification.json"',
    ),
  );
  assert.equal(report.steps.at(-2), verify);
  const retained = report.steps.at(-1);
  assert.equal(retained.if, "${{ always() && github.event_name == 'merge_group' }}");
  assert.equal(retained.with.path, "${{ runner.temp }}/vize-release-integration-final/");
  assert.equal(retained.with["if-no-files-found"], "error");
  for (const step of [...gate.steps, ...report.steps]) {
    assert.ok(!String(step.uses).includes("download-artifact"));
    assert.ok(!/gh (?:workflow run|run rerun)|--dispatch|--rerun/.test(step.run ?? ""));
  }
}

test("required queue report freshly verifies the same-run exact Q after every original dependency", () => {
  assertRequiredQualification(qualificationWorkflows());
});

test("required Q wiring refuses advisory gates, missing context, stale receipt routes and write permissions", () => {
  const mutations = [
    (w: ReturnType<typeof qualificationWorkflows>) => w.check.jobs["test-report"].needs.pop(),
    (w: ReturnType<typeof qualificationWorkflows>) =>
      (w.check.jobs["test-report"].permissions.actions = "write"),
    (w: ReturnType<typeof qualificationWorkflows>) =>
      (w.source.jobs["release-integration-catalog"].permissions.actions = "write"),
    (w: ReturnType<typeof qualificationWorkflows>) =>
      (w.catalog.jobs["release-integration-catalog"].outputs.qualification =
        "${{ inputs.qualification }}"),
    (w: ReturnType<typeof qualificationWorkflows>) =>
      (w.catalog.jobs["release-integration-catalog"].steps.find(
        (s: { id?: string }) => s.id === "qualification",
      ).if = "${{ always() }}"),
    (w: ReturnType<typeof qualificationWorkflows>) =>
      (w.check.jobs["test-report"].steps.find(
        (s: { id?: string }) => s.id === "final-candidate",
      ).if = "${{ false }}"),
    (w: ReturnType<typeof qualificationWorkflows>) =>
      (w.check.jobs["test-report"].steps.at(-2)["continue-on-error"] = true),
    (w: ReturnType<typeof qualificationWorkflows>) =>
      (w.check.jobs["test-report"].steps.at(-2).env.QUALIFICATION_JSON =
        "${{ inputs.qualification }}"),
    (w: ReturnType<typeof qualificationWorkflows>) =>
      (w.check.jobs["test-report"].steps.at(-2).run = 'echo "previous Q was green"'),
    (w: ReturnType<typeof qualificationWorkflows>) =>
      w.check.jobs["test-report"].steps.push({
        uses: "actions/download-artifact@any",
        with: { name: "qualification" },
      }),
  ];
  for (const mutate of mutations) {
    const workflows = qualificationWorkflows();
    mutate(workflows);
    assert.throws(() => assertRequiredQualification(workflows));
  }
});

test("the actual final context guard accepts ordinary emptiness and refuses lost official authority", () => {
  const workflow = qualificationWorkflows();
  const verify = workflow.check.jobs["test-report"].steps.at(-2);
  const guard = verify.run.split("<<'NODE'\n")[1].split("\nNODE")[0];
  const directory = mkdtempSync(join(tmpdir(), "vize-required-q-context-"));
  const run = (selected: string, catalog: string, receipt: string) =>
    execFileSync(process.execPath, ["--input-type=module"], {
      input: guard,
      env: {
        ...process.env,
        RUNNER_TEMP: directory,
        CURRENT_INTEGRATION: selected,
        CATALOG_INTEGRATION: catalog,
        QUALIFICATION_JSON: receipt,
      },
      stdio: "pipe",
    });
  try {
    // Production creates this directory before executing the inline guard.
    mkdirSync(join(directory, "vize-release-integration-final"));
    run("", "", "");
    for (const values of [
      ["99", "", "{}"],
      ["", "99", "{}"],
      ["99", "98", "{}"],
      ["99", "99", ""],
      ["", "", "{}"],
      ["99", "99", "null"],
      ["99", "99", "[]"],
      ["99", "99", "{"],
      ["99", "99", " ".repeat(16385)],
    ])
      assert.throws(() => run(...(values as [string, string, string])));
    // This is transport only: the mandatory --verify authenticates the receipt afterward.
    run("99", "99", '{"qualified":true}');
    assert.equal(
      readFileSync(join(directory, "vize-release-integration-final/qualification.json"), "utf8"),
      '{"qualified":true}',
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
