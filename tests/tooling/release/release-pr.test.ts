import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { repoRoot, runReleaseCommand } from "../support/release-command.ts";
import { writeFakeCommand } from "../support/fake-command.ts";
import { readRepoFile } from "../support/github-workflows.ts";

const script = path.join(repoRoot, "tools/commands/release/pr.rs");

test("public release CLI forwards pinned start, resume and retirement through its Rust protocol", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-pinned-cli-"));
  const bin = path.join(root, "bin");
  const capture = path.join(root, "capture.json");
  fs.mkdirSync(bin);
  writeFakeCommand(
    bin,
    "rust-script",
    `require('node:fs').writeFileSync(${JSON.stringify(capture)}, JSON.stringify(process.argv.slice(2)));`,
  );
  try {
    const retire = [
      "--retire",
      "42",
      "--head",
      "a".repeat(40),
      "--tag",
      "v0.440.0",
      "--run",
      "43",
      "--operator-run",
      "44",
    ];
    const cases = [
      { args: ["--resume", "42"], protocol: ["resume", "42"] },
      { args: ["--resume", "42", "--pin"], protocol: ["resume", "42", "--pin"] },
      { args: ["patch", "-y", "--pin"], protocol: ["start", "patch", "--pin"] },
      { args: retire, protocol: ["retire", "42", "a".repeat(40), "v0.440.0", "43", "44"] },
    ];
    for (const { args, protocol } of cases) {
      const result = runReleaseCommand("release", args, {
        env: { PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}` },
      });
      assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
      assert.deepEqual(JSON.parse(fs.readFileSync(capture, "utf8")), [
        "tools/commands/release/pr.rs",
        ...protocol,
      ]);
    }
    fs.rmSync(capture);
    const rejected = runReleaseCommand("release", ["patch", "-y", "--pin", "--prepare-only"]);
    assert.notEqual(rejected.status, 0);
    assert.match(rejected.stderr, /--pin requires the release PR protocol/);
    assert.equal(fs.existsSync(capture), false);
    const invalid = [
      retire.slice(0, -1),
      [...retire, "--pin"],
      [...retire, "-y"],
      ["--retire", "42"],
      ...[1, 7, 9].flatMap((index) =>
        ["0", "-1", "042", "42; touch injected"].map((value) =>
          retire.map((arg, position) => (position === index ? value : arg)),
        ),
      ),
      ...["", "main", "a".repeat(39), "a".repeat(41), "g".repeat(40)].map((head) =>
        retire.map((arg, index) => (index === 3 ? head : arg)),
      ),
      ...["v1.440.0", "v0.0440.0", "v0.440.1", "v0.440.0-rc.1"].map((tag) =>
        retire.map((arg, index) => (index === 5 ? tag : arg)),
      ),
      retire.map((arg, index) => (index === 2 ? "--run" : arg)),
    ];
    for (const args of invalid) {
      const result = runReleaseCommand("release", args, {
        env: { PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}` },
      });
      assert.notEqual(result.status, 0, JSON.stringify(args));
      assert.equal(fs.existsSync(capture), false);
      assert.equal(fs.existsSync(path.join(root, "injected")), false);
    }
    const inputs = ["Cargo.toml", "Cargo.lock", "package.json", "pnpm-lock.yaml"];
    const before = inputs.map((file) => fs.readFileSync(path.join(repoRoot, file)));
    const target = "0.440.0";
    for (const args of [
      ["minor", "-y", "--pinned-target", target],
      ["patch", "-y", "--prepare-only", "--pinned-target", target],
      ["minor", "-y", "--prepare-only", "--pinned-target"],
      ["minor", "-y", "--prepare-only", "--pinned-target", target, "--pinned-target", target],
    ]) {
      const result = runReleaseCommand("release", args, {
        env: { PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}` },
      });
      assert.notEqual(result.status, 0, JSON.stringify(args));
      assert.equal(fs.existsSync(capture), false);
      assert.deepEqual(
        inputs.map((file) => fs.readFileSync(path.join(repoRoot, file))),
        before,
      );
    }
    writeFakeCommand(
      bin,
      "rust-script",
      `
      require('node:fs').writeFileSync(${JSON.stringify(capture)}, JSON.stringify(process.argv.slice(2)));
      process.exit(1);
    `,
    );
    const prepared = runReleaseCommand(
      "release",
      ["minor", "-y", "--prepare-only", "--pinned-target", target],
      { env: { PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}` } },
    );
    assert.equal(prepared.status, 1, `${prepared.stdout}\n${prepared.stderr}`);
    const current = /^version = "([^"]+)"$/m.exec(before[0].toString())?.[1];
    assert.notEqual(current, undefined);
    assert.deepEqual(JSON.parse(fs.readFileSync(capture, "utf8")), [
      "tools/commands/release/pr.rs",
      "validate-preparation-target",
      "minor",
      current,
      target,
    ]);
    assert.match(prepared.stderr, /Pinned target was not authenticated; no release files changed/);
    assert.deepEqual(
      inputs.map((file) => fs.readFileSync(path.join(repoRoot, file))),
      before,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("release promotion rejects stale main and immutable tag conflicts atomically", () => {
  const result = spawnSync("rust-script", ["--test", script], { encoding: "utf8" });
  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);
  assert.equal(result.status, 0, `${result.error ?? ""}\n${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /\b[1-9]\d* passed; 0 failed/);
  assert.match(result.stdout, /normal_cached_driver_module_changes_require_new_entry_source_pin/);
  for (const law of [
    "legacy_archive_receipts_logs_and_commit_messages_remain_byte_exact",
    "pre_release_archive_roundtrips_whole_q_custody_and_actual_binary_failure_bytes",
    "qualification_archive_replays_actual_source_policy_and_preserves_typed_failure_origin",
    "original_hosted_operator_identity_is_required_without_status_shortcuts",
    "shared_release_checks_accept_only_documented_pending_exit_with_actual_evidence",
    "legacy_and_integration_checks_keep_exact_required_names_heads_apps_and_success",
    "versioned_same_ref_lease_captures_original_budget_without_source_or_pin_mutation",
    "changed_versioned_lease_is_preserved_on_error_and_drop",
    "compensation_runner_sigkills_reaps_and_retains_whole_process_failure_packets",
    "gnu_outer_timeout_bounds_hung_node_and_inherited_stdio_descendants",
    "expired_original_budget_uses_actual_watch_finish_to_withdraw_only_acquired_entry",
    "native_queue_acquires_only_actual_returned_entry_with_expected_head_and_whole_custody",
    "owned_dequeue_uses_real_id_input_once_and_verifies_exact_entry_then_fresh_null",
    "already_removed_owned_entry_needs_fresh_identity_and_authority_but_no_write",
    "same_identity_merged_integration_allows_only_fresh_null_cleanup_noop",
    "changed_effective_required_contexts_app_or_queue_policy_refuse_before_admission",
    "missing_effective_grouping_refuses_without_secondary_lookup_or_mutation",
    "existing_entries_unknown_auto_or_changed_integration_are_never_adopted_or_mutated",
    "authentication_head_mismatch_and_graphql_failures_are_fatal_without_retry",
    "foreign_cleanup_entries_changed_lease_attempt_or_unknown_auto_remain_untouched",
    "ambiguous_dequeue_response_or_replacement_after_write_never_causes_second_mutation",
    "enqueue_records_only_authenticated_response_owner_before_fallible_postwrite_reads",
    "original_rust_readiness_adapter_preserves_pending_complete_packets_and_every_failure",
    "fresh_minor_policy_requires_every_queue_and_promotion_boundary_and_pin",
    "prepared_normal_cut_reuses_whole_tree_without_another_worktree_or_graph_install",
    "prepared_normal_cut_keeps_noninitial_and_moved_main_on_the_original_route",
    "prepared_normal_cut_rejects_changed_owner_identity_and_revoked_lease_before_creation",
    "prepared_normal_cut_refuses_hidden_changes_omitted_versions_and_raw_parent_substitution",
  ]) {
    assert.ok(result.stdout.includes(law), `actual Rust driver must execute ${law}`);
  }
});

void test("prepared integration reuse is confined to fresh exact-C preparation before unchanged full fallback", () => {
  const start = readRepoFile("tools", "support", "release", "pr_pin_start.rs");
  const integration = readRepoFile("tools", "support", "release", "pr_pin_integration.rs");
  const fresh = start.slice(start.indexOf("fn start_selected("), start.indexOf("pub fn resume("));
  const resume = start.slice(start.indexOf("pub fn resume("));
  assert.equal(fresh.split("PreparedNormalCut::capture(").length, 2);
  assert.ok(!resume.includes("PreparedNormalCut::capture("));
  assert.match(resume, /budget\.as_ref\(\),\s*None,\s*&operator/);
  const ordered = (source: string, tokens: string[]) => {
    const positions = tokens.map((token) => source.indexOf(token));
    positions.forEach((position, index) => assert.ok(position >= 0, tokens[index]));
    positions.slice(1).forEach((position, index) => assert.ok(position > positions[index]!));
  };
  const tokens = [
    "let mut main = github::fetch_main(root)?;",
    "prepared.project(source, &main, root, &||",
    "\n            main = github::fetch_main(root)?;",
    "if main == source.cut {",
    'return Err("Integration branch appeared during prepared-tree reuse".into());',
    "return publish_integration(source, &branch, &head, budget, root);",
    "let path = pr_start::pinned_integration_worktree(&main, root)?;",
    'pr_start::prepare("patch", &path)?;',
    "super::retire::prepare(bump, &candidate.tag, budget, &path)?;",
    "metadata::verify_delta(",
    "publish_integration(source, &branch, &head, budget, &path)",
  ];
  const route = integration.slice(integration.indexOf("pub(super) fn integration_pr("));
  ordered(route, tokens);
  for (const token of tokens) assert.throws(() => ordered(route.replace(token, ""), tokens));
  assert.ok(!integration.includes("--force"));
  assert.ok(!integration.includes("--force-with-lease"));
  assert.ok(integration.includes('"{head}:refs/heads/{branch}"'));
  assert.ok(integration.includes("operator.verify()"));
  assert.ok(integration.includes('"commit-tree"'));
});

function assertOwnedWithdrawal(watch: string, queue: string, rollback: string, runner: string) {
  assert.ok(
    !watch.includes("--auto") &&
      !queue.includes("--auto") &&
      !rollback.includes("--auto") &&
      !runner.includes("--auto"),
  );
  assert.equal(watch.split("queue::enqueue(").length, 2, "one native admission call site");
  const required = watch.indexOf("pr_promote::checks_pass(candidate, root)?");
  assert.ok(required >= 0 && required < watch.indexOf("queue::enqueue("));
  assert.ok(watch.includes("if early_candidate.is_none() {\n                    let integration"));
  const outcome = watch.indexOf("let result = (|| {");
  const actualWatch = watch.indexOf("pub(super) fn watch(");
  const finish = rollback.indexOf("pub(super) fn finish_owned_watch(");
  assert.ok(finish >= 0 && outcome > actualWatch && actualWatch >= 0);
  const call = /return finish_owned_watch\(\s*failure,\s*&context,/u.exec(watch);
  assert.ok(call && call.index > watch.indexOf("})();", outcome));
  assert.ok(rollback.slice(finish).includes("queue::cleanup("));
  assert.ok(!rollback.slice(finish).includes("budget.check()"));
  assert.ok(watch.includes("(&result, &early_candidate, &owned_queue)"));
  assert.ok(watch.includes("Owned queue withdrawal refused; preserve its complete evidence"));
}

test("early queue admission waits ordinary required checks and retains finite owned withdrawal evidence", () => {
  const watch = readRepoFile("tools", "support", "release", "pr_pin_watch.rs");
  const queue = readRepoFile("tools", "support", "release", "pr_pin_queue.rs");
  const runner = readRepoFile("tools", "support", "release", "pr_pin_process.rs");
  const rollback = readRepoFile("tools", "support", "release", "pr_pin_rollback.rs");
  const pin = readRepoFile("tools", "support", "release", "pr_pin.rs");
  assert.match(queue, /#\[path = "pr_pin_process.rs"\]\s*mod process;/u);
  assert.match(pin, /#\[path = "pr_pin_rollback.rs"\]\s*mod rollback;/u);
  assert.ok(watch.includes("pub(super) use super::rollback::{Rollback, finish_owned_watch};"));
  assertOwnedWithdrawal(watch, queue, rollback, runner);
  for (const [changedWatch, changedQueue, changedRollback, changedRunner] of [
    [watch.replace("pr_promote::checks_pass(candidate, root)?", "false"), queue, rollback, runner],
    [watch.replace("queue::enqueue(", "unowned::enqueue("), queue, rollback, runner],
    [watch.replace("return finish_owned_watch(", "return unowned_watch("), queue, rollback, runner],
    [watch, queue + " --auto", rollback, runner],
    [watch, queue, rollback + " --auto", runner],
    [
      watch,
      queue,
      rollback.replace("pub(super) fn finish_owned_watch(", "fn foreign_finish("),
      runner,
    ],
    [watch, queue, rollback.replace("queue::cleanup(", "unowned::cleanup("), runner],
    [watch, queue, rollback + " budget.check()", runner],
    [watch, queue, rollback, runner + " --auto"],
  ])
    assert.throws(() =>
      assertOwnedWithdrawal(changedWatch, changedQueue, changedRollback, changedRunner),
    );
  const operator = parse(readRepoFile(".github", "workflows", "release-operator.yml"));
  const invocation = operator.jobs.operator.steps.find(
    (step: { env?: Record<string, string> }) => step.env?.GH_TOKEN,
  );
  assert.equal(
    invocation.env.VIZE_RELEASE_INTEGRATION_QUEUE_EVIDENCE,
    "${{ runner.temp }}/release-integration-queue/responses.jsonl",
  );
  const retention = operator.jobs.operator.steps.at(-1);
  assert.equal(retention.name, "Preserve exact owned integration queue responses");
  assert.equal(retention.if, "${{ always() }}");
  assert.equal(retention.with.path, "${{ runner.temp }}/release-integration-queue");
  assert.equal(retention.with["if-no-files-found"], "ignore");
  assert.equal(retention.with["retention-days"], 30);
});

test("actual cached Rust readiness command retains whole accepted jobs and refuses malformed evidence", () => {
  const ready = {
    id: 9,
    run_id: 7,
    run_attempt: 2,
    head_sha: "1".repeat(40),
    name: "Release candidate ready",
    status: "completed",
    conclusion: "success",
    steps: [{ name: "original", conclusion: "success" }],
    future: { field: "preserved" },
  };
  const invoke = (input: string) =>
    spawnSync("rust-script", [script, "check-release-readiness"], {
      cwd: repoRoot,
      encoding: "utf8",
      input,
    });
  for (const [jobs, expected] of [
    [[ready], { ready: true, readyJob: ready }],
    [[], { ready: false }],
    [[{ ...ready, status: "in_progress", conclusion: null }], { ready: false }],
  ] as const) {
    const result = invoke(JSON.stringify(jobs));
    assert.equal(result.status, 0, `${result.error ?? ""}\n${result.stdout}\n${result.stderr}`);
    assert.deepEqual(JSON.parse(result.stdout), expected);
  }
  for (const input of [
    ...["failure", "cancelled", "skipped"].map((conclusion) =>
      JSON.stringify([{ ...ready, conclusion }]),
    ),
    JSON.stringify([ready, ready]),
    "null",
    "{}",
    "[] trailing",
    " ".repeat(2_097_153),
  ]) {
    const result = invoke(input);
    assert.equal(result.status, 1, `${result.error ?? ""}\n${result.stdout}\n${result.stderr}`);
    assert.equal(result.stdout, "");
    assert.ok(result.stderr.length > 0);
  }
});

test("release-only workflow validates all artifacts before promotion and preserves normal PR cost", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "release.yml"));
  assert.deepEqual(Object.keys(workflow.on), ["workflow_dispatch"]);
  assert.match(workflow["run-name"], /Release \{0\} PR #\{1\} @ \{2\}/);
  assert.ok(workflow["run-name"].endsWith("}}"));
  assert.equal(workflow.jobs.candidate.if, "inputs.release_pr != ''");
  const ready = workflow.jobs["candidate-ready"];
  assert.equal(ready.name, "Release candidate ready");
  assert.equal(
    ready.if,
    undefined,
    "default success dependency condition must reject skipped/failed builds",
  );
  assert.deepEqual(
    [...ready.needs].sort((left: string, right: string) => left.localeCompare(right)),
    [
      "build-cli",
      "build-editor-extensions",
      "build-native-all",
      "build-release-packages",
      "build-wasm-package",
      "candidate-preflight",
      "smoke-release-packages",
    ],
  );
  assert.equal(workflow.jobs["candidate-preflight"].needs, "candidate");
  assert.deepEqual(workflow.jobs["candidate-preflight"].permissions, {
    actions: "read",
    contents: "read",
    issues: "read",
    "pull-requests": "read",
  });
  assert.equal(workflow.jobs["release-preflight"].needs, "candidate-ready");
  assert.equal(
    workflow.jobs["release-preflight"].uses,
    "./.github/workflows/release-promotion.yml",
  );
  const publication = workflow.jobs["create-github-release"].steps.find((step: { uses?: string }) =>
    step.uses?.startsWith("softprops/action-gh-release@"),
  );
  assert.equal(publication.with.tag_name, "${{ inputs.tag_name }}");
  const wasm = workflow.jobs["build-wasm-package"].steps;
  const installed = wasm.findIndex(
    (step: { with?: Record<string, unknown> }) => step.with?.["run-install"] === true,
  );
  const packed = wasm.findIndex((step: { run?: string }) =>
    step.run?.includes("node tools/commands/release/npm/smoke-release-install.ts npm/wasm"),
  );
  assert.ok(
    installed >= 0 && packed > installed,
    "WASM tarball checks must run before the candidate can be promoted",
  );
  const barrier = parse(readRepoFile(".github", "workflows", "release-promotion.yml"));
  assert.deepEqual(barrier.permissions, { contents: "read", "pull-requests": "read" });
  assert.match(barrier.jobs.promotion.steps.at(-1).run, /pr\.rs wait-promotion/);
});

test("pinned publication retains the original build barrier and explicitly opts into every verifier", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "release.yml"));
  assert.deepEqual(workflow.on.workflow_dispatch.inputs.pinned, {
    description: "Publish an immutable source cut after a protected version integration",
    required: false,
    default: false,
    type: "boolean",
  });
  assert.match(workflow["run-name"], /Pinned Release \{0\} PR #\{1\} @ \{2\}/);
  const authorization = workflow.jobs.candidate.steps.at(-1);
  assert.equal(authorization.env.RELEASE_PINNED, "${{ inputs.pinned }}");
  assert.match(authorization.run, /validate-pinned/);
  for (const job of ["candidate-preflight", "release-preflight"]) {
    assert.equal(workflow.jobs[job].with.pinned, "${{ inputs.pinned }}");
  }
  const preflight = parse(readRepoFile(".github", "workflows", "release-preflight.yml"));
  assert.equal(preflight.on.workflow_call.inputs.pinned.default, false);
  assert.equal(preflight.jobs.verify.steps.at(-1).env.RELEASE_PINNED, "${{ inputs.pinned }}");
  const promotion = parse(readRepoFile(".github", "workflows", "release-promotion.yml"));
  assert.equal(promotion.on.workflow_call.inputs.pinned.default, false);
  assert.match(promotion.jobs.promotion.steps.at(-1).run, /wait-promotion-pinned/);
  assert.equal(workflow.jobs["release-preflight"].needs, "candidate-ready");
  assert.deepEqual(promotion.permissions, { contents: "read", "pull-requests": "read" });
});

test("Actions candidate authorization checks actual Git ancestry and never creates a tag", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-candidate-test-"));
  const work = path.join(root, "work");
  const remote = path.join(root, "remote.git");
  const bin = path.join(root, "bin");
  fs.mkdirSync(work);
  fs.mkdirSync(bin);
  const git = (...args: string[]) => {
    const result = spawnSync("git", args, { cwd: work, encoding: "utf8" });
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  };
  try {
    git("init", "--bare", remote);
    git("init", "-b", "main");
    git("config", "user.name", "Release fixture");
    git("config", "user.email", "release@example.invalid");
    git("config", "commit.gpgsign", "false");
    git("config", "core.hooksPath", path.join(root, "no-hooks"));
    fs.writeFileSync(path.join(work, "Cargo.toml"), '[workspace.package]\nversion = "1.2.2"\n');
    git("add", "Cargo.toml");
    git("commit", "-m", "main");
    const base = git("rev-parse", "HEAD");
    git("remote", "add", "origin", remote);
    git("push", "origin", "main");
    git("checkout", "-b", "release/v1.2.3");
    fs.writeFileSync(path.join(work, "Cargo.toml"), '[workspace.package]\nversion = "1.2.3"\n');
    git("commit", "-am", "chore: release v1.2.3");
    const head = git("rev-parse", "HEAD");
    const pull = {
      number: 42,
      state: "open",
      merged: false,
      draft: true,
      user: { login: "maintainer" },
      head: { sha: head, ref: "release/v1.2.3", repo: { full_name: "owner/repo" } },
      base: { sha: base, ref: "main", repo: { full_name: "owner/repo" } },
    };
    writeFakeCommand(
      bin,
      "gh",
      `
      const args = process.argv.slice(2);
      if (args.join(' ') === 'api repos/owner/repo/pulls/42') {
        console.log(${JSON.stringify(JSON.stringify(pull))});
      } else if (args.join(' ') === 'api repos/owner/repo/collaborators/maintainer/permission') {
        console.log(JSON.stringify({role_name: process.env.TEST_RELEASE_ROLE}));
      } else { console.error('Unexpected GitHub request', args); process.exit(1); }
    `,
    );
    const validate = (role = "maintain", sha = head) =>
      spawnSync("rust-script", [script, "validate", "42", sha, "v1.2.3"], {
        cwd: work,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
          GITHUB_REPOSITORY: "owner/repo",
          GITHUB_SHA: head,
          TEST_RELEASE_ROLE: role,
          GITHUB_OUTPUT: path.join(root, "outputs"),
        },
      });
    fs.writeFileSync(path.join(root, "outputs"), "");
    const accepted = validate();
    assert.equal(accepted.status, 0, `${accepted.error ?? ""}\n${accepted.stderr}`);
    assert.match(fs.readFileSync(path.join(root, "outputs"), "utf8"), new RegExp(`base=${base}`));
    assert.match(validate("write").stderr, /maintain or admin/);
    assert.match(validate("maintain", base).stderr, /head changed/);
    git("checkout", "main");
    git("commit", "--allow-empty", "-m", "main advanced");
    git("push", "origin", "main");
    git("checkout", "release/v1.2.3");
    const stale = validate();
    assert.notEqual(stale.status, 0);
    assert.match(stale.stderr, /main advanced/);
    assert.equal(git("ls-remote", "--tags", "origin"), "");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
