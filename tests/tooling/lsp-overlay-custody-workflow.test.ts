import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

type Step = {
  name?: string;
  uses?: string;
  run?: string;
  if?: string;
  with?: Record<string, unknown>;
};
const workflow = parse(
  fs.readFileSync(
    new URL("../../.github/workflows/lsp-overlay-custody.yml", import.meta.url),
    "utf8",
  ),
);
const job = workflow.jobs["native-overlay"];
const steps: Step[] = job.steps;
const run = steps.find(
  (step) =>
    step.name ===
    "Exercise native authored logical namespace and retained physical policy controls",
)!.run!;

function platformSelectors(script: string): [string, string, boolean, boolean][] {
  let platform: string | undefined;
  const selected: [string, string, boolean, boolean][] = [];
  for (const sourceLine of script.split("\n")) {
    const line = sourceLine.trim();
    const guard = line.match(/^if \[ "\$RUNNER_OS" = (\w+) \]; then$/u);
    if (guard) platform = guard[1];
    else if (line === "fi") platform = undefined;
    else if (platform) {
      const command = line.match(/\bcargo test .* --lib (\S+) -- (.+)$/u);
      if (command)
        selected.push([
          platform,
          command[1]!,
          line.startsWith("VIZE_TEST_REQUIRE_TSGO=1 "),
          command[2]!.split(/\s+/u).includes("--exact"),
        ]);
    }
  }
  return selected;
}

test("overlay custody preserves the exact source checkout, two hosts and read-only job boundaries", () => {
  assert.deepEqual(Object.keys(workflow.on), ["pull_request"]);
  assert.deepEqual(workflow.on.pull_request.types, ["opened", "synchronize", "reopened"]);
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.deepEqual(workflow.concurrency, {
    group: "lsp-overlay-custody-${{ github.event.pull_request.number }}",
    "cancel-in-progress": false,
  });
  assert.deepEqual(job.strategy, {
    "fail-fast": false,
    matrix: {
      include: [
        { os: "Linux", runner: "blacksmith-32vcpu-ubuntu-2404" },
        { os: "Windows", runner: "blacksmith-32vcpu-windows-2025" },
      ],
    },
  });
  assert.equal(job["runs-on"], "${{ matrix.runner }}");
  assert.deepEqual(job.defaults, { run: { shell: "bash" } });
  const checkout = steps.find((step) => step.uses?.startsWith("actions/checkout@"))!;
  assert.deepEqual(checkout.with, {
    ref: "${{ github.event.pull_request.head.sha }}",
    "persist-credentials": false,
  });
  const cache = steps.find((step) => step.uses === "./.github/actions/setup-rust-sticky-cache")!;
  assert.deepEqual(cache.with, {
    key: "lsp-overlay-custody",
    "cache-key-suffix": "${{ runner.os }}-${{ runner.arch }}",
  });
  assert.ok(
    steps.indexOf(cache) < steps.indexOf(steps.find((step) => step.run?.includes("cargo test"))!),
  );
  for (const input of [
    "Cargo.toml",
    "Cargo.lock",
    "crates/vize_canon/Cargo.toml",
    ".github/actions/setup-rust-sticky-cache/**",
    "tools/support/compat/github/full-rust-runtime.ts",
  ]) {
    assert.equal(workflow.on.pull_request.paths.filter((path: string) => path === input).length, 1);
  }
});

test("overlay custody runs the exact platform controls, mandatory native case and strict workspace compiler", () => {
  const expected: [string, string, boolean, boolean][] = [
    [
      "Windows",
      "corsa_bridge::vue_document::build::tests::windows_overlays::verbatim_windows_overlay_inputs_preserve_materialized_revision_bytes",
      false,
      true,
    ],
    ["Linux", "corsa_bridge::bridge::initial_diagnostics::tests::", false, false],
    [
      "Linux",
      "lsp_client::editor_lsp::configured_project::shared_transport::tests::native::native_editor_resolves_package_alias_from_monorepo_root_with_one_api_editor_owner",
      true,
      true,
    ],
    [
      "Linux",
      "lsp_client::editor_lsp::configured_project::shared_transport::tests::native::replay::native_editor_resolves_package_alias_from_monorepo_root_replays_unsaved_api_after_disk_events",
      true,
      true,
    ],
    [
      "Linux",
      "lsp_client::editor_lsp::configured_project::shared_transport::tests::logical_currentness::native_logical_revision_refusal_quarantines_current_api_but_preserves_held_snapshot",
      true,
      true,
    ],
  ];
  assert.deepEqual(platformSelectors(run), expected);
  assert.deepEqual(
    platformSelectors(
      run
        .split("\n")
        .map((line) => `    ${line}`)
        .join("\n"),
    ),
    expected,
  );
  assert.notDeepEqual(
    platformSelectors(run.replace('"$RUNNER_OS" = Windows', '"$RUNNER_OS" = Linux')),
    expected,
  );
  assert.deepEqual(
    [...run.matchAll(/^cargo test .+ -- --exact --ignored --nocapture .+$/gm)].map(
      (match) => match[0],
    ),
    [
      'cargo test --locked --profile ci -p vize_canon --lib lsp_client::session_storage::native_tests::actual_native_old_and_private_roots_preserve_complete_configured_overlay_answers -- --exact --ignored --nocapture > "$VIZE_PRIVATE_OVERLAY_CAPTURE/native.stdout" 2> "$VIZE_PRIVATE_OVERLAY_CAPTURE/native.stderr"',
    ],
  );
  assert.deepEqual(
    [...run.matchAll(/^compiler=.+$/gm)].map((match) => match[0]),
    [
      `compiler="$(node --input-type=module -e 'import {createRequire} from "node:module"; console.log(createRequire(new URL("./tests/package.json", import.meta.url)).resolve("typescript/lib/tsc.js"));')"`,
    ],
  );
  assert.deepEqual(
    [...run.matchAll(/^node "\$compiler" --ignoreConfig .+$/gm)].map((match) => match[0]),
    [
      'node "$compiler" --ignoreConfig --strict --noEmit --noImplicitAny --noUncheckedIndexedAccess --exactOptionalPropertyTypes --erasableSyntaxOnly --target es2023 --module esnext --moduleResolution bundler --allowImportingTsExtensions --skipLibCheck --types node --typeRoots "$PWD/node_modules/@types" .github/actions/lsp-overlay-custody/source-cli.ts > "$VIZE_PRIVATE_OVERLAY_CAPTURE/source-cli-helper-strict.stdout" 2> "$VIZE_PRIVATE_OVERLAY_CAPTURE/source-cli-helper-strict.stderr"',
    ],
  );
});

test("overlay custody retains evidence on every outcome and gives each run attempt its own artifact", () => {
  const uploads = steps.filter((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(uploads.length, 1);
  assert.equal(uploads[0]!.if, "always()");
  assert.deepEqual(uploads[0]!.with, {
    name: "lsp-overlay-custody-${{ matrix.os }}-${{ github.event.pull_request.head.sha }}-${{ github.run_attempt }}",
    path: "${{ runner.temp }}/lsp-overlay-custody/\ntarget/nextest/full/native-runtime-provider.json\n",
    "if-no-files-found": "error",
    "retention-days": 14,
  });
  assert.equal(
    workflow.on.pull_request.paths.filter(
      (path: string) => path === "tests/tooling/lsp-overlay-custody-workflow.test.ts",
    ).length,
    1,
  );
});

test("URI review controls run independently of source admission and retain both failed cohorts", () => {
  const review = steps.find((step) => step.name === "Require URI identity review regressions")!;
  assert.equal(review.if, "${{ always() && runner.os == 'Linux' }}");
  const script = review.run!;
  assert.match(script, /test "\$\(git rev-parse HEAD\)" = "\$EXPECTED_SOURCE"/);
  const commands = script.split("\n").filter((line) => line.startsWith("cargo test "));
  assert.equal(commands.length, 2);
  const selectors = commands.map((command) =>
    command.split(" -- ")[1]!.split(" --nocapture")[0]!.split(" "),
  );
  assert.deepEqual(selectors, [
    [
      "lsp_client::editor_lsp::client::tests::hierarchy_items_preserve_unmapped_foreign_documents",
      "lsp_client::session::tests::moving_one_spelling_preserves_the_other_native_identity_owner",
      "lsp_client::session::tests::releasing_either_normalized_owner_keeps_the_other_until_the_last_close",
      "lsp_client::utils::tests::converging_workspace_edit_keys_keep_every_edit_in_each_original_order",
      "lsp_client::editor_lsp::file_rename::tests::materialized_renames_share_one_native_namespace_and_keep_each_caller_context",
      "lsp_client::lifecycle::workspace::initial_project_tests::native_first_materialized_editor_matches_existing_activation_and_delta",
      "lsp_client::session::logical::tests::an_independent_non_authored_owner_is_not_promoted_to_logical_authority",
      "lsp_client::session::logical::tests::a_missing_attached_snapshot_is_an_error_not_accepted_logical_authority",
    ],
    [
      "ide::corsa_support::authored_uri::tests::closed_foreign_targets_keep_the_import_path_before_an_open_owner_takes_precedence",
    ],
  ]);
  for (const role of ["canon", "maestro"]) {
    assert.ok(script.includes(`> "$capture/${role}.stdout" 2> "$capture/${role}.stderr"`));
    assert.ok(script.includes(`${role}_status=$?`));
    assert.ok(script.includes(`test "$${role}_status" = 0`));
  }
  assert.ok(script.indexOf("maestro_status=$?") < script.indexOf('test "$canon_status" = 0'));
});
