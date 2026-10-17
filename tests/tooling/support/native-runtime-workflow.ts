import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { parse } from "yaml";

/** Every selected native lane must execute its action and reach the required report. */
export function assertNativeRuntimeHook(lane: string, action: string) {
  const workflow = parse(
    readFileSync(
      new URL("../../../.github/workflows/pr-source-checks.yml", import.meta.url),
      "utf8",
    ),
  );
  assert.equal(
    workflow.jobs["pr-source-plan"].outputs["native-runtime-matrix"],
    "${{ steps.plan.outputs.native-runtime-matrix }}",
  );
  const runtime = workflow.jobs["pr-native-runtime"];
  assert.equal(runtime.needs, "pr-source-plan");
  assert.equal(runtime.if, undefined);
  assert.equal(runtime["continue-on-error"], undefined);
  assert.equal(
    runtime.strategy.matrix,
    "${{ fromJSON(needs.pr-source-plan.outputs.native-runtime-matrix) }}",
  );
  const hooks = runtime.steps.filter(
    (step: { uses?: string }) => step.uses === `./.github/actions/${action}`,
  );
  assert.equal(hooks.length, 1, action);
  assert.equal(hooks[0].if, `\${{ matrix.lane == '${lane}' }}`, action);
  assert.equal(hooks[0]["continue-on-error"], undefined, action);
  const report = workflow.jobs["source-report"];
  assert(report.needs.includes("pr-native-runtime"));
  assert.equal(
    report.if,
    "${{ always() && github.event_name == 'merge_group' && !contains(needs.*.result, 'cancelled') }}",
  );
  assert.equal(report["continue-on-error"], undefined);
  const gate = report.steps.find(
    (step: { run?: string }) =>
      step.run === "node tools/support/compat/github/canonical-corpus-selection.mjs --check",
  );
  assert(gate);
  assert.equal(gate.env.NEEDS_JSON, "${{ toJSON(needs) }}");
  assert.equal(gate.if, undefined);
  assert.equal(gate["continue-on-error"], undefined);
}
