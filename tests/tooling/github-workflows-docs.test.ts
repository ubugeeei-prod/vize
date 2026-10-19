import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, matchesGlob } from "node:path";
import { test } from "node:test";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { parse } from "yaml";

import {
  requiredReleaseWorkflowEvidence,
  requiredReleaseWorkflows,
  selectRequiredWorkflowRuns,
} from "../../tools/support/compat/github/release-preflight-evidence.mjs";
import { readRepoFile } from "./support/github-workflows.ts";
import { releaseSha, successfulReleaseRun } from "./support/release-preflight.ts";

interface WorkflowStep {
  name?: string;
  id?: string;
  if?: string;
  uses?: string;
  env?: Record<string, string>;
  run?: string;
  with?: Record<string, unknown>;
}

interface WorkflowJob {
  if?: string;
  needs?: string;
  concurrency?: { group?: string; queue?: string; "cancel-in-progress"?: boolean };
  permissions?: Record<string, string>;
  outputs?: Record<string, string>;
  steps?: WorkflowStep[];
}

interface Workflow {
  name?: string;
  on?: Record<string, unknown>;
  concurrency?: { group?: string; "cancel-in-progress"?: boolean };
  permissions?: Record<string, string>;
  jobs?: Record<string, WorkflowJob>;
}

function readWorkflow(file: string): Workflow {
  return parse(readRepoFile(".github", "workflows", file)) as Workflow;
}

function workflowJob(workflow: Workflow, name: string): WorkflowJob {
  const job = workflow.jobs?.[name];
  assert.ok(job, `missing workflow job ${name}`);
  return job;
}

function namedStep(job: WorkflowJob, name: string): WorkflowStep {
  const step = job.steps?.find((candidate) => candidate.name === name);
  assert.ok(step, `missing workflow step ${name}`);
  return step;
}

test("docs build evidence is immutable per SHA and has no Pages authority", () => {
  const workflow = readWorkflow("build-docs.yml");
  const events = workflow.on as {
    push?: { branches?: string[]; paths?: string[] };
    schedule?: Array<{ cron: string }>;
    workflow_dispatch?: unknown;
  };

  assert.equal(workflow.name, "Docs build");
  assert.deepEqual(Object.keys(events).sort(), [
    "push",
    "schedule",
    "workflow_call",
    "workflow_dispatch",
  ]);
  assert.deepEqual(events.push?.branches, ["main"]);
  assert.deepEqual(events.push?.paths, [
    ".github/workflows/build-docs.yml",
    "docs/**",
    "!docs/davinci/decisions/**",
    "npm/compose/core/examples/**",
    "npm/ui/src/**/examples/**",
    "npm/ui/scripts/generate-reference-docs.ts",
    "npm/ui/scripts/reference-docs/**",
    "tests/tooling/docs-preview-readiness-browser.test.ts",
    "playground/**",
    "examples/vite-musea/**",
  ]);
  assert.deepEqual(events.schedule, [{ cron: "41 5 * * *" }]);
  assert.equal(
    workflow.concurrency?.group,
    "docs-build-${{ github.event_name }}-${{ github.ref }}-${{ github.sha }}",
  );
  assert.equal(workflow.concurrency?.["cancel-in-progress"], true);
  assert.deepEqual(Object.keys(workflow.jobs ?? {}).sort(), ["build-docs", "build-playground"]);
  assert.equal(workflow.permissions?.contents, "read");

  const source = readRepoFile(".github", "workflows", "build-docs.yml");
  assert.doesNotMatch(source, /pages:\s*write|id-token:\s*write|actions\/deploy-pages/);
  for (const artifact of ["docs", "playground", "musea-examples"]) {
    const upload = Object.values(workflow.jobs ?? {})
      .flatMap((job) => job.steps ?? [])
      .find((step) => step.with?.name === artifact);
    assert.ok(upload, `missing ${artifact} artifact upload`);
    assert.match(upload.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
    assert.equal(upload.with?.["if-no-files-found"], "error");
  }
});

test("authored reader inputs publish while runtime-only and excluded inputs keep their policy", () => {
  const events = readWorkflow("build-docs.yml").on as {
    push: { paths: string[] };
  };
  const publishes = (file: string) =>
    events.push.paths.reduce((included, pattern) => {
      const excluded = pattern.startsWith("!");
      return matchesGlob(file, excluded ? pattern.slice(1) : pattern) ? !excluded : included;
    }, false);
  for (const [file, expected] of [
    ["npm/compose/core/examples/virtual-list.vue", true],
    ["npm/compose/core/examples/tsconfig.json", true],
    ["npm/ui/src/families/actions/button/examples/button-basic.vue", true],
    ["npm/ui/scripts/generate-reference-docs.ts", true],
    ["npm/ui/scripts/reference-docs/composable-examples.ts", true],
    ["docs/content/getting-started.md", true],
    ["tests/tooling/docs-preview-readiness-browser.test.ts", true],
    ["npm/compose/core/src/virtual-list.ts", false],
    ["npm/ui/scripts/check-renderers.ts", false],
    ["docs/davinci/decisions/2026-10-16-8374-virtual-list-catalogue.md", false],
    ["tests/tooling/github-workflows-docs.test.ts", false],
  ] as const) {
    assert.equal(publishes(file), expected, file);
  }
});

test("full Docs checks real preview resources before building and retains the raw evidence", () => {
  const job = workflowJob(readWorkflow("build-docs.yml"), "build-docs");
  const readiness = namedStep(job, "Check preview resource readiness");
  const steps = job.steps!;
  assert.equal(readiness.if, "${{ inputs.links-only != true }}");
  assert(steps.indexOf(readiness) > steps.indexOf(namedStep(job, "Install JS dependencies")));
  assert(
    steps.indexOf(readiness) > steps.indexOf(namedStep(job, "Install Japanese rendering fonts")),
  );
  assert(steps.indexOf(readiness) < steps.indexOf(namedStep(job, "Build docs")));
  assert.match(readiness.run!, /vp node docs\/scripts\/ensure-browser\.ts/);
  assert.match(
    readiness.run!,
    /node --test --test-reporter=tap tests\/tooling\/docs-preview-readiness-browser\.test\.ts/,
  );
  assert.match(readiness.run!, /\| tee docs-render-evidence\/resource-readiness\/tests\.tap/);
  assert.match(readiness.run!, /receipt\.json/);
  const reading = namedStep(job, "Verify rendered navigation and Japanese reading paths");
  assert.equal(reading.if, "${{ inputs.links-only != true }}");
  assert.equal(
    reading.run,
    [
      'ja_reading_routes=$(node --input-type=module -e \'import { defaultNavigationRenderRoutes } from "./docs/scripts/navigation-render-routes.ts"; process.stdout.write(defaultNavigationRenderRoutes.filter(route => route.startsWith("/ja/")).join(","))\')',
      'node docs/scripts/verify-navigation-render.ts --routes "$ja_reading_routes"',
      "node docs/scripts/verify-ja-editorial-render.ts",
      "",
    ].join("\n"),
  );
  const japaneseRoutes = defaultNavigationRenderRoutes.filter((route) => route.startsWith("/ja/"));
  assert.equal(japaneseRoutes.length, 42);
  const fixture = JSON.parse(readRepoFile("tests", "_fixtures", "docs-ja-rule-editorial.json")) as {
    cases: { case: string; route: string }[];
  };
  for (const witness of fixture.cases)
    assert(japaneseRoutes.includes(witness.route), `${witness.case}: original rendered witness`);
  assert(steps.indexOf(reading) > steps.indexOf(namedStep(job, "Build docs")));
  const upload = namedStep(job, "Upload rendered docs screenshots");
  assert(steps.indexOf(reading) < steps.indexOf(upload));
  assert.equal(upload.if, "${{ always() }}");
  assert.equal(upload.with?.path, "docs-render-evidence");
});

test("Docs resource fixture preserves Node exit failure even after a passing TAP summary", () => {
  const job = workflowJob(readWorkflow("build-docs.yml"), "build-docs");
  const step = namedStep(job, "Check preview resource readiness");
  const temporary = mkdtempSync(join(tmpdir(), "docs-resource-exit-"));
  try {
    const bin = join(temporary, "bin");
    mkdirSync(bin);
    writeFileSync(join(bin, "vp"), "#!/bin/sh\nexit 0\n", { mode: 0o755 });
    writeFileSync(
      join(bin, "node"),
      `#!/bin/sh
if [ "$1" = "--test" ]; then
  printf '%s\\n' '# tests 8' '# pass 8' '# fail 0' '# cancelled 0' '# skipped 0' '# todo 0'
  exit 17
fi
touch "$READINESS_CONTROL_MARKER"
`,
      { mode: 0o755 },
    );
    const script = join(temporary, "step.sh");
    const marker = join(temporary, "unexpected-receipt");
    writeFileSync(script, step.run!);
    // Execute the actual YAML body without supplying shell failure options:
    // the workflow must preserve the producer's status through tee itself.
    const result = spawnSync("bash", [script], {
      cwd: temporary,
      env: { ...process.env, PATH: `${bin}:${process.env.PATH}`, READINESS_CONTROL_MARKER: marker },
      encoding: "utf8",
    });
    assert.equal(result.status, 17, `${result.stdout}\n${result.stderr}`);
    assert.match(
      readFileSync(join(temporary, "docs-render-evidence/resource-readiness/tests.tap"), "utf8"),
      /# pass 8/,
    );
    assert.equal(existsSync(marker), false, "a failed producer cannot write a success receipt");
    assert.equal(
      existsSync(join(temporary, "docs-render-evidence/resource-readiness/receipt.json")),
      false,
    );
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

test("docs deployment serializes before validating actual source and publication custody", () => {
  const workflow = readWorkflow("deploy-docs.yml");
  const events = workflow.on as {
    workflow_run?: { workflows?: string[]; types?: string[]; branches?: string[] };
  };
  const deploy = workflowJob(workflow, "deploy");
  const checkoutMain = namedStep(deploy, "Checkout current main");
  const compare = namedStep(deploy, "Validate completed build and actual Pages publication floor");

  assert.equal(workflow.name, "Deploy docs");
  assert.deepEqual(Object.keys(events), ["workflow_run"]);
  assert.deepEqual(events.workflow_run?.workflows, ["Docs build"]);
  assert.deepEqual(events.workflow_run?.types, ["completed"]);
  assert.deepEqual(events.workflow_run?.branches, ["main"]);
  assert.deepEqual(Object.keys(workflow.jobs ?? {}), ["deploy"]);
  assert.equal(workflow.concurrency, undefined);
  assert.equal(deploy.if, "${{ github.event.workflow_run.conclusion == 'success' }}");
  assert.equal(deploy.concurrency?.group, "pages-main");
  assert.equal(
    deploy.concurrency?.queue,
    "max",
    "a stale late arrival must not replace a newer pending deployment",
  );
  assert.equal(
    deploy.concurrency?.["cancel-in-progress"],
    false,
    "an older completed build must never cancel a newer deployment",
  );
  assert.equal(checkoutMain.with?.ref, "main");
  assert.equal(
    deploy.steps?.[0],
    checkoutMain,
    "main must be fetched after concurrency is acquired",
  );
  assert.equal(
    deploy.steps?.[2],
    compare,
    "primary custody must be checked before any source checkout or Pages effects",
  );
  assert.equal(compare.env?.GITHUB_TOKEN, "${{ github.token }}");
  assert.match(
    compare.run ?? "",
    /cp tools\/support\/compat\/github\/docs-deployment-\*\.ts "\$RUNNER_TEMP\/"/,
  );
  assert.match(compare.run ?? "", /vp node "\$RUNNER_TEMP\/docs-deployment-entry\.ts" prepare/);
});

test("only a source-custody-qualified build can download or deploy, and success follows actual Pages", () => {
  const workflow = readWorkflow("deploy-docs.yml");
  const deploy = workflowJob(workflow, "deploy");
  const download = namedStep(deploy, "Download docs build artifacts");
  const checkout = namedStep(deploy, "Checkout docs build");
  const pages = namedStep(deploy, "Deploy to GitHub Pages");
  const record = namedStep(deploy, "Record Pages artifact custody");
  const confirm = namedStep(deploy, "Confirm actual Pages publication");
  const eligibility = "${{ steps.main.outputs.eligible == 'true' }}";

  assert.equal(deploy.permissions?.actions, "read");
  assert.equal(deploy.permissions?.pages, "write");
  assert.equal(deploy.permissions?.deployments, "write");
  assert.equal(deploy.permissions?.["id-token"], "write");
  assert.equal(checkout?.with?.ref, "${{ github.event.workflow_run.head_sha }}");
  for (const step of deploy.steps?.slice(3).filter((step) => step !== confirm) ?? []) {
    assert.equal(step.if, eligibility, `${step.name ?? step.uses} bypasses the freshness gate`);
  }
  assert.equal(download.with?.["run-id"], "${{ github.event.workflow_run.id }}");
  assert.equal(download.with?.["artifact-ids"], "${{ steps.main.outputs.artifact_ids }}");
  assert.equal(download.with?.["github-token"], "${{ github.token }}");
  assert.equal(download.with?.path, "artifacts");
  assert.match(
    pages.uses ?? "",
    /^actions\/deploy-pages@d6db90164ac5ed86f2b6aed7e0febac5b3c0c03e$/,
  );
  assert.equal(
    confirm.if,
    "${{ steps.main.outputs.eligible == 'true' && steps.deployment.conclusion == 'success' }}",
  );
  assert(deploy.steps!.indexOf(record) < deploy.steps!.indexOf(pages));
  assert(deploy.steps!.indexOf(confirm) > deploy.steps!.indexOf(pages));
});

test("release preflight does not wait on docs build or mutable deployment", () => {
  assert.equal(requiredReleaseWorkflows.includes("Docs build"), false);
  assert.equal(requiredReleaseWorkflows.includes("Deploy docs"), false);
  assert.deepEqual(requiredReleaseWorkflowEvidence.get("Docs build"), {
    path: ".github/workflows/build-docs.yml",
    events: ["workflow_dispatch"],
  });

  const runs = requiredReleaseWorkflows.map((name, index) => successfulReleaseRun(name, index + 1));
  assert.doesNotThrow(() => selectRequiredWorkflowRuns(runs, releaseSha));
});
