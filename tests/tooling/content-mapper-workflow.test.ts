import assert from "node:assert/strict";
import { test } from "node:test";
import { runInNewContext } from "node:vm";

import { parse } from "yaml";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

const UPSTREAM_SHA = "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799";
const CONTENT_MAPPER_PROTOCOL_COMMAND =
  "go test ./internal/contentmapper -run '^(TestRunnerTransform|TestRunnerTransformResponseValidation)$' -count=1";
const SPANMAP_PROTOCOL_COMMAND =
  "go test ./internal/spanmap -run '^(TestOriginalToVirtualOverlappingSpans|TestValidateOriginalOverlapAndFeatures)$' -count=1";
const TSGO_BUILD_COMMAND = 'go build -tags=noembed -trimpath -o "$RUNNER_TEMP/tsgo" ./cmd/tsc';
const MAESTRO_COMMAND = "cargo test -p vize_maestro -- --quiet";
const MAESTRO_LOOP = "for iteration in $(seq 1 20); do";
const MAESTRO_SUCCESS_LOG = 'echo "Content Mapper Maestro lifecycle cycle $iteration/20 passed"';
const PACKAGE_ROUTE_COMMANDS = [
  "cargo test -p vize --test check_importer_scoped_package_cli -- --nocapture",
  "cargo test -p vize --test check_package_declaration_barrel_cli -- --nocapture",
  "cargo test -p vize --test check_package_paths_authority_cli -- --nocapture",
  "cargo test -p vize --test check_package_resolution_modes_cli -- --nocapture",
  "cargo test -p vize --test check_package_scope_mode_cli -- --nocapture",
  "cargo test -p vize --test check_package_tsx_shadow_cli -- --nocapture",
];
interface WorkflowStep {
  env?: Record<string, string>;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
  "working-directory"?: string;
}

interface WorkflowConfig {
  on: {
    schedule?: Array<{ cron: string }>;
    workflow_dispatch?: unknown;
  };
}

function jobSteps(workflow: string, jobName: string): WorkflowStep[] {
  const jobs = (parse(workflow) as { jobs: Record<string, { steps?: WorkflowStep[] }> }).jobs;
  const steps = jobs[jobName]?.steps;
  assert.ok(Array.isArray(steps), `missing steps for job ${jobName}`);
  return steps;
}

function stepsRunning(steps: WorkflowStep[], command: string): number[] {
  return steps.flatMap((step, index) => (step.run?.includes(command) ? [index] : []));
}

function assertTriggerFilters(workflowConfig: WorkflowConfig): void {
  assert.ok(Object.hasOwn(workflowConfig.on, "workflow_dispatch"), "missing workflow_dispatch");
  assert.ok(
    !Object.hasOwn(workflowConfig.on, "pull_request"),
    "full Content Mapper conformance must stay off the fast PR path",
  );
  assert.equal(Object.hasOwn(workflowConfig.on, "push"), false);
  assert.deepEqual(workflowConfig.on.schedule, [{ cron: "21 4 * * *" }]);
}

test("Content Mapper conformance pins and runs the exact upstream project path", () => {
  const workflow = readRepoFile(".github", "workflows", "content-mapper-conformance.yml");
  const workflowConfig = parse(workflow) as WorkflowConfig;
  const job = workflowJobBody(workflow, "exact-tsgo-project");
  const steps = jobSteps(workflow, "exact-tsgo-project");

  assertTriggerFilters(workflowConfig);

  assert.match(workflow, new RegExp(`CONTENT_MAPPER_TYPESCRIPT_SHA: "${UPSTREAM_SHA}"`));
  assert.match(job, /repository: microsoft\/TypeScript/);
  assert.match(job, /ref: \$\{\{ env\.CONTENT_MAPPER_TYPESCRIPT_SHA \}\}/);
  const upstreamCheckoutStep = steps.find(
    (step) => step.name === "Checkout exact TypeScript Content Mapper revision",
  );
  assert.ok(upstreamCheckoutStep, "missing exact TypeScript Content Mapper checkout step");
  assert.equal(
    upstreamCheckoutStep.uses,
    "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
  );
  assert.equal(upstreamCheckoutStep.with?.repository, "microsoft/TypeScript");
  assert.equal(upstreamCheckoutStep.with?.ref, "${{ env.CONTENT_MAPPER_TYPESCRIPT_SHA }}");
  assert.equal(upstreamCheckoutStep.with?.path, "typescript-content-mapper");
  assert.equal(upstreamCheckoutStep.with?.["persist-credentials"], false);
  assert.match(job, /uses: actions\/setup-go@[0-9a-f]{40}\s+# v6\.1\.0/);
  assert.match(
    job,
    /uses: \.\/\.github\/actions\/setup-rust-sticky-cache\n\s+with:\n\s+key: content-mapper-conformance\n\s+cache-key-suffix: \$\{\{ runner\.os \}\}-\$\{\{ runner\.arch \}\}/,
  );
  assert.match(job, /go-version-file: typescript-content-mapper\/tsc\/go\.mod/);
  const contentMapperProtocolSteps = stepsRunning(steps, CONTENT_MAPPER_PROTOCOL_COMMAND);
  const spanmapProtocolSteps = stepsRunning(steps, SPANMAP_PROTOCOL_COMMAND);
  const buildSteps = stepsRunning(steps, TSGO_BUILD_COMMAND);
  assert.equal(
    contentMapperProtocolSteps.length,
    1,
    "expected exactly one upstream content mapper protocol regression step",
  );
  assert.equal(
    spanmapProtocolSteps.length,
    1,
    "expected exactly one upstream spanmap regression step",
  );
  assert.equal(buildSteps.length, 1, "expected exactly one exact tsgo build step");
  assert.equal(
    steps[contentMapperProtocolSteps[0]]["working-directory"],
    "typescript-content-mapper/tsc",
  );
  assert.equal(
    steps[spanmapProtocolSteps[0]]["working-directory"],
    "typescript-content-mapper/tsc",
  );
  assert.ok(
    contentMapperProtocolSteps[0] < buildSteps[0] && spanmapProtocolSteps[0] < buildSteps[0],
    "upstream protocol regressions must run before the exact tsgo build",
  );
  assert.match(job, /cp internal\/bundled\/libs\/\*\.d\.ts "\$RUNNER_TEMP\/"/);
  assert.match(job, /VIZE_TEST_CONTENT_MAPPER_TSGO: \$\{\{ runner\.temp \}\}\/tsgo/);
  assert.match(
    job,
    /VIZE_TEST_CONTENT_MAPPER_JAVASCRIPT_TSC: \$\{\{ github\.workspace \}\}\/npm\/cli\/node_modules\/\.bin\/tsc/,
  );
  assert.match(job, /cargo test -p vize --test content_mapper_tsgo_cli -- --nocapture/);
  assert.match(job, /cargo test -p vize --test content_mapper_tsgo_directives -- --nocapture/);
  assert.match(job, /cargo test -p vize --test content_mapper_tsgo_build -- --nocapture/);
  assert.match(job, /cargo test -p vize --test content_mapper_tsgo_incremental -- --nocapture/);
  assert.match(
    job,
    /cargo test -p vize --test content_mapper_tsgo_package_references -- --nocapture/,
  );
  assert.match(job, /cargo test -p vize --test content_mapper_tsgo_watch -- --nocapture/);
  assert.match(job, /vp run --filter '\.\/npm\/native' build:ci/);
  assert.match(job, /\(cd npm\/cli && vp pack\)/);
  assert.match(job, /vp exec napi create-npm-dirs/);
  assert.match(job, /cp "\$binary" "npm\/\$target\/"/);
  assert.match(
    job,
    /VIZE_TEST_CONTENT_MAPPER_TSGO: \$\{\{ runner\.temp \}\}\/tsgo[\s\S]*smoke-release-install\.rs --prepare-manifests --content-mapper-checks[\s\S]*npm\/native npm\/native\/npm\/\*[\s\S]*npm\/cli/,
  );
  const packageRouteSteps = steps.flatMap((step, index) =>
    PACKAGE_ROUTE_COMMANDS.every((command) => step.run?.includes(command)) ? [index] : [],
  );
  assert.equal(packageRouteSteps.length, 1, "expected one package resolution matrix step");
  assert.ok(buildSteps[0] < packageRouteSteps[0], "package resolution must use exact tsgo");
  const packageRouteStep = steps[packageRouteSteps[0]];
  assert.equal(packageRouteStep.name, "Run importer-scoped package resolution matrix");
  assert.equal(packageRouteStep.env?.CORSA_PATH, "${{ runner.temp }}/tsgo");
  assert.equal(packageRouteStep.env?.VIZE_TEST_REQUIRE_TSGO, "1");
  const editorCommands = [
    "cargo test -p vize --test content_mapper_tsgo_lsp -- --nocapture",
    "cargo test -p vize --test content_mapper_lsp_shutdown -- --nocapture",
    "cargo test -p vize --test content_mapper_tsgo_declaration_lsp -- --nocapture",
    "cargo test -p vize --test content_mapper_tsgo_lsp_event_forms -- --nocapture",
    "cargo test -p vize_canon --test lsp_import_resolution -- --nocapture",
  ];
  const editorSteps = steps.filter((step) =>
    editorCommands.every((command) => step.run?.includes(command)),
  );
  assert.equal(editorSteps.length, 1, "expected one mixed editor backend step");
  assert.equal(editorSteps[0].name, "Run Content Mapper LSP and virtual-overlay editor checks");
  assert.equal(editorSteps[0].env?.TSGO_PATH, "${{ runner.temp }}/tsgo");
  assert.equal(editorSteps[0].env?.VIZE_TEST_CONTENT_MAPPER_TSGO, "${{ runner.temp }}/tsgo");
  const stressSteps = stepsRunning(steps, MAESTRO_COMMAND);
  assert.equal(stressSteps.length, 1, "expected exactly one Maestro lifecycle stress step");
  const stressStep = steps[stressSteps[0]];
  const stressRun = stressStep.run ?? "";
  assert.equal(stressStep.env?.TSGO_PATH, "${{ runner.temp }}/tsgo");
  assert.equal(stressStep.env?.VIZE_TEST_CONTENT_MAPPER_TSGO, "${{ runner.temp }}/tsgo");
  assert.equal(
    [...stressRun.matchAll(/^\s*cargo test -p vize_maestro -- --quiet\s*$/gm)].length,
    1,
    "expected one direct Maestro command in the iteration loop",
  );
  assert.equal(
    [...stressRun.matchAll(/^\s*(?:for|while|until)\b/gm)].length,
    1,
    "retry loops are forbidden in the lifecycle stress step",
  );
  assert.doesNotMatch(
    stressRun,
    /\bsleep\b|--test-threads(?:=|\s+)1|RUST_TEST_THREADS\s*=\s*1|\|\|\s*true|\bset\s+\+e\b|\b(?:retry|attempt)\b|(?:grep|rg).*\b(?:panic|fail)/i,
    "stress step must not serialize, retry, sleep, ignore failures, or filter panics",
  );
  const loopStart = stressRun.indexOf(MAESTRO_LOOP);
  assert.ok(loopStart >= 0, "Maestro stress step must run a 20-iteration loop");
  const maestroRun = stressRun.indexOf(MAESTRO_COMMAND);
  const successLog = stressRun.indexOf(MAESTRO_SUCCESS_LOG);
  const loopEnd = stressRun.slice(loopStart).search(/^\s*done\s*$/m) + loopStart;
  assert.ok(loopStart < maestroRun, "Maestro command must run inside the loop");
  assert.ok(maestroRun < successLog, "success log must follow the Maestro command");
  assert.ok(successLog < loopEnd, "success log must run inside the same loop");
});

test("manual focused campaigns preserve the full default and upload before gating", () => {
  const workflow = parse(
    readRepoFile(".github", "workflows", "content-mapper-conformance.yml"),
  ) as {
    on: { workflow_dispatch: { inputs: Record<string, { type: string; default: boolean }> } };
    concurrency: { group: string };
    jobs: Record<
      string,
      {
        if: string;
        steps: Array<WorkflowStep & { id?: string; if?: string; "continue-on-error"?: boolean }>;
      }
    >;
  };
  assert.equal(
    workflow.concurrency.group,
    "content-mapper-${{ github.event.pull_request.number || github.ref }}${{ inputs.directive-owners-only && '-directive-owners' || inputs.event-rename-only && '-event-rename' || '' }}",
    "a focused campaign must not cancel the full conformance campaign",
  );
  const input = workflow.on.workflow_dispatch.inputs["event-rename-only"];
  assert.equal(input.type, "boolean");
  assert.equal(input.default, false);
  assert.deepEqual(workflow.on.workflow_dispatch.inputs["directive-owners-only"], {
    description: "Compare genuine old/new static directive owners and standard stock LSP gates",
    type: "boolean",
    default: false,
  });
  const groups: string[] = [];
  for (const [job, suffix, rename, owners] of [
    ["exact-tsgo-project", "", false, false],
    ["event-rename", "-event-rename", true, false],
    ["directive-owners", "-directive-owners", false, true],
  ] as const) {
    const context = {
      github: {
        event_name: "workflow_dispatch",
        event: { pull_request: {} },
        ref: "refs/heads/main",
      },
      inputs: { "event-rename-only": rename, "directive-owners-only": owners },
    };
    const evaluate = (expression: string): unknown =>
      runInNewContext(
        expression
          .replace(/^\$\{\{\s*|\s*\}\}$/gu, "")
          .replace(/\binputs\.([a-z-]+)/gu, 'inputs["$1"]'),
        context,
      );
    const group: string = workflow.concurrency.group.replace(/\$\{\{(.*?)\}\}/gu, (_, expression) =>
      String(evaluate(expression)),
    );
    assert.equal(group, `content-mapper-refs/heads/main${suffix}`);
    groups.push(group);
    assert.deepEqual(
      Object.entries(workflow.jobs)
        .filter(([, candidate]) => evaluate(candidate.if))
        .map(([name]) => name),
      [job],
    );
  }
  assert.equal(
    new Set(groups).size,
    3,
    "all three campaigns must run without canceling each other",
  );
  assert.equal(
    workflow.jobs["exact-tsgo-project"].if,
    "${{ !inputs.event-rename-only && !inputs.directive-owners-only }}",
  );
  const focused = workflow.jobs["event-rename"];
  assert.equal(
    focused.if,
    "github.event_name == 'workflow_dispatch' && inputs.event-rename-only && !inputs.directive-owners-only",
  );
  const rust = focused.steps.find((step) => step.name === "Setup Rust");
  const pinned = /^channel = "([^"]+)"$/mu.exec(readRepoFile("rust-toolchain.toml"))?.[1];
  assert.ok(pinned);
  assert.equal(rust?.with?.toolchain, pinned);
  const transactions = focused.steps.find((step) => step.id === "transactions");
  assert.ok(transactions);
  assert.equal(transactions["continue-on-error"], true);
  assert.equal(transactions.env?.VIZE_TEST_CONTENT_MAPPER_TSGO, "${{ runner.temp }}/tsgo");
  assert.equal(
    transactions.env?.VIZE_TEST_CONTENT_MAPPER_VUE,
    "${{ github.workspace }}/npm/cli/node_modules/vue",
  );
  assert.match(transactions.run ?? "", /cargo test --locked --profile ci --no-fail-fast -p vize/u);
  for (const target of ["lsp_bound_event_casing_cli", "lsp_named_model_transactions_cli"])
    assert.match(transactions.run ?? "", new RegExp(`--test ${target}\\b`, "u"));
  const upload = focused.steps.findIndex((step) =>
    step.uses?.startsWith("actions/upload-artifact@"),
  );
  const gate = focused.steps.findIndex((step) => step.env?.TRANSACTIONS_OUTCOME !== undefined);
  assert.ok(upload > focused.steps.indexOf(transactions));
  assert.ok(gate > upload);
  assert.equal(focused.steps[upload].if, "always()");
  assert.equal(focused.steps[gate].if, "always()");
  assert.equal(focused.steps[gate].env?.TRANSACTIONS_OUTCOME, "${{ steps.transactions.outcome }}");
  assert.equal(focused.steps[gate].run?.trim(), 'test "$TRANSACTIONS_OUTCOME" = success');
});
