import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  expectedBuildIdentity,
  LEGACY_BUILD_RECIPE,
  validateBuildReceipt,
} from "../../../../tests/differential/build-receipt.ts";
import { sha256 } from "../../../../tests/differential/sha256.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const directory = path.join(root, "target/vize-tests/lsp-config-diagnostics");
const sourcePaths = [
  "tests/performance/vben-lsp-incremental.test.ts",
  "tests/performance/support/incremental-metrics.ts",
  "tests/_helpers/realworld-patch.ts",
  "tests/_fixtures/vue-ecosystem-fixtures.json",
  "tests/differential/build-receipt.ts",
  "tests/tooling/lsp-startup-local-interactive.test.ts",
  "tests/tooling/native-config-preload-observations.test.ts",
  "tests/tooling/native-config-compile-cache.test.ts",
  "tests/tooling/config-host-boundary.test.ts",
  "tests/tooling/support/davinci-host-imports.ts",
  "crates/vize/tests/lsp_symlink_workspace_staging_cli.rs",
  "crates/vize/tests/lsp_completion_sibling_overlay_cli.rs",
  "crates/vize/tests/lsp_component_attribute_authority_cli.rs",
  "crates/vize/tests/lsp_component_attribute_authority_cli/library_refusal.rs",
  "crates/vize/tests/lsp_component_attribute_authority_cli/library_refusal/safe_update.rs",
  "crates/vize/tests/lsp_component_attribute_authority_cli/native_probe.rs",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/Toggle.vue.txt",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/App.vue.txt",
  "crates/vize_canon/src/corsa_bridge/source_catalog.rs",
  "crates/vize_canon/src/corsa_bridge/vue_document.rs",
  "crates/vize_canon/src/corsa_bridge/vue_document/build/tests.rs",
  "crates/vize_canon/src/corsa_bridge/vue_document/build/tests/native_union.rs",
  "crates/vize_canon/src/batch/virtual_project/materialize_delta.rs",
  "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache/catalog.rs",
  "crates/vize/tests/lsp_symlink_workspace_staging_cli/fixture.rs",
  "crates/vize_canon/src/batch/virtual_project/materialize.rs",
  "crates/vize_canon/src/batch/virtual_project/materialize/alias_transition_tests.rs",
  "crates/vize_canon/src/batch/virtual_project/tests/workspace_package_dependency_shadows/pnpm_workspace_routes/lifecycle/filesystem.rs",
  "crates/vize_canon/src/batch/virtual_project/tests/workspace_package_dependency_shadows/pnpm_workspace_routes/lifecycle.rs",
  "crates/vize_canon/src/batch/virtual_project/tests/workspace_package_dependency_shadows/pnpm_workspace_routes/lifecycle/cleanup.rs",
  "crates/vize_canon/src/corsa_bridge/vue_document_package_tests/symlink_workspace.rs",
  "crates/vize_canon/src/batch/virtual_project/tsconfig_gen.rs",
  "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache.rs",
  "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache/tests.rs",
  "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/prepare.rs",
  "crates/vize_maestro/src/server/initial_diagnostics.rs",
  "crates/vize_maestro/src/server/initial_diagnostics/open_importer_tests.rs",
  "crates/vize_maestro/src/server/initial_diagnostics/tests.rs",
  "crates/vize_maestro/src/ide.rs",
  "crates/vize_maestro/src/server/initial_diagnostics/preparation.rs",
  "crates/vize_maestro/src/server/state/corsa_requests.rs",
  "crates/vize_maestro/src/server/native_requests.rs",
  "crates/vize_maestro/src/server/native_requests/tests.rs",
  "tests/tooling/lsp-config-diagnostics-workflow.test.ts",
  "crates/vize/tests/support/lsp_process.rs",
  "crates/vize_maestro/src/server/handlers.rs",
  "crates/vize_maestro/src/server/handlers/typed.rs",
  ".github/workflows/lsp-config-diagnostics.yml",
  "tools/support/compat/github/run-lsp-config-diagnostics.ts",
  "Cargo.lock",
  "crates/vize_canon/src/batch.rs",
  "crates/vize_canon/src/batch/virtual_project.rs",
  "crates/vize_canon/src/batch/virtual_project/tsconfig_gen/compiler_inputs.rs",
  "crates/vize_canon/src/batch/virtual_project/tsconfig_gen/compiler_options.rs",
  "crates/vize_canon/src/batch/virtual_project/tsconfig_gen/compiler_snapshot.rs",
  "crates/vize_canon/src/batch/virtual_project/tsconfig_gen/compiler_snapshot/tests.rs",
  "crates/vize_canon/src/lib.rs",
  "crates/vize_canon/src/package_route/stamp.rs",
  "crates/vize_canon/tests/lsp_import_resolution/compiler_snapshot.rs",
  "crates/vize_carton/Cargo.toml",
  "crates/vize_carton/src/config.rs",
  "crates/vize_carton/src/config/loader.rs",
  "crates/vize_carton/src/config/loader/checked.rs",
  "crates/vize_carton/src/config/loader/compiler_keys.rs",
  "crates/vize_carton/src/config/loader/discovery.rs",
  "crates/vize_carton/src/config/loader/experimental_tests.rs",
  "crates/vize_carton/src/config/loader/extends_tests.rs",
  "crates/vize_carton/src/config/loader/failure_identity.rs",
  "crates/vize_carton/src/config/loader/formatter.rs",
  "crates/vize_carton/src/config/loader/js.rs",
  "crates/vize_carton/src/config/loader/js_observation.rs",
  "crates/vize_carton/src/config/loader/jsx.rs",
  "crates/vize_carton/src/config/loader/language_server.rs",
  "crates/vize_carton/src/config/loader/legacy_dialect_tests.rs",
  "crates/vize_carton/src/config/loader/library.rs",
  "crates/vize_carton/src/config/loader/lint_features.rs",
  "crates/vize_carton/src/config/loader/lsp_snapshot.rs",
  "crates/vize_carton/src/config/loader/module-runtime.d.mts",
  "crates/vize_carton/src/config/loader/module-runtime.mjs",
  "crates/vize_carton/src/config/loader/parse.rs",
  "crates/vize_carton/src/config/loader/pkl.rs",
  "crates/vize_carton/src/config/loader/project_settings_tests.rs",
  "crates/vize_carton/src/config/loader/snapshots/vize_carton__config__loader__tests__validate_explicit_config_path_malformed_errors.snap",
  "crates/vize_carton/src/config/loader/tests.rs",
  "crates/vize_carton/src/config/loader/tests/linter_plan_tests.rs",
  "crates/vize_carton/src/config/loader/vapor.rs",
  "crates/vize_carton/src/config/loader/vite-runtime.d.mts",
  "crates/vize_carton/src/config/loader/vite-runtime.mjs",
  "crates/vize_maestro/src/server.rs",
  "crates/vize_maestro/src/server/diagnostic_publishing/configuration_failure_tests.rs",
  "crates/vize_maestro/src/server/helpers.rs",
  "crates/vize_maestro/src/server/project_contexts.rs",
  "crates/vize_maestro/src/server/project_contexts/operation_admission_tests.rs",
  "crates/vize_maestro/src/server/state/config.rs",
  "crates/vize_maestro/src/server/state/config/host.rs",
  "crates/vize_maestro/src/server/state/config/project.rs",
  "crates/vize_maestro/src/server/state/corsa.rs",
  "crates/vize_maestro/src/server/state.rs",
  "crates/vize_maestro/src/server/state/project_contexts.rs",
  "crates/vize_maestro/src/server/state/project_contexts/config_reload.rs",
  "crates/vize_maestro/src/server/state/project_contexts/current.rs",
  "crates/vize_maestro/src/server/state/project_contexts/current/tests.rs",
  "crates/vize_maestro/src/server/state/project_contexts/freshness.rs",
  "crates/vize_maestro/src/server/state/project_contexts/freshness_tests.rs",
  "crates/vize_maestro/src/server/state/project_contexts/lock_order_tests.rs",
  "crates/vize_maestro/src/server/state/project_contexts/references.rs",
  "crates/vize_maestro/src/server/state/project_contexts/registry.rs",
  "crates/vize_maestro/src/server/state/project_contexts/topology.rs",
  "crates/vize_maestro/src/server/state/project_contexts/topology_tests.rs",
  "crates/vize_maestro/src/server/state/project_contexts/watched.rs",
  "crates/vize_maestro/src/server/state/workspace_folders/tests.rs",
  "crates/vize_maestro/src/server/workspace_files.rs",
  "crates/vize_maestro/src/server/workspace_files/config_watchers.rs",
  "crates/vize_maestro/src/server/workspace_files/config_watchers/tests/lifecycle.rs",
  "crates/vize_maestro/src/server/workspace_files/config_watchers/tests/protocol.rs",
  "crates/vize_maestro/src/server/workspace_files/config_watchers/tests.rs",
  "crates/vize_maestro/src/server/workspace_files/watched.rs",
  "crates/vize_maestro/src/server/workspace_folder_events.rs",
  "davinci/vize_l0/src/config/document.rs",
  "npm/cli/schemas/vize.config.schema.json",
  "pnpm-lock.yaml",
  "rust-toolchain.toml",
  "tests/performance/support/churn-oracle.ts",
  "tests/performance/support/lsp-oracle.ts",
  "tests/tooling/support/lsp/session.ts",
  "tests/tooling/support/vue-config-fixture.ts",
  "Cargo.toml",
  "crates/vize_canon/src/corsa_bridge.rs",
  "crates/vize_canon/src/corsa_bridge/bridge.rs",
  "crates/vize_canon/src/corsa_bridge/bridge/documents.rs",
  "crates/vize_canon/src/lsp_client.rs",
  "crates/vize_canon/src/lsp_client/diagnostics.rs",
  "crates/vize_canon/src/lsp_client/diagnostics/virtual_overlay_diagnostics.rs",
  "crates/vize_maestro/src/ide/diagnostics.rs",
  "crates/vize_maestro/src/ide/diagnostics/corsa.rs",
  "crates/vize_maestro/src/ide/diagnostics/corsa/collect.rs",
  "crates/vize_maestro/src/ide/diagnostics/native.rs",
  "crates/vize_maestro/src/ide/diagnostics/native/timeout.rs",
  "crates/vize_maestro/src/server/diagnostic_publishing.rs",
  "crates/vize_maestro/src/server/importers/canonical.rs",
  "crates/vize_canon/src/corsa_bridge/bridge/initial_diagnostics.rs",
  "crates/vize_canon/src/lsp_client/initial_diagnostics.rs",
  "crates/vize_maestro/src/ide/diagnostics/corsa/initial.rs",
  "crates/vize_maestro/src/server/initial_diagnostics/api_read.rs",
];

function git(...args: string[]): string {
  const result = spawnSync("git", args, { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

export function retainNativeLog(source: string, destination: string): boolean {
  if (!fs.existsSync(source)) return false;
  // The native server logs at its repository cwd, outside the mutable fixture
  // directory removed by the unchanged test. Preserve its complete raw bytes.
  fs.copyFileSync(source, destination);
  return true;
}

export function retainMeasuredProducer(binary: string, receiptPath: string, destination: string) {
  const retainedBinary = "measured-vize";
  const retainedBuildReceipt = "measured-vize.differential-build.json";
  const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
  const bytes = fs.readFileSync(binary);
  assert.equal(sha256(bytes), receipt.binarySha256);
  fs.writeFileSync(path.join(destination, retainedBinary), bytes);
  fs.copyFileSync(receiptPath, path.join(destination, retainedBuildReceipt));
  assert.equal(
    sha256(fs.readFileSync(path.join(destination, retainedBinary))),
    receipt.binarySha256,
  );
  return { retainedBinary, retainedBuildReceipt };
}

export function requireRustTestsExecuted(output: string): void {
  assert.match(
    output,
    /^running [1-9][0-9]* tests?$/m,
    "a real nonzero Rust test count is required",
  );
}

export function retainIntegrationProducer(log: string, destination: string, name = "symlink-test") {
  const output = fs.readFileSync(log, "utf8");
  requireRustTestsExecuted(output);
  const artifacts = output.split("\n").flatMap((line) => {
    try {
      const message = JSON.parse(line);
      return message.reason === "compiler-artifact" &&
        message.target?.name === "vize" &&
        message.target.kind?.includes("bin") &&
        typeof message.executable === "string"
        ? [message]
        : [];
    } catch {
      return [];
    }
  });
  assert.equal(artifacts.length, 1, "Cargo must identify exactly one real test CLI producer");
  const producer = artifacts[0];
  const bytes = fs.readFileSync(producer.executable);
  const metadata = {
    schema: `vize.${name}-producer`,
    sourceRevision: git("rev-parse", "HEAD"),
    executable: producer.executable,
    binarySha256: sha256(bytes),
    cargoArtifact: producer,
  };
  fs.writeFileSync(path.join(destination, `${name}-vize`), bytes);
  fs.writeFileSync(
    path.join(destination, `${name}-producer.json`),
    `${JSON.stringify(metadata, null, 2)}\n`,
  );
  return metadata;
}

export function runDiagnostics(): number {
  fs.mkdirSync(directory, { recursive: true });
  const identity = expectedBuildIdentity(root);
  assert.equal(identity.sourceRevision, process.env.EXPECTED_SOURCE);
  assert.equal(process.env.VIZE_LSP_BIN, identity.binaryPath);
  assert.equal(process.env.RAYON_NUM_THREADS, "4");
  assert.equal(process.env.VIZE_PERF_BUDGET_SCALE, "1");
  assert.equal(process.env.VIZE_TRACE_EDITOR_PREPARATION, "1");
  const receiptPath = `${identity.binaryPath}.differential-build.json`;
  const receipt = JSON.parse(fs.readFileSync(path.join(root, receiptPath), "utf8"));
  validateBuildReceipt(receipt, identity, LEGACY_BUILD_RECIPE);
  // Later integration tests can relink this path with Cargo dev dependencies.
  // Retain the measured producer before any post-measurement Cargo command.
  const retainedProducer = retainMeasuredProducer(
    path.join(root, identity.binaryPath),
    path.join(root, receiptPath),
    directory,
  );
  const registry = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
  );
  const projects = registry.projects.filter(
    (project: { id: string }) => project.id === "vue-vben-admin",
  );
  assert.equal(projects.length, 1);
  const fixture = projects[0];
  const gitlink = git("ls-tree", "HEAD", "--", fixture.fixturePath).split(/\s+/);
  assert.equal(gitlink[0], "160000");
  assert.equal(gitlink[2], fixture.revision);
  assert.equal(git("-C", fixture.fixturePath, "rev-parse", "HEAD"), fixture.revision);
  const command = [
    process.execPath,
    "--test",
    "--test-concurrency=1",
    "tests/performance/vben-lsp-incremental.test.ts",
  ];
  const nativeLog = path.join(root, "node_modules/.vize/lsp.log");
  const nativeLogBefore = fs.existsSync(nativeLog) ? fs.readFileSync(nativeLog) : Buffer.alloc(0);
  const sources = sourcePaths.map((relative) => {
    const bytes = fs.readFileSync(path.join(root, relative));
    const destination = path.join(directory, "source", relative);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, bytes);
    return { path: relative, sha256: sha256(bytes) };
  });
  const metadata = {
    schema: "vize.lsp-config-diagnostics",
    sourceRevision: identity.sourceRevision,
    sourceTree: git("rev-parse", "HEAD^{tree}"),
    fixture: { path: fixture.fixturePath, revision: fixture.revision },
    buildReceipt: receipt,
    ...retainedProducer,
    command,
    cwd: root,
    runtime: { node: process.version, executable: process.execPath },
    environment: Object.fromEntries(
      [
        "RAYON_NUM_THREADS",
        "VIZE_PERF_BUDGET_SCALE",
        "VIZE_TRACE_EDITOR_PREPARATION",
        "VIZE_LSP_BIN",
      ].map((name) => [name, process.env[name]]),
    ),
    sources,
    nativeLogBefore: { bytes: nativeLogBefore.length, sha256: sha256(nativeLogBefore) },
    scope:
      "One instrumented original Vben performance case only; no full Check or paired performance qualification.",
  };
  fs.writeFileSync(path.join(directory, "command.json"), `${JSON.stringify(metadata, null, 2)}\n`);
  const stdout = fs.openSync(path.join(directory, "test.stdout"), "w");
  const stderr = fs.openSync(path.join(directory, "test.stderr"), "w");
  let result;
  try {
    result = spawnSync(command[0], command.slice(1), {
      cwd: root,
      env: process.env,
      stdio: ["ignore", stdout, stderr],
    });
  } finally {
    fs.closeSync(stdout);
    fs.closeSync(stderr);
    retainNativeLog(nativeLog, path.join(directory, "native-lsp.log"));
  }
  const output = ["test.stdout", "test.stderr", "native-lsp.log"].flatMap((name) => {
    const source = path.join(directory, name);
    return fs.existsSync(source) ? [{ path: name, sha256: sha256(fs.readFileSync(source)) }] : [];
  });
  fs.writeFileSync(
    path.join(directory, "result.json"),
    `${JSON.stringify({ exitStatus: result.status, signal: result.signal, processError: result.error?.message ?? null, output }, null, 2)}\n`,
  );
  process.stdout.write(fs.readFileSync(path.join(directory, "test.stdout")));
  process.stderr.write(fs.readFileSync(path.join(directory, "test.stderr")));
  return result.status ?? 1;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv[2] === "--require-rust-tests") {
    requireRustTestsExecuted(fs.readFileSync(process.argv[3], "utf8"));
  } else if (process.argv[2] === "--retain-symlink-producer") {
    assert.equal(git("rev-parse", "HEAD"), process.env.EXPECTED_SOURCE);
    retainIntegrationProducer(path.join(directory, "symlink.stdout"), directory);
  } else if (process.argv[2] === "--retain-completion-producer") {
    assert.equal(git("rev-parse", "HEAD"), process.env.EXPECTED_SOURCE);
    retainIntegrationProducer(
      path.join(directory, "completion.stdout"),
      directory,
      "completion-test",
    );
  } else if (process.argv[2] === "--retain-rename-producer") {
    assert.equal(git("rev-parse", "HEAD"), process.env.EXPECTED_SOURCE);
    retainIntegrationProducer(path.join(directory, "rename.stdout"), directory, "rename-test");
  } else {
    process.exitCode = runDiagnostics();
  }
}
