/** Source-only path changes must keep the real native/runtime regression selected. */
import assert from "node:assert/strict";
import fs from "node:fs";
import { matchesGlob } from "node:path";
import { test } from "node:test";
const workflow = fs.readFileSync(
  new URL("../../.github/workflows/jsx-authored-native-modules.yml", import.meta.url),
  "utf8",
);
const paths = workflow
  .split("    paths:\n")[1]!
  .split("\npermissions:")[0]!
  .split("\n")
  .map((line) => line.match(/^      - "([^"]+)"$/)?.[1])
  .filter((p): p is string => p !== undefined);
const selected = (file: string) => paths.some((pattern) => matchesGlob(file, pattern));
test("all raw-child producers, exhaustive visitors and the three registered input controls select genuine regression", () => {
  for (const file of [
    "crates/vize_atelier_jsx/src/lower/child.rs",
    "crates/vize_atelier_core/src/codegen/slots/detect.rs",
    "crates/vize_relief/src/relief/expressions.rs",
    "davinci/vize_l1_to_l2/src/emit/children.rs",
    "davinci/vize_l2/src/op/text.rs",
    "davinci/vize_l2_to_l3/src/lower.rs",
    "davinci/vize_l3/src/dom/build.rs",
    "davinci/vize_l3_to_l4/src/lower.rs",
    "davinci/vize_l4/src/targets/dom.rs",
    "crates/vize_canon/src/projection/expression.rs",
    "crates/vize_canon/tests/jsx_child_expression_control.rs",
    "crates/vize_patina/src/markup/visitor.rs",
    "crates/vize_patina/tests/jsx_child_expression_control.rs",
    "crates/vize_patina/tests/main.rs",
    "tests/tooling/jsx-vdom-snapshot-custody.test.ts",
    "tests/tooling/jsx-vdom-slot-snapshot-custody.test.ts",
    "tests/tooling/jsx-vdom-consumer-snapshot-custody.test.ts",
    "npm/builder/rspack/src/jsx.test.ts",
    "npm/builder/rspack/src/shared/jsx-compile.test.ts",
    "npm/builder/rspack/src/test/jsx-vdom-child-values.ts",
    "tests/_fixtures/differential/jsx/child-values-6890-vdom-slot-snapshot-successors.json",
    "tests/_fixtures/differential/jsx/child-values-6890-vdom-snapshot-successors.json",
    "tests/_fixtures/differential/compiler/jsx-vdom-template-control-6890/Layout.vue.txt",
    "tests/_fixtures/differential/linter/jsx-child-expression-control-6890/Layout.vue.txt",
    "tests/_fixtures/differential/typechecker/jsx-child-expression-control-6890/template.html.txt",
    "tests/tooling/jsx-vapor-oracle/vdom-child-review.ts",
    "tests/tooling/jsx-vapor-oracle/vdom-child-review-browser.ts",
    "crates/vize_atelier_jsx/tests/vdom_child_review.rs",
    "tests/tooling/jsx-authored-native-workflow-paths.test.ts",
  ])
    assert.equal(selected(file), true, file);
});
test("unrelated documentation and formatter inputs do not select this lane", () => {
  for (const file of [
    "README.md",
    "crates/vize_glyph/src/lib.rs",
    "tests/_fixtures/formatter/ordinary.vue",
  ])
    assert.equal(selected(file), false, file);
});
test("both genuine pre-review and head browser packets remain mandatory", () => {
  assert.match(
    workflow,
    /git worktree add --detach "\$baseline" 18ebd0556f55fc1fff4b7a993efa752bdabc7232/,
  );
  assert.match(
    workflow,
    /Slot updates and sequence effects before and after in genuine Chromium\n        if: always\(\)/,
  );
  assert.match(
    workflow,
    /vdom-child-review-browser\.ts "\$RUNNER_TEMP\/jsx-authored-native\/vdom-review\/base\.json" "\$RUNNER_TEMP\/jsx-authored-native\/vdom-review\/head\.json"/,
  );
});

test("Rspack complete captures retain the original group environment and every actual fallback output", () => {
  const source = fs.readFileSync(
    new URL("../../.github/workflows/pr-js-checks.yml", import.meta.url),
    "utf8",
  );
  const group = source
    .split("      - name: Test JS package group\n")[1]!
    .split("\n      - name:")[0]!;
  assert.doesNotMatch(group, /VIZE_RSPACK_JSX_CAPTURE/);
  const capture = source
    .split("      - name: Preserve complete Rspack JSX producer and bundle\n")[1]!
    .split("\n      - name:")[0]!;
  for (const path of [
    "jsx-vapor/authored-vapor-bundle.json",
    "jsx-vdom/native-vdom-bundle.json",
    "jsx-vdom-module/native-vdom-module.json",
    "jsx-vdom-map/native-vdom-map.json",
  ])
    assert.ok(capture.includes("target/vize-tests/test-output/rspack-vize-plugin/" + path));
  assert.match(capture, /!cancelled\(\)/);
  assert.match(capture, /if-no-files-found: error/);
});
