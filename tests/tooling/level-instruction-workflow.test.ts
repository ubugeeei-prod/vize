import assert from "node:assert/strict";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import { parse } from "yaml";

import { aggregateCheckReportResults } from "../../tools/support/compat/github/require-needs-success.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

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

test("instruction ceilings are required in the queue and scheduled qualification", () => {
  const caller = workflow.jobs?.["instruction-counts"];
  assert.equal(caller?.uses, "./.github/workflows/level-instruction-counts.yml");
  assert.equal(caller?.["continue-on-error"], undefined);
  assert.equal(instructionWorkflow.jobs?.["instruction-counts"]?.["continue-on-error"], undefined);
  assert.equal(
    caller?.if,
    "${{ github.event_name == 'merge_group' || github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  for (const name of ["test-report", "full-js-report"])
    assert.ok(workflow.jobs?.[name]?.needs?.includes("instruction-counts"), name);
  assert.ok(Object.hasOwn(instructionWorkflow.on ?? {}, "workflow_call"));
  const gate = instructionWorkflow.jobs?.["instruction-counts"];
  assert.equal(gate?.if, undefined);
  for (const step of gate?.steps ?? [])
    assert.equal(step["continue-on-error"], undefined, step.name);
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

test("required outer report rejects absent instructions and permits only the PR skip", () => {
  for (const event of ["pull_request", "merge_group", "schedule", "workflow_dispatch"]) {
    assert.throws(
      () => aggregateCheckReportResults({ "check-js": { result: "success" } }, event),
      /missing instruction-counts/,
    );
    assert.throws(
      () => aggregateCheckReportResults({ "instruction-counts": {} }, event),
      /reported no result/,
    );
    for (const result of ["success", "failure", "cancelled", "skipped"]) {
      const decision = aggregateCheckReportResults({ "instruction-counts": { result } }, event);
      assert.equal(
        decision.exitCode,
        result === "success" || (event === "pull_request" && result === "skipped") ? 0 : 1,
        `${event}: ${result}`,
      );
    }
    // The instruction exception must not permit a missing product check.
    assert.equal(
      aggregateCheckReportResults(
        { "instruction-counts": { result: "success" }, "pr-source-checks": { result: "skipped" } },
        event,
      ).exitCode,
      1,
    );
  }
  assert.throws(
    () => aggregateCheckReportResults({ "instruction-counts": { result: "success" } }, "push"),
    /Unsupported Check report event/,
  );
});

test("the actual required Check report command fails a red or missing instruction producer", () => {
  const report = workflow.jobs?.["test-report"];
  const gates = report?.steps?.filter(
    (step) => step.name === "Require every needed job to succeed",
  );
  assert.equal(gates?.length, 1);
  const gate = gates?.[0];
  assert.equal(gate?.if, undefined);
  assert.equal(gate?.["continue-on-error"], undefined);
  const command = gate?.run;
  assert.ok(command);
  const [executable, ...args] = command.trim().split(/\s+/);
  assert.equal(executable, "node");
  assert.deepEqual(args, [
    "tools/support/compat/github/require-needs-success.ts",
    "--check-report",
  ]);
  const run = (event: string, results: Record<string, unknown>) =>
    spawnSync(process.execPath, args, {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, GITHUB_EVENT_NAME: event, NEEDS_JSON: JSON.stringify(results) },
    });
  const green = () =>
    Object.fromEntries((report?.needs ?? []).map((name) => [name, { result: "success" }]));
  for (const event of ["pull_request", "merge_group"]) {
    assert.equal(run(event, green()).status, 0);
    for (const result of ["failure", "cancelled", "skipped"]) {
      const observation = run(event, { ...green(), "instruction-counts": { result } });
      assert.equal(
        observation.status,
        event === "pull_request" && result === "skipped" ? 0 : 1,
        `${event}: ${result}`,
      );
    }
    const missing = green();
    delete missing["instruction-counts"];
    const absent = run(event, missing);
    assert.equal(absent.status, 1);
    assert.match(absent.stderr, /missing instruction-counts/);
  }
});

test("full scheduled and manual aggregate commands reject incomplete instruction measurement", () => {
  const report = workflow.jobs?.["full-js-report"];
  const action = parse(
    readRepoFile(".github", "actions", "report-full-js-qualification", "action.yml"),
  ) as { runs: { steps: { run?: string; env?: Record<string, string> }[] } };
  assert.equal(report?.steps?.at(-1)?.uses, "./.github/actions/report-full-js-qualification");
  assert.deepEqual(report?.steps?.at(-1)?.with, { "needs-json": "${{ toJSON(needs) }}" });
  const command = action.runs.steps.at(-1)?.run;
  assert.ok(command);
  const [executable, ...args] = command.trim().split(/\s+/);
  assert.equal(executable, "node");
  assert.deepEqual(args, ["tools/support/compat/github/require-needs-success.ts"]);
  for (const event of ["schedule", "workflow_dispatch"])
    for (const result of ["success", "failure", "cancelled", "skipped", undefined]) {
      const results: Record<string, { result?: string }> = Object.fromEntries(
        (report?.needs ?? []).map((name) => [name, { result: "success" }]),
      );
      results["instruction-counts"] = result ? { result } : {};
      const observation = spawnSync(process.execPath, args, {
        cwd: root,
        encoding: "utf8",
        env: { ...process.env, GITHUB_EVENT_NAME: event, NEEDS_JSON: JSON.stringify(results) },
      });
      assert.equal(observation.status, result === "success" ? 0 : 1, `${event}: ${result}`);
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
