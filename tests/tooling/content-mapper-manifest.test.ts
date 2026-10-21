import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";

import { parse } from "yaml";

import { officialActorInputs } from "../../tools/support/compat/content-mapper/official-fixtures.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("vize package exposes an executable TypeScript content mapper", () => {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/package.json"), "utf-8"),
  ) as {
    bin?: Record<string, string>;
    files?: string[];
    tsContentMapper?: unknown;
    typescript?: {
      contentMapper?: {
        dynamicConfig?: boolean;
        compilerOptions?: string[];
        exec?: string[];
      };
    };
  };
  const binPath = path.join(root, "npm/cli/bin/vize");

  assert.deepEqual(packageJson.typescript?.contentMapper, {
    dynamicConfig: true,
    exec: ["node", "./bin/vize", "content-mapper"],
    compilerOptions: ["checkJs", "noUnusedLocals"],
  });
  assert.equal(
    packageJson.tsContentMapper,
    undefined,
    "the retired tsContentMapper key must not ship alongside typescript.contentMapper",
  );
  assert.equal(packageJson.bin?.vize, "bin/vize");
  assert.ok(packageJson.files?.includes("bin"));
  assert.notEqual(
    fs.statSync(binPath).mode & 0o111,
    0,
    "content mapper entrypoint must be executable",
  );
  assert.match(fs.readFileSync(binPath, "utf-8"), /^#!\/usr\/bin\/env node\n/);
});

test("VS Code discovers Vize through the trusted native preview command", () => {
  const discovery = fs.readFileSync(
    path.join(root, "editors/vscode/src/content-mapper-discovery.ts"),
    "utf-8",
  );

  assert.match(discovery, /extensions\.getExtension\("TypeScriptTeam\.native-preview"\)/);
  assert.match(
    discovery,
    /commands\.executeCommand\(\s*"typescript\.native-preview\.discoverContentMappers"/,
  );
  assert.match(discovery, /workspace\.isTrusted/);
  assert.match(discovery, /extensions: \["\.vue"\]/);
});

interface PublicStep {
  name?: string;
  if?: string;
  uses?: string;
  run?: string;
  env?: Record<string, string>;
  with?: Record<string, unknown>;
  "continue-on-error"?: boolean;
}
const publishedWorkflow = () =>
  parse(readRepoFile(".github", "workflows", "content-mapper-conformance.yml")) as {
    permissions: Record<string, string>;
    concurrency: { group: string; "cancel-in-progress": string };
    on: {
      workflow_dispatch: {
        inputs: Record<string, { type: string; default?: boolean | string; options?: string[] }>;
      };
    };
    jobs: Record<
      string,
      { if: string; env?: Record<string, string>; "timeout-minutes": number; steps: PublicStep[] }
    >;
  };
const publicScope = {
  "published-defaults-only": true,
  "official-css-options-only": false,
  "event-rename-only": false,
};
function evaluates(expression: string, inputs = publicScope, runId = "7001"): unknown {
  return runInNewContext(
    expression
      .replace(/^\$\{\{\s*|\s*\}\}$/gu, "")
      .replace(/\binputs\.([a-z-]+)/gu, 'inputs["$1"]'),
    {
      inputs,
      github: {
        event_name: "workflow_dispatch",
        ref: "refs/heads/main",
        run_id: runId,
        event: { pull_request: {} },
      },
      always: () => true,
    },
  );
}

test("published mapper workflow selects fixed reports without source builds or baselines", () => {
  const workflow = publishedWorkflow();
  assert.deepEqual(workflow.permissions, { contents: "read" });
  const selection = workflow.on.workflow_dispatch.inputs["published-defaults-only"];
  assert.equal(selection.type, "boolean");
  assert.equal(selection.default, false);
  assert.deepEqual(
    Object.entries(workflow.jobs)
      .filter(([, job]) => evaluates(job.if))
      .map(([name]) => name),
    ["official-css-options"],
  );
  assert.equal(evaluates(workflow.concurrency["cancel-in-progress"]), false);
  assert.equal(
    evaluates(workflow.concurrency["cancel-in-progress"], {
      ...publicScope,
      "published-defaults-only": false,
    }),
    true,
  );
  const group = (runId: string, inputs = publicScope) =>
    workflow.concurrency.group.replace(/\$\{\{(.*?)\}\}/gu, (_, expression) =>
      String(evaluates(expression, inputs, runId)),
    );
  assert.deepEqual(
    ["7001", "7002", "7003"].map((runId) => group(runId)),
    ["7001", "7002", "7003"].map((runId) => `content-mapper-${runId}-published-defaults`),
    "independent public runs must never replace a pending acceptance run",
  );
  const sourceScope = { ...publicScope, "published-defaults-only": false };
  assert.equal(
    group("7001", sourceScope),
    group("7002", sourceScope),
    "source runs on the same ref retain their existing supersession policy",
  );
  const job = workflow.jobs["official-css-options"];
  assert.equal(job["timeout-minutes"], 15);
  assert.equal(job.env?.EXPECTED_SOURCE_SHA, "${{ inputs.expected-source-sha }}");
  const steps = job.steps.filter((step) => !step.if || evaluates(step.if));
  assert.equal(
    steps.some((step) => /rust|wild-linker|setup-vp/u.test(step.uses ?? "")),
    false,
  );
  assert.equal(
    steps.some((step) => /\bcargo\b|\bvp\b|\bnapi\b|build:ci/u.test(step.run ?? "")),
    false,
  );
  const checkout = steps.find((step) => step.uses?.startsWith("actions/checkout@"))!;
  assert.deepEqual(checkout.with, {
    ref: "${{ github.workflow_sha }}",
    "fetch-depth": 0,
    "persist-credentials": false,
  });
  const node = steps.find((step) => step.uses?.startsWith("actions/setup-node@"))!;
  assert.equal(node.uses, "actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38");
  assert.deepEqual(node.with, {
    "node-version-file": "package.json",
    "package-manager-cache": false,
  });
  const admission = steps.find(
    (step) => step.name === "Authenticate source and recipe before any build",
  )!;
  assert.ok(steps.indexOf(admission) > steps.indexOf(checkout));
  assert.ok(steps.indexOf(admission) < steps.indexOf(node));
  for (const comparison of [
    'test "$(git rev-parse HEAD)" = "$EXPECTED_SOURCE_SHA"',
    'test "$GITHUB_SHA" = "$EXPECTED_SOURCE_SHA"',
    'test "$GITHUB_WORKFLOW_SHA" = "$EXPECTED_SOURCE_SHA"',
    'test -z "$(git status --porcelain)"',
  ])
    assert.ok(admission.run?.includes(comparison), comparison);
  const runs = steps.filter((step) => step.run?.includes("--candidate-only"));
  assert.equal(runs.length, 1);
  const run = runs[0];
  assert.equal(run["continue-on-error"], undefined);
  assert.match(run.run!, /--candidate-only --explicit-fixtures-only/u);
  assert.deepEqual(
    [...run.run!.matchAll(/--fixture ([^\s]+)/gu)].map((m) => m[1]),
    ['"$public_fixture"'],
  );
  const report = workflow.on.workflow_dispatch.inputs["published-report"];
  assert.deepEqual(report, {
    description: "Report",
    type: "choice",
    options: ["9499", "9498"],
    default: "9499",
  });
  assert.equal(run.env?.PUBLIC_REPORT, "${{ inputs.published-report }}");
  const selector = run.run!.match(/case "\$PUBLIC_REPORT" in[\s\S]*?\n\s*esac/u)?.[0];
  assert.ok(selector, "fixture selection must run before the public actor");
  assert.ok(run.run!.indexOf(selector) < run.run!.indexOf("officialActorInputs(inputs)"));
  assert.ok(run.run!.includes('sha256sum "$public_fixture"'));
  const choose = (value: string) =>
    spawnSync(
      "bash",
      ["-eu", "-o", "pipefail", "-c", `${selector}\nprintf '%s' "$public_fixture"`],
      {
        encoding: "utf8",
        env: { ...process.env, PUBLIC_REPORT: value },
      },
    );
  for (const [value, fixture] of [
    [String(report.default), "tests/_fixtures/content-mapper/with-defaults-surplus.json"],
    ["9498", "tests/_fixtures/content-mapper/escaton-public-props-9498.json"],
  ]) {
    const selected = choose(value);
    assert.equal(selected.status, 0, selected.stderr);
    assert.equal(selected.stdout, fixture);
    assert.equal(selected.stderr, "");
  }
  for (const value of [
    "",
    "9497",
    "9499/../9498",
    "../fixture.json",
    "9498; echo injected",
    " 9498",
  ]) {
    const selected = choose(value);
    assert.equal(selected.status, 2, value);
    assert.equal(selected.stdout, "");
    assert.equal(selected.stderr, "Unsupported published report\n");
  }
  assert.match(
    run.run!,
    /--public-version "\$\{PUBLIC_TAG#v\}" --public-inputs "\$PUBLIC_INPUTS"/u,
  );
  assert.match(run.run!, /officialActorInputs\(inputs\)/u);
  assert.equal(run.env?.PUBLIC_ACTOR, "${{ github.workflow_sha }}");
  for (const [key, input] of Object.entries({
    PUBLIC_CUT: "cut",
    PUBLIC_HEAD: "head",
    PUBLIC_TAG: "tag",
    PUBLIC_SOURCE_PR: "source_pr",
    PUBLIC_RELEASE_RUN: "release_run",
  })) {
    assert.equal(run.env?.[key], `\${{ inputs.${input} }}`);
    assert.equal(workflow.on.workflow_dispatch.inputs[input].type, "string");
  }
  assert.equal(Object.hasOwn(workflow.on.workflow_dispatch.inputs, "actor"), false);
  const upload = steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"))!;
  assert.equal(upload.if, "always()");
  assert.equal(upload.with?.["include-hidden-files"], true);
  assert.equal(upload.with?.["if-no-files-found"], "error");
  assert.match(String(upload.with?.path), /official-css-source\/[\s\S]*official-css-options\//u);
  assert.ok(steps.indexOf(upload) > steps.indexOf(run));
});

test("published mapper admission rejects mixed scopes, tags, runtime overrides and incomplete tuples", () => {
  const job = publishedWorkflow().jobs["official-css-options"];
  const guard = job.steps.find(
    (step) => step.name === "Require the explicit public scope and absent runtime overrides",
  )!;
  const focused = job.steps.find(
    (step) => step.name === "Require an unambiguous focused campaign",
  )!;
  const script = `${guard.run}\n${focused.run}`;
  const env = {
    PATH: process.env.PATH,
    SOURCE_MODE: "false",
    GITHUB_REF: "refs/heads/reviewed-actor",
    EVENT_RENAME_ONLY: "false",
  };
  const execute = (extra = {}) =>
    spawnSync("bash", ["-euo", "pipefail", "-c", script], {
      env: { ...env, ...extra },
      encoding: "utf8",
    });
  assert.equal(execute().status, 0);
  for (const extra of [
    { SOURCE_MODE: "true" },
    { EVENT_RENAME_ONLY: "true" },
    { GITHUB_REF: "refs/tags/v0.1.0" },
  ])
    assert.notEqual(execute(extra).status, 0);
  const overrides = /for key in (.+); do/u.exec(guard.run!)![1].split(" ");
  assert.ok(overrides.includes("NODE_OPTIONS") && overrides.includes("CORSA_PATH"));
  for (const key of overrides) assert.notEqual(execute({ [key]: "forbidden" }).status, 0, key);
  const inputs: Record<string, string> = {
    PUBLIC_CUT: "a".repeat(40),
    PUBLIC_HEAD: "b".repeat(40),
    PUBLIC_ACTOR: "c".repeat(40),
    PUBLIC_TAG: "v0.1.0",
    PUBLIC_SOURCE_PR: "1",
    PUBLIC_RELEASE_RUN: "2",
  };
  assert.equal(officialActorInputs(inputs).actor, inputs.PUBLIC_ACTOR);
  for (const key of Object.keys(inputs)) {
    const incomplete = { ...inputs };
    delete incomplete[key];
    assert.throws(() => officialActorInputs(incomplete), key);
  }
});
