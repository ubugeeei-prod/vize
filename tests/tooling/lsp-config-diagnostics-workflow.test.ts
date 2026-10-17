import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { createHash } from "node:crypto";
import { parse } from "yaml";

import {
  retainIntegrationProducer,
  requireRustTestsExecuted,
  retainMeasuredProducer,
  retainNativeLog,
} from "../../tools/support/compat/github/run-lsp-config-diagnostics.ts";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  uses?: string;
  run?: string;
  if?: string;
  with?: Record<string, unknown>;
};

test("startup diagnostics retain the production producer and unchanged scale-one oracle", () => {
  const workflow = parse(readRepoFile(".github/workflows/lsp-config-diagnostics.yml"));
  assert.deepEqual(Object.keys(workflow.on), ["pull_request"]);
  assert.deepEqual(workflow.on.pull_request.paths, [
    ".github/workflows/lsp-config-diagnostics.yml",
    "tools/support/compat/github/run-lsp-config-diagnostics.ts",
    "tests/tooling/lsp-config-diagnostics-workflow.test.ts",
  ]);
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.equal(workflow.jobs["vben-startup"]["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(workflow.env.RAYON_NUM_THREADS, "4");
  assert.equal(workflow.env.VIZE_PERF_BUDGET_SCALE, "1");
  assert.equal(workflow.env.VIZE_TRACE_EDITOR_PREPARATION, "1");
  const steps: Step[] = workflow.jobs["vben-startup"].steps;
  const checkout = steps.find((step) => step.uses?.startsWith("actions/checkout@"))!;
  assert.deepEqual(checkout.with, {
    ref: "${{ github.event.pull_request.head.sha }}",
    "persist-credentials": false,
  });
  const build = steps.find(
    (step) => step.name === "Build and receipt the same Vue parity producer",
  )!;
  assert.equal(
    build.run,
    "cargo build --profile ci -p vize --features legacy\nnode tests/differential/build-receipt.ts --legacy\n",
  );
  assert.equal(
    steps.find((step) => step.name === "Hydrate only the original pinned Vben fixture")?.run,
    "git submodule update --init --depth 1 tests/_fixtures/_git/vue-vben-admin",
  );
  const controls = steps.findIndex(
    (step) => step.name === "Verify native preparation and request freshness laws",
  );
  const measurement = steps.findIndex(
    (step) => step.name === "Run the unchanged original performance case with native phase tracing",
  );
  assert.ok(measurement > steps.indexOf(build));
  assert.ok(controls > measurement);
  assert.equal(
    steps[controls].run,
    "cargo test --locked --profile ci -p vize_maestro --features legacy --lib server::initial_diagnostics::\n" +
      "cargo test --locked --profile ci -p vize_maestro --features legacy --lib server::native_requests::\n" +
      "cargo clippy --locked --profile ci -p vize_maestro --features legacy --all-targets -- -D warnings\n",
  );
  const symlinkControls = steps.findIndex(
    (step) => step.name === "Observe the original six symlink workspace layouts",
  );
  const unionControls = steps.findIndex(
    (step) => step.name === "Verify real union ownership and reverse open-order regressions",
  );
  assert.ok(unionControls > controls);
  const completionControls = steps.findIndex(
    (step) =>
      step.name === "Resolve the original native candidate after unchanged sibling diagnostics",
  );
  const freshnessControls = steps.findIndex(
    (step) =>
      step.name === "Verify whole native views and issued candidate freshness with nonzero tests",
  );
  assert.ok(freshnessControls > unionControls);
  assert.ok(completionControls > freshnessControls);
  assert.match(
    steps[freshnessControls].run!,
    /run_required_rust_tests native-view cargo test .*corsa_bridge::vue_document::build::tests:: -- --nocapture/,
  );
  assert.match(
    steps[freshnessControls].run!,
    /run_required_rust_tests issued-candidates cargo test .*lsp_client::editor_lsp::completion_resolve::issued::tests:: -- --nocapture/,
  );
  assert.match(steps[freshnessControls].run!, /--require-rust-tests/);
  assert.match(steps[freshnessControls].run!, /return "\$status"/);
  assert.ok(symlinkControls > completionControls);
  assert.match(
    steps[completionControls].run!,
    /--test lsp_completion_sibling_overlay_cli -- --nocapture/,
  );
  assert.match(steps[completionControls].run!, /--retain-completion-producer/);
  assert.match(steps[completionControls].run!, /exit "\$status"/);
  assert.deepEqual(
    Array.from(
      readRepoFile("crates/vize/tests/lsp_completion_sibling_overlay_cli.rs").matchAll(
        /#\[test\]\nfn (\w+)\(/g,
      ),
      (match) => match[1],
    ),
    ["native_completion_resolves_after_unchanged_sibling_diagnostics_with_a_closed_dependency"],
  );
  assert.match(
    readRepoFile("crates/vize_canon/src/corsa_bridge/vue_document/build.rs"),
    /#\[cfg\(test\)\]\nmod tests;/,
  );
  assert.match(
    readRepoFile("crates/vize_canon/src/lsp_client/editor_lsp/completion_resolve/issued.rs"),
    /#\[cfg\(test\)\]\nmod tests;/,
  );
  assert.equal(
    steps[unionControls].run,
    "cargo test --locked --profile ci -p vize_canon --features legacy --lib corsa_bridge::vue_document_package_tests::symlink_workspace::\n" +
      "cargo test --locked --profile ci -p vize_canon --features legacy --lib batch::virtual_project::materialize::alias_transition_tests::\n" +
      "cargo test --locked --profile ci -p vize_canon --features legacy --lib corsa_bridge::vue_dependencies_alias::context::cache::tests::\n" +
      "cargo clippy --locked --profile ci -p vize_canon --features legacy --lib -- -D warnings\n",
  );
  // The #[path] fixture is nested under materialize, not virtual_project::tests.
  // Bind this filter to its real module and both existing test declarations.
  assert.match(
    readRepoFile("crates/vize_canon/src/batch/virtual_project/materialize.rs"),
    /#\[path = "tests\/editor_union_alias_transition\.rs"\]\nmod alias_transition_tests;/,
  );
  assert.deepEqual(
    Array.from(
      readRepoFile(
        "crates/vize_canon/src/batch/virtual_project/tests/editor_union_alias_transition.rs",
      ).matchAll(/#\[test\]\nfn (\w+)\(/g),
      (match) => match[1],
    ),
    [
      "editor_union_replaces_external_link_with_owned_workspace_alias",
      "editor_union_still_rejects_distinct_owned_alias_targets",
    ],
  );
  assert.equal(
    steps[symlinkControls].run,
    "set +e\n" +
      "cargo test --locked --profile ci -p vize --features legacy --message-format=json --test lsp_symlink_workspace_staging_cli original_include_and_open_order_controls_keep_all_authored_bytes -- --nocapture > target/vize-tests/lsp-config-diagnostics/symlink.stdout 2> target/vize-tests/lsp-config-diagnostics/symlink.stderr\n" +
      "status=$?\nset -e\n" +
      "printf '%s\\n' \"$status\" > target/vize-tests/lsp-config-diagnostics/symlink.status\n" +
      "node tools/support/compat/github/run-lsp-config-diagnostics.ts --retain-symlink-producer\n" +
      "cat target/vize-tests/lsp-config-diagnostics/symlink.stdout\n" +
      'cat target/vize-tests/lsp-config-diagnostics/symlink.stderr >&2\nexit "$status"\n',
  );
  const upload = steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"))!;
  assert.equal(upload.if, "always()");
  for (const evidence of [
    "target/ci/vize",
    "target/ci/vize.differential-build.json",
    "node_modules/.vize/lsp.log",
    "target/vize-tests/metrics/vben-lsp-incremental/",
  ]) {
    assert.ok(String(upload.with?.path).split("\n").includes(evidence));
  }
  assert.equal(upload.with?.["include-hidden-files"], true);
  for (const step of steps) {
    assert.equal("continue-on-error" in step, false);
    assert.doesNotMatch(step.run ?? "", /warmup|budget.scale|\|\| true/);
  }
});

test("native phase logs retain all raw bytes after the mutable fixture is removed", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-startup-log-custody-"));
  try {
    const fixture = path.join(directory, "mutable-fixture");
    const native = path.join(directory, "node_modules/.vize/lsp.log");
    const artifact = path.join(directory, "native-lsp.log");
    fs.mkdirSync(fixture);
    fs.mkdirSync(path.dirname(native), { recursive: true });
    const original = Buffer.from(
      "2026-10-17T01:18:34Z INFO phase=alias_build\r\nelapsed_ms=147.25\n\0raw\n",
    );
    fs.writeFileSync(native, original);
    fs.rmSync(fixture, { recursive: true });
    assert.equal(retainNativeLog(native, artifact), true);
    assert.deepEqual(fs.readFileSync(native), original);
    assert.deepEqual(fs.readFileSync(artifact), original);
    fs.rmSync(native);
    assert.equal(retainNativeLog(native, path.join(directory, "missing.log")), false);
    assert.equal(fs.existsSync(path.join(directory, "missing.log")), false);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("post-measurement Cargo relinking cannot replace the measured producer evidence", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-measured-producer-"));
  try {
    const binary = path.join(directory, "vize");
    const receipt = path.join(directory, "vize.differential-build.json");
    const original = Buffer.from([0x7f, 0x45, 0x4c, 0x46, 0xff, 0, 3]);
    const originalReceipt = Buffer.from(
      JSON.stringify({ binarySha256: createHash("sha256").update(original).digest("hex") }),
    );
    fs.writeFileSync(binary, original);
    fs.writeFileSync(receipt, originalReceipt);
    const retained = retainMeasuredProducer(binary, receipt, directory);
    fs.writeFileSync(binary, "integration test relink with dev dependencies");
    fs.writeFileSync(receipt, "subsequent producer receipt");
    assert.deepEqual(fs.readFileSync(path.join(directory, retained.retainedBinary)), original);
    assert.deepEqual(
      fs.readFileSync(path.join(directory, retained.retainedBuildReceipt)),
      originalReceipt,
    );
    assert.throws(() =>
      retainMeasuredProducer(
        binary,
        path.join(directory, retained.retainedBuildReceipt),
        directory,
      ),
    );
    assert.deepEqual(fs.readFileSync(path.join(directory, retained.retainedBinary)), original);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("the six-layout CLI receipt uses the Cargo-reported integration producer", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-symlink-producer-"));
  try {
    const executable = path.join(directory, "test-cli");
    const log = path.join(directory, "cargo.stdout");
    const bytes = Buffer.from([0x7f, 0x45, 0x4c, 0x46, 0, 0xff, 9]);
    fs.writeFileSync(executable, bytes);
    const producer = {
      reason: "compiler-artifact",
      target: { name: "vize", kind: ["bin"] },
      executable,
    };
    fs.writeFileSync(log, `running 1 test\noriginal test output\n${JSON.stringify(producer)}\n`);
    const metadata = retainIntegrationProducer(log, directory);
    assert.equal(metadata.executable, executable);
    assert.equal(metadata.binarySha256, createHash("sha256").update(bytes).digest("hex"));
    assert.deepEqual(metadata.cargoArtifact, producer);
    assert.deepEqual(fs.readFileSync(path.join(directory, "symlink-test-vize")), bytes);
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(directory, "symlink-test-producer.json"), "utf8")),
      metadata,
    );
    const completionBytes = Buffer.concat([bytes, Buffer.from([10])]);
    fs.writeFileSync(executable, completionBytes);
    const completion = retainIntegrationProducer(log, directory, "completion-test");
    assert.equal(completion.schema, "vize.completion-test-producer");
    assert.equal(
      completion.binarySha256,
      createHash("sha256").update(completionBytes).digest("hex"),
    );
    assert.deepEqual(
      fs.readFileSync(path.join(directory, "completion-test-vize")),
      completionBytes,
    );
    assert.deepEqual(fs.readFileSync(path.join(directory, "symlink-test-vize")), bytes);
    for (const messages of [
      [],
      [{ ...producer, target: { name: "vize_canon", kind: ["lib"] } }],
      [producer, producer],
    ]) {
      fs.writeFileSync(
        log,
        `running 1 test\n${messages.map((message) => JSON.stringify(message)).join("\n")}`,
      );
      assert.throws(() => retainIntegrationProducer(log, directory), /exactly one real test CLI/);
      assert.deepEqual(fs.readFileSync(path.join(directory, "symlink-test-vize")), bytes);
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("Rust producer evidence refuses empty and missing test enrollment", () => {
  requireRustTestsExecuted("running 1 test\ntest native_case ... ok\n");
  requireRustTestsExecuted("running 12 tests\n");
  for (const output of ["running 0 tests\n", "Finished ci profile\n", "not running 1 test\n"]) {
    assert.throws(() => requireRustTestsExecuted(output), /real nonzero Rust test count/);
  }
});
