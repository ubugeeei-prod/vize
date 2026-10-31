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
});

test("overlay custody runs the exact platform controls, mandatory native case and strict workspace compiler", () => {
  const commands = [...run.matchAll(/^  (cargo test .+)$/gm)].map((match) => match[1]);
  assert.deepEqual(commands, [
    'cargo test --locked --profile ci -p vize_canon --lib corsa_bridge::vue_document::build::tests::windows_overlays::verbatim_windows_overlay_inputs_preserve_materialized_revision_bytes -- --exact --nocapture > "$VIZE_PRIVATE_OVERLAY_CAPTURE/windows-overlay-input.stdout" 2> "$VIZE_PRIVATE_OVERLAY_CAPTURE/windows-overlay-input.stderr"',
    'cargo test --locked --profile ci -p vize_canon --lib corsa_bridge::bridge::initial_diagnostics::tests:: -- --nocapture > "$VIZE_PRIVATE_OVERLAY_CAPTURE/leases.stdout" 2> "$VIZE_PRIVATE_OVERLAY_CAPTURE/leases.stderr"',
  ]);
  assert.deepEqual(
    [
      ...run.matchAll(
        /^  VIZE_TEST_REQUIRE_TSGO=1 cargo test .+ --lib (\S+) -- --exact --nocapture > "\$VIZE_PRIVATE_OVERLAY_CAPTURE\/([^"]+)"/gm,
      ),
    ].map((match) => match.slice(1)),
    [
      [
        "lsp_client::editor_lsp::configured_project::shared_transport::tests::native::native_editor_resolves_package_alias_from_monorepo_root_with_one_api_editor_owner",
        "shared-api-owner.stdout",
      ],
      [
        "lsp_client::editor_lsp::configured_project::shared_transport::tests::native::replay::native_editor_resolves_package_alias_from_monorepo_root_replays_unsaved_api_after_disk_events",
        "shared-api-replay.stdout",
      ],
    ],
  );
  assert.deepEqual(
    [...run.matchAll(/^if \[ "\$RUNNER_OS" = (\w+) \]; then$/gm)].map((match) => match[1]),
    ["Windows", "Linux"],
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
