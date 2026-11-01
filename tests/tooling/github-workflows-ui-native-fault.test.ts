import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

interface Step {
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  env?: Record<string, string>;
  with?: Record<string, string | number>;
  "continue-on-error"?: boolean;
}

interface ConformanceJob {
  steps: Step[];
  strategy: { matrix: { surface: string[] }; "fail-fast": boolean };
  "timeout-minutes": number;
  env: Record<string, string>;
}

const selected =
  "${{ inputs.run-js && matrix.surface == 'ui' && fromJSON(inputs.js-plan).conformance.ui }}";
const retained =
  "${{ !cancelled() && inputs.run-js && matrix.surface == 'ui' && fromJSON(inputs.js-plan).conformance.ui }}";
const command =
  "node .github/actions/test-js-packages-with-history/native-fault-evidence.ts --ui-conformance";
const workflow = parse(
  readFileSync(new URL("../../.github/workflows/pr-js-checks.yml", import.meta.url), "utf8"),
) as { jobs: Record<string, ConformanceJob> };
const job = workflow.jobs["pr-js-package-conformance"];
assert.ok(job);

function exactStep(steps: Step[], name: string): Step {
  const matches = steps.filter((step) => step.name === name);
  assert.equal(matches.length, 1, `Expected one ${name} step`);
  assert.ok(matches[0]);
  return matches[0];
}

function uiObservation(steps: Step[]) {
  const run = exactStep(steps, "Check UI package conformance");
  const upload = exactStep(steps, "Preserve UI conformance native fault diagnostics");
  assert.deepEqual(run, { name: "Check UI package conformance", if: selected, run: command });
  assert.deepEqual(upload, {
    name: "Preserve UI conformance native fault diagnostics",
    if: retained,
    uses: "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
    with: {
      name: "ui-native-fault-${{ github.sha }}-${{ github.run_id }}-${{ github.run_attempt }}",
      path: "${{ runner.temp }}/js-native-fault-evidence-*/**/*.json",
      "if-no-files-found": "warn",
      "retention-days": 14,
    },
  });
  const css = exactStep(steps, "Verify whole source CSS API maps with the built native image");
  assert.equal(steps.indexOf(css), steps.indexOf(run) + 1);
  assert.equal(steps.indexOf(upload), steps.indexOf(css) + 1);
}

void test("UI conformance uses one closed observer route and retains only bounded JSON on failure", () => {
  uiObservation(job.steps);
  assert.deepEqual(job.strategy.matrix.surface, ["fresco", "ui"]);
  assert.equal(job.strategy["fail-fast"], false);
  assert.equal(job["timeout-minutes"], 35);
  assert.deepEqual(job.env, {
    NODE_OPTIONS: "--disable-warning=DEP0040 --disable-warning=DEP0169",
    CARGO_BUILD_JOBS: "12",
  });
  assert.deepEqual(
    exactStep(job.steps, "Verify whole source CSS API maps with the built native image"),
    {
      name: "Verify whole source CSS API maps with the built native image",
      if: selected,
      env: { VIZE_CSS_MAP_REQUIRE_SOURCE: "1", VIZE_CSS_MAP_SOURCE_SHA: "${{ github.sha }}" },
      run: "node --test tests/native/css-compile-source-maps.mjs tests/native/css-v-bind-source-maps.mjs",
    },
  );
});

function evidenceFields(steps: Step[]): Record<string, string | number> {
  const fields = exactStep(steps, "Preserve UI conformance native fault diagnostics").with;
  assert.ok(fields);
  return fields;
}

const mutations: Array<(steps: Step[]) => void> = [
  (steps) => {
    const upload = exactStep(steps, "Preserve UI conformance native fault diagnostics");
    steps.splice(steps.indexOf(upload), 1);
    steps.splice(steps.indexOf(exactStep(steps, "Check UI package conformance")) + 1, 0, upload);
  },
  (steps) => {
    exactStep(steps, "Check UI package conformance").run = "vp run --filter './npm/ui' check";
  },
  (steps) => {
    exactStep(steps, "Check UI package conformance").env = { RUST_MIN_STACK: "16777216" };
  },
  (steps) => {
    exactStep(steps, "Check UI package conformance")["continue-on-error"] = true;
  },
  (steps) => {
    exactStep(steps, "Check UI package conformance").if = "${{ inputs.run-js }}";
  },
  (steps) => {
    exactStep(steps, "Preserve UI conformance native fault diagnostics").if = selected;
  },
  (steps) => {
    evidenceFields(steps).path += "\n**/*.node";
  },
  (steps) => {
    evidenceFields(steps)["if-no-files-found"] = "error";
  },
  (steps) => {
    steps.push(structuredClone(exactStep(steps, "Check UI package conformance")));
  },
  (steps) => {
    steps.splice(
      steps.indexOf(exactStep(steps, "Preserve UI conformance native fault diagnostics")),
      1,
    );
  },
  (steps) => {
    const upload = exactStep(steps, "Preserve UI conformance native fault diagnostics");
    steps.splice(steps.indexOf(upload), 1);
    steps.push(upload);
  },
];

void test("UI caller controls reject command, environment, status, selector and evidence regressions", () => {
  for (const mutate of mutations) {
    const steps = structuredClone(job.steps);
    mutate(steps);
    assert.throws(() => uiObservation(steps), assert.AssertionError);
  }
});
