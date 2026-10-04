import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { nativeAttributeValues7502CaptureRequired } from "../../tools/support/compat/github/native-attribute-values-7502-capture.mjs";
import { toolingShardMatrix } from "../../tools/support/compat/github/tooling-test-shards.ts";

test("genuine value producers, consumers, writers and capture dependencies qualify", () => {
  for (const file of [
    "davinci/vize_l0/src/span.rs",
    "davinci/vize_l0_derive/src/lib.rs",
    "davinci/vize_l1/src/markup/native/operand/value/frame.rs",
    "davinci/vize_l1/src/markup/native/operand.rs",
    "davinci/vize_l1/src/markup/native/selected.rs",
    "davinci/vize_l1/src/parse.rs",
    "davinci/vize_l1/src/markup/entity/native/decode.rs",
    "davinci/vize_l1/src/embed/source/map.rs",
    "davinci/vize_l1/src/container/vue/descriptor/policy.rs",
    "davinci/vize_l1_to_l2/src/native_file/selected/walk.rs",
    "davinci/vize_l1_to_l2/src/native_file/observe.rs",
    "davinci/vize_l2/src/file/attribute_value.rs",
    "davinci/vize_l2/src/file/region/native/attribute_value/interruption.rs",
    "davinci/vize_l2/src/lang/js/file/native.rs",
    "davinci/vize_l2/src/artifact/builder/region.rs",
    "davinci/vize_l3/src/decision/attribute_value.rs",
    "davinci/vize_l3/src/decision/build/walk.rs",
    "davinci/vize_l3/src/decision/native.rs",
    "davinci/vize_l3/src/decision/dom/build/file/native.rs",
    "davinci/vize_l3/src/decision/ssr/build/read.rs",
    "davinci/vize_l3/src/decision/vapor/build.rs",
    "davinci/vize_l4/src/targets/dom/write.rs",
    "davinci/vize_l4/src/targets/ssr/write.rs",
    "davinci/vize_l4/src/targets/vapor/literal.rs",
    "davinci/vize_l4/src/write/source_map/vlq.rs",
    "davinci/vize_l4/src/module.rs",
    "davinci/vize_l4/src/runtime.rs",
    "crates/vize_atelier_sfc/src/native_selected.rs",
    "crates/vize_atelier_sfc/src/native_ssr.rs",
    "crates/vize_atelier_sfc/src/native_vapor.rs",
    "crates/vize_atelier_sfc/tests/native_attribute_values_7502.rs",
    "crates/vize_atelier_sfc/tests/native_attribute_values_7502/custody.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_7502/original_inputs.json",
    "crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_7502/reviewed_output.json",
    "tests/tooling/native-attribute-values-7502-reference.test.ts",
    "tests/tooling/native-attribute-values-7502-hook.test.ts",
    "tests/tooling/support/native-attribute-values-7502-runtime.ts",
    "tests/tooling/support/native-attribute-values-7502-build.ts",
    "tests/tooling/support/native-attribute-values-7502-judge.ts",
    "tests/tooling/support/native-attribute-values-7502-source.ts",
    "npm/ui/package.json",
    "npm/plugin-sdk/sandbox.js",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "Cargo.lock",
    "davinci/vize_l3/Cargo.toml",
    ".cargo/config.toml",
    "rust-toolchain.toml",
    "tools/support/compat/davinci/plugin-sandbox-image.mjs",
    "tools/support/compat/github/native-attribute-values-7502-capture.mjs",
    "tools/support/compat/github/plan-tooling-tests.mjs",
    ".github/actions/test-native-attribute-values-7502/action.yml",
    ".github/workflows/pr-source-checks.yml",
    ".github/workflows/check.yml",
  ])
    assert.equal(nativeAttributeValues7502CaptureRequired([file]), true, file);
  assert.equal(nativeAttributeValues7502CaptureRequired([]), false);
  assert.equal(nativeAttributeValues7502CaptureRequired(["README.md", "Cargo.lock"]), true);
  assert.deepEqual(toolingShardMatrix({ tier: "pr", tests: [] }), {
    include: [{ index: 1, total: 1 }],
  });
});

test("prose, other fixtures and extension near misses do not admit native capture", () => {
  for (const file of [
    "README.md",
    "docs/davinci/decisions/2026-09-27-level-restructure.md",
    "docs/davinci/plan/native-attribute-values-7502.md",
    "davinci/vize_l1/src/markup/native/operand/value/README.md",
    "davinci/vize_l2/src/file/attribute_value.rs.bak",
    "davinci/vize_l3/src/decision/attribute_values.rs.txt",
    "davinci/vize_l4/src/targets/vapor/literal.md",
    "davinci/vize_l4/src/targets/vapors/literal.rs",
    "davinci/vize_l4/src/targets/ts/mapping.rs",
    "davinci/vize_l2/tests/file_attribute_value.rs",
    "crates/vize_atelier_sfc/tests/native_attribute_values_75020.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_7502/README.md",
    "crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_75020/original_inputs.json",
    "tests/tooling/native-attribute-values-75020-reference.test.ts",
    "tests/tooling/support/native-attribute-values-7502-runtime.ts.md",
    "tests/tooling/support/native-vapor-sfc-oracle.mjs",
    "tests/differential/observer-build.ts",
    "tests/differential/harness.mjs",
    ".github/actions/test-native-attribute-values-75020/action.yml",
    "tools/support/compat/github/native-attribute-values-7502-capture.mjs.bak",
    "npm/ui/src/index.ts",
    "crates/vize_canon/src/native.rs",
  ])
    assert.equal(nativeAttributeValues7502CaptureRequired([file]), false, file);
});

test("the dedicated action preserves both failing processes and all evidence", () => {
  const action = readFileSync(
    new URL("../../.github/actions/test-native-attribute-values-7502/action.yml", import.meta.url),
    "utf8",
  );
  assert.match(
    action,
    /run: vp exec node tools\/support\/compat\/davinci\/plugin-sandbox-image\.mjs/,
  );
  for (const [key, suffix] of [
    ["CAPTURE", "/first.capture.json"],
    ["BUILD_RECEIPT", "/build-receipt.json"],
    ["EVIDENCE_DIR", ""],
  ])
    assert(
      action.includes(
        `VIZE_NATIVE_ATTRIBUTE_VALUES_7502_${key}: \${{ runner.temp }}/native-attribute-values-7502${suffix}`,
      ),
    );
  const run = action.split("      run: |\n")[1]?.split("    - name:")[0];
  assert(run);
  assert.match(run, /set -euo pipefail/);
  assert.match(
    run,
    /set \+e[\s\S]*vp node tests\/tooling\/support\/native-attribute-values-7502-build\.ts[\s\S]*build_status=\$\?[\s\S]*vp node tests\/tooling\/support\/native-attribute-values-7502-judge\.ts[\s\S]*judge_status=\$\?[\s\S]*set -e/,
  );
  for (const file of [
    "hosted-build.stdout.txt",
    "hosted-build.stderr.txt",
    "judge.stdout.txt",
    "judge.stderr.txt",
    "action-processes.json",
  ])
    assert(run.includes(file), file);
  assert.match(
    run,
    /if \[\[ "\$build_status" -ne 0 \|\| "\$judge_status" -ne 0 \]\]; then exit 1; fi/,
  );
  const upload = action.split("    - name: Upload complete native attribute")[1];
  assert(upload);
  assert.match(upload, /if: \$\{\{ always\(\) \}\}/);
  assert.match(upload, /uses: actions\/upload-artifact@[a-f0-9]{40}/);
  assert.match(upload, /path: \$\{\{ runner\.temp \}\}\/native-attribute-values-7502\n/);
  assert.match(upload, /if-no-files-found: error/);
  assert.match(
    upload,
    /native-attribute-values-7502-\$\{\{ github\.sha \}\}-\$\{\{ github\.run_attempt \}\}/,
  );
  assert.doesNotMatch(action, /continue-on-error|hashFiles|existsSync|\|\| true/);
});

// Explicit integration check: the helper-only branch intentionally has no shared
// edits. Call this after adoption or against the qualification worktree; no skip
// or missing-source fallback turns absence into a passing integration result.
export function assertNativeAttributeValues7502SharedHooks(root: string) {
  const read = (file: string) => readFileSync(path.join(root, file), "utf8");
  const planner = read("tools/support/compat/github/plan-tooling-tests.mjs");
  assert.match(
    planner,
    /import \{ nativeAttributeValues7502CaptureRequired \} from "\.\/native-attribute-values-7502-capture\.mjs"/,
  );
  assert.match(
    planner,
    /tooling=\$\{[^}]*\|\| nativeAttributeValues7502CaptureRequired\(capturePaths\)/,
  );
  assert.match(
    planner,
    /native-attribute-values-7502-capture=\$\{nativeAttributeValues7502CaptureRequired\(capturePaths\)\}/,
  );
  const pr = read(".github/workflows/pr-source-checks.yml");
  assert.match(
    pr,
    /native-attribute-values-7502-capture: \$\{\{ steps\.tooling-plan\.outputs\.native-attribute-values-7502-capture \}\}/,
  );
  const action = "uses: ./.github/actions/test-native-attribute-values-7502";
  const start = pr.lastIndexOf("      - name:", pr.indexOf(action));
  const step = pr.slice(start, pr.indexOf("      - name:", start + 1));
  assert.match(step, /matrix\.index == 1/);
  assert.match(step, /github\.event_name == 'merge_group' \|\|/);
  assert.match(
    step,
    /github\.event_name == 'pull_request' && needs\.pr-source-plan\.outputs\.native-attribute-values-7502-capture == 'true'/,
  );
  assert(step.includes(action));
  assert(pr.indexOf(action) < pr.indexOf("- name: Test selected PR tooling scripts"));
  assert(pr.indexOf(action) < pr.indexOf("- name: Test tooling scripts"));
  assert.doesNotMatch(step, /continue-on-error|hashFiles|existsSync/);
  const check = read(".github/workflows/check.yml");
  const fullStart = check.lastIndexOf("      - name:", check.indexOf(action));
  const full = check.slice(fullStart, check.indexOf("      - name:", fullStart + 1));
  assert(full.includes(action));
  assert.doesNotMatch(full, /\bif:|continue-on-error|hashFiles|existsSync/);
}
