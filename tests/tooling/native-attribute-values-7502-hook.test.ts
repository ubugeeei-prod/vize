import assert from "node:assert/strict";
import { readFileSync, mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { nativeAttributeValues7502CaptureRequired } from "../../tools/support/compat/github/native-attribute-values-7502-capture.mjs";
import { validateScriptlessHistory7502 } from "./support/native-attribute-values-7502-scriptless-history.ts";
import { validateVaporHistory7502 } from "./support/native-attribute-values-7502-vapor-history.ts";
import { baseline7502, hash7502 } from "./support/native-attribute-values-7502-inputs.ts";
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
    "crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_7502/reviewed_output_v2.json",
    "tests/tooling/native-attribute-values-7502-reference.test.ts",
    "tests/tooling/native-attribute-values-7502-hook.test.ts",
    "tests/tooling/support/native-attribute-values-7502-runtime.ts",
    "tests/tooling/support/native-attribute-values-7502-build.ts",
    "tests/tooling/support/native-attribute-values-7502-judge.ts",
    "tests/tooling/support/native-attribute-values-7502-source.ts",
    "tests/tooling/support/native-attribute-values-7502-history.ts",
    "tests/tooling/support/native-attribute-values-7502-history-protocol.ts",
    "tests/tooling/support/native-attribute-values-7502-scriptless-history.ts",
    "tests/tooling/support/native-attribute-values-7502-vapor-history.ts",
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
    ["HISTORY_RECEIPT", "/history-receipt.json"],
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
    /set \+e[\s\S]*vp node tests\/tooling\/support\/native-attribute-values-7502-history\.ts[\s\S]*history_status=\$\?[\s\S]*vp node tests\/tooling\/support\/native-attribute-values-7502-build\.ts[\s\S]*build_status=\$\?[\s\S]*vp node tests\/tooling\/support\/native-attribute-values-7502-judge\.ts[\s\S]*judge_status=\$\?[\s\S]*set -e/,
  );
  for (const file of [
    "hosted-build.stdout.txt",
    "history.stdout.txt",
    "history.stderr.txt",
    "hosted-build.stderr.txt",
    "judge.stdout.txt",
    "judge.stderr.txt",
    "action-processes.json",
  ])
    assert(run.includes(file), file);
  assert.match(
    run,
    /if \[\[ "\$history_status" -ne 0 \|\| "\$build_status" -ne 0 \|\| "\$judge_status" -ne 0 \]\]; then exit 1; fi/,
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

// These synthetic receipt mutations provide schema evidence, never native credit.
test("the pinned whole scriptless archive rejects dropping its original class refusal", () => {
  const directory = mkdtempSync(path.join(tmpdir(), "native-scriptless-history-law-"));
  try {
    const bytes = readFileSync(
      new URL(
        "../../crates/vize_atelier_sfc/tests/fixtures/native-scriptless-ssr-output.json",
        import.meta.url,
      ),
    );
    const original = JSON.parse(bytes.toString("utf8"));
    const capture = Buffer.from(JSON.stringify(original.capture));
    const runtime = Buffer.from(JSON.stringify(original.runtime));
    writeFileSync(path.join(directory, "scriptless.capture.json"), capture);
    writeFileSync(path.join(directory, "scriptless.runtime.json"), runtime);
    const receipt = {
      sourceRevision: baseline7502.revision,
      sourceTree: baseline7502.tree,
      archiveSha256: hash7502(bytes),
      captureSha256: hash7502(capture),
      runtimeSha256: hash7502(runtime),
    };
    validateScriptlessHistory7502(receipt, directory);
    const changed = structuredClone(original.capture);
    changed.refusals = changed.refusals.filter((row: any) => row.id !== "class");
    const modified = Buffer.from(JSON.stringify(changed));
    writeFileSync(path.join(directory, "scriptless.capture.json"), modified);
    assert.throws(
      () =>
        validateScriptlessHistory7502({ ...receipt, captureSha256: hash7502(modified) }, directory),
      /whole untouched historical scriptless gate remains exact/,
    );
    assert.throws(() =>
      validateScriptlessHistory7502({ ...receipt, sourceRevision: "main" }, directory),
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("the pinned historical workspace cannot overwrite current transitive Cargo products", () => {
  const history = readFileSync(
    new URL("./support/native-attribute-values-7502-history.ts", import.meta.url),
    "utf8",
  );
  assert(history.includes('CARGO_TARGET_DIR: path.join(worktree, ".target-history")'));
  assert(!history.includes('CARGO_TARGET_DIR: path.join(root7502, "target")'));
});

test("the historical Vapor receipt rejects changed source, complete code, maps and populations", () => {
  // Synthetic data checks receipt validation, never actual historical execution.
  const directory = mkdtempSync(path.join(tmpdir(), "native-vapor-history-law-"));
  try {
    const bytes = readFileSync(
      new URL(
        "../../davinci/vize_l4/tests/fixtures/native-vapor-vue-3.6.0-rc.9.json",
        import.meta.url,
      ),
    );
    const fixture = JSON.parse(bytes.toString("utf8"));
    const rows = fixture.fixtures.map((row: any) => ({
      id: row.id,
      source: row.source,
      code: row.code,
      map: row.map,
      nodes: row.id === "empty" ? 0 : 1,
      roots: row.id === "empty" ? 0 : 1,
    }));
    const capture = Buffer.from(JSON.stringify(rows));
    writeFileSync(path.join(directory, "vapor.capture.json"), capture);
    const receipt = {
      sourceRevision: baseline7502.revision,
      sourceTree: baseline7502.tree,
      testSourceSha256: "5903289dae43f4efa359612a243976365bf0280536fd5dead194b8bc959f19b9",
      fixtureSha256: hash7502(bytes),
      captureSha256: hash7502(capture),
    };
    validateVaporHistory7502(receipt, directory);
    for (const field of [
      "sourceRevision",
      "sourceTree",
      "testSourceSha256",
      "fixtureSha256",
      "captureSha256",
    ] as const)
      assert.throws(() => validateVaporHistory7502({ ...receipt, [field]: "changed" }, directory));
    for (const mutate of [
      (rows: any[]) => rows.pop(),
      (rows: any[]) => (rows[0].code += "\n"),
      (rows: any[]) => (rows[0].source += "\n"),
      (rows: any[]) => (rows[0].map.mappings += "A"),
      (rows: any[]) => (rows[0].nodes = 1),
      (rows: any[]) => (rows[0].roots = 1),
      (rows: any[]) => (rows[1].nodes = 0),
      (rows: any[]) => (rows[1].roots = 0),
      (rows: any[]) => (rows[1].roots = 2),
    ]) {
      const changed = structuredClone(rows);
      mutate(changed);
      const modified = Buffer.from(JSON.stringify(changed));
      writeFileSync(path.join(directory, "vapor.capture.json"), modified);
      assert.throws(() =>
        validateVaporHistory7502({ ...receipt, captureSha256: hash7502(modified) }, directory),
      );
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
