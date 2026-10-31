/** Source-only path changes must keep the real native/runtime regression selected. */
import assert from "node:assert/strict";
import fs from "node:fs";
import { matchesGlob } from "node:path";
import { test } from "node:test";
import { parse, stringify } from "yaml";
const workflow = fs.readFileSync(
  new URL("../../.github/workflows/jsx-authored-native-modules.yml", import.meta.url),
  "utf8",
);
type Step = {
  name?: string;
  if?: string;
  run?: string;
  env?: Record<string, string>;
  with?: Record<string, unknown>;
};
type Workflow = {
  on: { pull_request: { paths: string[] } };
  jobs: Record<string, { steps: Step[] }>;
};
const document = parse(workflow) as Workflow;
const paths = document.on.pull_request.paths;
function step(steps: Step[], name: string) {
  const found = steps.filter((item) => item.name === name);
  assert.equal(found.length, 1, name);
  return found[0]!;
}
function requirePairedPackets(document: Workflow) {
  const steps = document.jobs["authored-modules"]!.steps;
  const original = step(
    steps,
    "Original VDOM child values and parent block ownership before and after",
  );
  assert.match(
    original.run!,
    /^git\s+worktree\s+add\s+--detach\s+"?\$(?:baseline|\{baseline\})"?\s+9c594186bae97cbc4b4c5441d9165b3354ccbec0$/m,
  );
  assert.doesNotMatch(original.run!, /\$(?:BASE_SHA|\{BASE_SHA\})/);
  const review = step(steps, "Genuine pre-review slot and sequence producer packets");
  assert.match(
    review.run!,
    /^git\s+worktree\s+add\s+--detach\s+"?\$(?:baseline|\{baseline\})"?\s+18ebd0556f55fc1fff4b7a993efa752bdabc7232$/m,
  );
  const browser = step(
    steps,
    "Slot updates and sequence effects before and after in genuine Chromium",
  );
  assert.equal(browser.if, "always()");
  assert.match(
    browser.run!,
    /vdom-child-review-browser\.ts\s+"?\$(?:RUNNER_TEMP|\{RUNNER_TEMP\})\/jsx-authored-native\/vdom-review\/base\.json"?\s+"?\$(?:RUNNER_TEMP|\{RUNNER_TEMP\})\/jsx-authored-native\/vdom-review\/head\.json"?/,
  );
}
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
  requirePairedPackets(document);
});
test("equivalent YAML indentation and scalar quoting retain the same workflow checks", () => {
  const reserialized = parse(
    stringify(document, { indent: 4, defaultStringType: "QUOTE_DOUBLE" }),
  ) as Workflow;
  assert.deepEqual(reserialized, document);
  requirePairedPackets(reserialized);
});
test("paired-packet guards reject moving old baselines and omitted browser execution", () => {
  for (const mutate of [
    (steps: Step[]) => {
      const original = step(
        steps,
        "Original VDOM child values and parent block ownership before and after",
      );
      original.run = original.run!.replace("9c594186bae97cbc4b4c5441d9165b3354ccbec0", "$BASE_SHA");
    },
    (steps: Step[]) => {
      step(steps, "Slot updates and sequence effects before and after in genuine Chromium").if =
        "success()";
    },
    (steps: Step[]) => {
      const browser = step(
        steps,
        "Slot updates and sequence effects before and after in genuine Chromium",
      );
      browser.run = browser.run!.replace("/vdom-review/head.json", "/vdom-review/base.json");
    },
  ]) {
    const changed = structuredClone(document);
    mutate(changed.jobs["authored-modules"]!.steps);
    assert.throws(() => requirePairedPackets(changed));
  }
});

test("Rspack complete captures retain the original group environment and every actual fallback output", () => {
  const source = fs.readFileSync(
    new URL("../../.github/workflows/pr-js-checks.yml", import.meta.url),
    "utf8",
  );
  const steps = (parse(source) as Workflow).jobs["pr-js-package-tests"]!.steps;
  const group = step(steps, "Test JS package group");
  assert.equal(Object.hasOwn(group.env ?? {}, "VIZE_RSPACK_JSX_CAPTURE"), false);
  const capture = step(steps, "Preserve complete Rspack JSX producer and bundle");
  const capturedPaths = String(capture.with?.path)
    .split("\n")
    .map((path) => path.trim());
  for (const path of [
    "jsx-vapor/authored-vapor-bundle.json",
    "jsx-vdom/native-vdom-bundle.json",
    "jsx-vdom-module/native-vdom-module.json",
    "jsx-vdom-map/native-vdom-map.json",
  ])
    assert.ok(capturedPaths.includes("target/vize-tests/test-output/rspack-vize-plugin/" + path));
  assert.match(capture.if!, /!cancelled\(\)/);
  assert.equal(capture.with?.["if-no-files-found"], "error");
});
