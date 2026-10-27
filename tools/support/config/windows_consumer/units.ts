import assert from "node:assert/strict";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Recorder, success } from "./common.ts";
export function runLoaderUnits(source: string, evidence: string): void {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.version, "v24.22.1");
  const record = new Recorder(evidence);
  const node = record.run("actual-node", ["node", "--version"], source);
  success(node);
  assert.equal(node.stdout.trim(), "v24.22.1");
  const result = record.run(
    "source-config-loader-tests",
    [
      "cargo",
      "test",
      "--locked",
      "--profile",
      "ci",
      "-p",
      "vize_carton",
      "config::loader::",
      "--",
      "--nocapture",
    ],
    source,
    undefined,
    1_200_000,
  );
  success(result);
  for (const name of [
    "large_evaluator_is_a_private_file_instead_of_a_command_argument",
    "file_execution_preserves_unicode_paths_cache_marker_and_cwd",
    "file_runtime_keeps_stdin_available_for_the_real_observation_handshake",
    "temporary_module_is_removed_after_success_or_child_failure",
    "runtime_source_identity_preserves_authored_errors_across_retries",
  ])
    assert.ok(result.stdout.includes(name + " ... ok"), "required actual-Node test: " + name);
  const count = result.stdout.match(/test result: ok\. ([1-9]\d*) passed; 0 failed;/);
  assert.ok(count);
  assert.ok(Number(count[1]) >= 5 && Number(count[1]) < 500);
  record.json("required-unit-summary.json", {
    actualNode: node.stdout.trim(),
    passed: Number(count[1]),
    requiredTransportCases: 5,
    status: "pass",
  });
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [source, evidence] = process.argv.slice(2);
  assert.ok(source && evidence);
  runLoaderUnits(resolve(source), evidence);
}
