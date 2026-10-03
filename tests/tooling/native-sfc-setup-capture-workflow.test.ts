import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  nativeSetupCaptureRequired,
  toolingChecksRequired,
} from "../../tools/support/compat/github/native-setup-capture.mjs";
import { toolingShardMatrix } from "../../tools/support/compat/github/tooling-test-shards.ts";

test("actual native setup inputs qualify affected source capture while prose stays fast", () => {
  for (const path of [
    "davinci/vize_l2/src/file.rs",
    "davinci/vize_l2/src/file/build.rs",
    "davinci/vize_l2/src/lang/js/file/walk/annotations.rs",
    "davinci/vize_l4/src/module/setup.rs",
    "davinci/vize_l4/src/targets/dom/vue.rs",
    "crates/vize_atelier_sfc/src/native.rs",
    "crates/vize_atelier_sfc/src/native/setup/tests/annotations.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_sfc_ts_annotation_setup_vue_3_5_35.json",
    "tests/tooling/native-sfc-primitive-setup-reference.test.ts",
    "tests/tooling/support/native-sfc-setup-reference.ts",
    ".github/actions/test-native-js-setup/action.yml",
    "vendor/oxc_parser/src/lexer/string.rs",
  ])
    assert(nativeSetupCaptureRequired([path]), path);
  for (const path of [
    "docs/davinci/plan/completion-2026-10-03.md",
    "docs/davinci/decisions/2026-09-27-level-restructure.md",
    "README.md",
    "crates/vize_canon/src/native.rs",
    "davinci/vize_l2/tests/file_vue_setup.rs",
    "npm/ui/src/index.ts",
  ])
    assert.equal(nativeSetupCaptureRequired([path]), false, path);
  assert.equal(nativeSetupCaptureRequired([]), false);
});

test("a capture-only source change retains an actual first worker with an empty ordinary selector", () => {
  const plan = { tier: "pr" as const, tests: [] };
  assert(toolingChecksRequired(plan, ["davinci/vize_l2/src/lang/js/file/setup.rs"]));
  assert.deepEqual(toolingShardMatrix(plan), { include: [{ index: 1, total: 1 }] });
  assert.equal(toolingChecksRequired(plan, ["docs/davinci/plan/completion-2026-10-03.md"]), false);
  assert(toolingChecksRequired({ tier: "pr", tests: ["tests/tooling/example.test.ts"] }, []));
});

test("the existing first-shard hook keeps unconditional protected capture and exact source artifacts", () => {
  const workflow = fs.readFileSync(
    new URL("../../.github/workflows/pr-source-checks.yml", import.meta.url),
    "utf8",
  );
  const step = workflow
    .split("- name: Test source-bound native JS setup modules and runtime")[1]
    ?.split("- name:")[0];
  assert(step);
  assert.match(step, /matrix\.index == 1/);
  assert.match(step, /github\.event_name == 'merge_group' \|\|/);
  assert.match(
    step,
    /github\.event_name == 'pull_request' && needs\.pr-source-plan\.outputs\.native-setup-capture == 'true'/,
  );
  assert.match(step, /uses: \.\/\.github\/actions\/test-native-js-setup/);
  const action = fs.readFileSync(
    new URL("../../.github/actions/test-native-js-setup/action.yml", import.meta.url),
    "utf8",
  );
  assert.match(action, /cargo test --locked --profile ci -p vize_atelier_sfc/);
  assert.match(action, /three_source_owned_annotation_modules_and_maps_are_captured -- --exact/);
  assert.match(action, /native-js-setup-\$\{\{ github\.sha \}\}-\$\{\{ github\.run_attempt \}\}/);
  assert.match(action, /if-no-files-found: error/);
  assert.doesNotMatch(step, /continue-on-error|hashFiles|existsSync/);
});
