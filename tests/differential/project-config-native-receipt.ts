import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { stripVTControlCharacters } from "node:util";
import { BUILD_RECIPE, expectedBuildIdentity, validateBuildReceipt } from "./build-receipt.ts";
import { sha256 } from "./sha256.ts";

// Each existing lifecycle law owns one process; the original formatter law
// additionally owns both its enabled and explicit-false processes.
const nestedCases = [
  "nested_settings_are_stable_under_concurrent_requests_and_watch_reload",
  "explicit_initialization_flags_win_over_each_nested_project",
  "native::nested_native_diagnostics_use_their_own_relative_tsconfig_and_alias_types",
  "native::cross_package_unsaved_dependency_edit_and_close_refresh_the_importer",
  "workspace_operations::workspace_symbols_and_alias_file_edits_preserve_package_flags_after_primary_reload",
  "workspace_operations::explicit_bulk_provider_disables_survive_primary_reload_and_package_defaults",
  "workspace_operations::cross_package_file_rename_delete_and_create_refresh_native_buffers_after_primary_reload",
  "executor::delayed_cold_package_import_does_not_block_warm_sibling_completion",
  "executor::shutdown_retires_pending_config_without_waiting_for_node_import",
  "executor::rename_into_cold_package_keeps_later_destination_changes",
  "ignores::vite_root_global_ignores_preserve_negation_escaping_and_watch_diagnostics",
  "ignores::ignored_open_sources_keep_native_types_while_lint_exclusions_and_negation_apply",
  "marker_lifecycle::newly_created_and_removed_nested_config_refreshes_without_document_requests",
].map((name) => `nested_lsp::${name}`);
const expectedCounts = new Map([
  ...nestedCases.map((name): [string, number] => [name, 1]),
  ["lsp::lsp_initializes_project_formatting_without_options_and_respects_false", 2],
  ["lsp::active_native_typecheck_advertises_fresh_jsx_without_an_opt_in_flag", 1],
]);
const buildArgv = ["cargo", "build", "--locked", "--profile", "ci", "-p", "vize"];
const testArgv = [
  "cargo",
  "test",
  "--locked",
  "--profile",
  "ci",
  "-p",
  "vize",
  "--test",
  "project_config_cli",
  "--",
  "--nocapture",
];

const root = process.cwd();
const destination = path.join(root, "target/project-config-native");
const json = (file: string) => JSON.parse(fs.readFileSync(file, "utf8"));
const identity = expectedBuildIdentity(root);
assert.equal(identity.sourceRevision, process.env.SOURCE_SHA);
assert.equal(process.env.VIZE_TEST_REQUIRE_TSGO, "1");
assert.equal(process.env.VIZE_TEST_DISABLE_TSGO, undefined);
// Recompute the executable after cargo test: its hash must still equal the
// receipt written immediately after the authenticated production build.
const build = json(path.join(destination, "build.json"));
validateBuildReceipt(build, identity);
const workflow = fs.readFileSync(path.join(root, ".github/workflows/project-config-native.yml"));
for (const argv of [buildArgv, testArgv]) {
  assert.ok(workflow.toString().includes(argv.join(" ")), "declared locked recipe is present");
}
fs.writeFileSync(path.join(destination, "workflow.yml"), workflow);

const logBytes = fs.readFileSync(path.join(destination, "tests.log"));
const log = stripVTControlCharacters(logBytes.toString());
assert.match(log, /test result: ok\. \d+ passed; 0 failed; 0 ignored;/);
for (const name of expectedCounts.keys()) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  assert.match(log, new RegExp(`^test ${escaped} \\.\\.\\. ok$`, "m"));
}
const seen = new Map<string, number>();
const processes = fs
  .readdirSync(path.join(destination, "processes"))
  .sort()
  .map((directory) => {
    const capture = path.join(destination, "processes", directory);
    const metadata = json(path.join(capture, "process.json"));
    const terminal = json(path.join(capture, "terminal.json"));
    assert.equal(metadata.sourceSha, identity.sourceRevision);
    assert.equal(metadata.executable, fs.realpathSync(path.join(root, identity.binaryPath)));
    assert.equal(metadata.executableSha256, identity.binarySha256);
    assert.deepEqual(metadata.arguments, ["lsp"]);
    assert.ok(
      expectedCounts.has(metadata.testName),
      "captured process belongs to an original case",
    );
    seen.set(metadata.testName, (seen.get(metadata.testName) ?? 0) + 1);
    assert.equal(metadata.requiredCorsa, true);
    assert.equal(metadata.disabledNative, false);
    assert.deepEqual(metadata.versionProbe, {
      arguments: ["--version"],
      success: true,
      exitCode: 0,
      stdout: [...Buffer.from(`${identity.cliVersion}\n`)],
      stderr: [],
    });
    assert.equal(terminal.success, true);
    assert.equal(terminal.exitCode, 0);
    assert.equal(terminal.stdoutReaderJoined, true);
    assert.equal(terminal.stderrReaderJoined, true);
    const stderr = fs.readFileSync(path.join(capture, "stderr.bin"));
    assert.equal(terminal.stderrBytes, stderr.length);
    assert.equal(terminal.stderrSha256, sha256(stderr));
    const protocol = fs.readFileSync(path.join(capture, "protocol.jsonl"));
    const records = protocol
      .toString()
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    const requests = records.filter((record) => record.direction === "request");
    for (const method of ["initialize", "shutdown", "exit"]) {
      assert.equal(requests.filter((record) => record.message.method === method).length, 1);
    }
    for (const method of ["initialize", "shutdown"]) {
      const request = requests.find((record) => record.message.method === method).message;
      const responses = records.filter(
        (record) => record.direction === "response" && record.message.id === request.id,
      );
      assert.equal(responses.length, 1);
      assert.equal(responses[0].message.error, undefined);
      if (method === "initialize") assert.ok(responses[0].message.result.capabilities);
      else assert.equal(responses[0].message.result, null);
    }
    return {
      directory,
      testName: metadata.testName,
      pid: metadata.pid,
      executableSha256: metadata.executableSha256,
      metadataSha256: sha256(fs.readFileSync(path.join(capture, "process.json"))),
      terminalSha256: sha256(fs.readFileSync(path.join(capture, "terminal.json"))),
      protocolSha256: sha256(protocol),
      decodedRecordCount: records.length,
      stderrSha256: sha256(stderr),
      stderrBytes: stderr.length,
    };
  });
assert.deepEqual([...seen].sort(), [...expectedCounts].sort());
assert.equal(processes.length, 16);
fs.writeFileSync(
  path.join(destination, "custody.json"),
  `${JSON.stringify(
    {
      schema: "vize.project-config.native",
      version: 1,
      ...identity,
      canonicalBuildRecipe: BUILD_RECIPE,
      declaredWorkflowArgv: { build: buildArgv, test: testArgv },
      workflowSha256: sha256(workflow),
      testLogSha256: sha256(logBytes),
      requiredNative: true,
      originalNestedCases: nestedCases,
      processes,
    },
    null,
    2,
  )}\n`,
);
console.log(
  `Authenticated ${nestedCases.length} original lifecycle cases and ${processes.length} source-built LSP processes`,
);
