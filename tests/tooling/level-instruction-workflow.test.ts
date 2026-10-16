import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.ts";
import { readRepoFile } from "./support/github-workflows.ts";

type Job = {
  "continue-on-error"?: boolean;
  env?: Record<string, string>;
  if?: string;
  needs?: string[];
  steps?: Array<{
    name?: string;
    if?: string;
    run?: string;
    uses?: string;
    with?: Record<string, unknown>;
    "continue-on-error"?: boolean | string;
  }>;
  uses?: string;
};
type Workflow = { name: string; on?: Record<string, unknown>; jobs?: Record<string, Job> };
const workflow = parse(readRepoFile(".github", "workflows", "check.yml")) as Workflow;
const instructionWorkflow = parse(
  readRepoFile(".github", "workflows", "level-instruction-counts.yml"),
) as Workflow;

test("instruction ceilings remain advisory in the queue and scheduled qualification", () => {
  const caller = workflow.jobs?.["instruction-counts"];
  assert.equal(caller?.uses, "./.github/workflows/level-instruction-counts.yml");
  assert.equal(caller?.["continue-on-error"], undefined);
  assert.equal(instructionWorkflow.jobs?.["instruction-counts"]?.["continue-on-error"], true);
  assert.equal(
    caller?.if,
    "${{ github.event_name == 'merge_group' || github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  assert.equal(workflow.jobs?.["test-report"]?.needs?.includes("instruction-counts"), false);
  assert.ok(Object.hasOwn(instructionWorkflow.on ?? {}, "workflow_call"));
  const gate = instructionWorkflow.jobs?.["instruction-counts"];
  assert.equal(gate?.if, undefined);
  assert.equal(
    gate?.env?.MEASURE,
    "${{ github.event_name == 'merge_group' || github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' || github.workflow == 'Level instruction counts' }}",
  );
  assert.equal(instructionWorkflow.name, "Level instruction counts");
  const verify = gate?.steps?.find(
    (step) => step.name === "Verify pinned registry and immutable base ratchet",
  );
  assert.equal(verify?.if, undefined, "a missing or invalid registry must fail every call");
  assert.match(verify?.run ?? "", /--verify-budgets/);
  assert.match(verify?.run ?? "", /sha256sum --check --strict/);
  for (const name of [
    "Measure each stage three times without Criterion sampling",
    "Enforce pinned ceilings",
  ]) {
    const step = gate?.steps?.find((candidate) => candidate.name === name);
    assert.ok(step);
    assert.equal(step.if, "env.MEASURE == 'true'");
    assert.doesNotMatch(step.run ?? "", /hashFiles|continue-on-error/);
  }
});

test("required aggregate rejects failed, cancelled and skipped instruction checks", () => {
  for (const result of ["failure", "cancelled", "skipped"]) {
    const decision = aggregateNeedsResults({ "instruction-counts": { result } });
    assert.equal(decision.exitCode, 1);
    assert.match(decision.message, new RegExp(`instruction-counts: ${result}`));
  }
});

test("original formatter calibration preserves level enforcement and retains its own full packet", () => {
  const gate = instructionWorkflow.jobs?.["instruction-counts"];
  const formatter = gate?.steps?.find(
    (step) =>
      step.name === "Measure original formatter routines three times without Criterion sampling",
  );
  assert.equal(formatter?.if, "env.MEASURE == 'true'");
  assert.match(
    formatter?.run ?? "",
    /--formatter --collect --out "\$\{RUNNER_TEMP\}\/formatter-instruction-counts"/,
  );
  assert.equal(formatter?.["continue-on-error"], undefined);
  const upload = gate?.steps?.find(
    (step) =>
      step.with?.name ===
      "formatter-instruction-counts-${{ github.sha }}-${{ github.run_attempt }}",
  );
  assert.equal(upload?.if, "${{ always() && env.MEASURE == 'true' }}");
  assert.equal(upload?.with?.path, "${{ runner.temp }}/formatter-instruction-counts");
  assert.equal(upload?.with?.["if-no-files-found"], "error");
  assert.equal(upload?.["continue-on-error"], undefined);
  const tests = gate?.steps?.find((step) => step.name === "Test instruction parser and gates");
  assert.match(tests?.run ?? "", /level-instruction-counts\.test\.mjs/);
  assert.match(tests?.run ?? "", /formatter-instruction-counts\.test\.mjs/);
});

test("formatter caps fail closed on every source and are enforced for every actual measurement", () => {
  const steps = instructionWorkflow.jobs?.["instruction-counts"]?.steps ?? [];
  const verify = steps.find(
    (step) => step.name === "Verify pinned registry and immutable base ratchet",
  );
  assert.equal(verify?.if, undefined);
  assert.match(
    verify?.run ?? "",
    /git cat-file -e "\$BASE_SHA:docs\/davinci\/plan\/formatter-instruction-budgets\.toml"/,
  );
  assert.match(
    verify?.run ?? "",
    /git show "\$BASE_SHA:docs\/davinci\/plan\/formatter-instruction-budgets\.toml"/,
  );
  assert.match(
    verify?.run ?? "",
    /[a-f0-9]{64}  docs\/davinci\/plan\/formatter-instruction-budgets\.toml' \| sha256sum --check --strict/,
  );
  assert.match(verify?.run ?? "", /--formatter --verify-budgets "\$\{FORMATTER_BASE_ARGS\[@\]\}"/);
  const enforce = steps.find((step) => step.name === "Enforce original formatter ceilings");
  assert.equal(enforce?.if, "env.MEASURE == 'true'");
  assert.match(enforce?.run ?? "", /--formatter --check/);
  assert.match(
    enforce?.run ?? "",
    /--measurement "\$\{RUNNER_TEMP\}\/formatter-instruction-counts\/measurement\.json" "\$\{BASE_ARGS\[@\]\}"/,
  );
  assert.match(
    enforce?.run ?? "",
    /--base-budgets "\$\{RUNNER_TEMP\}\/base-formatter-instruction-budgets\.toml"/,
  );
  assert.equal(enforce?.["continue-on-error"], undefined);
  assert.doesNotMatch(enforce?.run ?? "", /hashFiles|\|\| true|continue-on-error/);
  assert.ok(steps.indexOf(enforce) > steps.indexOf(verify));
});
