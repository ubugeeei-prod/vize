import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const workflow = parse(readFileSync(join(root, ".github/workflows/pr-source-checks.yml"), "utf8"));
const jsChecksPath = "./.github/workflows/pr-js-checks.yml";
const jsChecks = parse(readFileSync(join(root, jsChecksPath), "utf8"));
const selectedJs = "${{ needs.pr-source-plan.outputs.js == 'true' }}";
const inputJs = "${{ inputs.run-js }}";
const selectedGroup = "${{ (inputs.run-js && matrix.group != 'none') }}";
const boolean = { required: true, type: "boolean" };
const evidenceJs = "${{ !cancelled() && inputs.run-js }}";
const groupCommand =
  'node .github/actions/test-js-packages-with-history/native-fault-evidence.ts -- vp run --workspace-root "test:js:$VIZE_JS_PACKAGE_GROUP"';
const historyActionPath = "./.github/actions/test-js-packages-with-history";
const history = parse(readFileSync(join(root, historyActionPath, "action.yml"), "utf8"));

function recordingCli(directory, source = "process.exit(0);") {
  const calls = join(directory, "calls.jsonl");
  writeFileSync(
    join(directory, "vp"),
    `#!${process.execPath}\nimport fs from 'node:fs'; import path from 'node:path'; import {spawnSync} from 'node:child_process'; const args=process.argv.slice(2); fs.appendFileSync(${JSON.stringify(calls)},JSON.stringify({args,cwd:process.cwd()})+'\\n'); ${source}\n`,
    { mode: 0o755 },
  );
  return {
    calls,
    env: { ...process.env, PATH: `${directory}:${process.env.PATH}` },
    records: () =>
      existsSync(calls) ? readFileSync(calls, "utf8").trim().split("\n").map(JSON.parse) : [],
  };
}

void test("full partitions validate the actual inventory and retain every original execution guard", () => {
  const [validate, all, group, build, qualify] = history.runs.steps;
  assert.equal(validate.name, "Validate full JS partition");
  assert.deepEqual(validate.env, { VIZE_JS_PARTITION: "${{ inputs.partition }}" });
  const enabled = (step, partition, prior) =>
    runInNewContext(step.if.slice(3, -3), {
      inputs: { partition },
      job: { status: prior },
      success: () => prior === "success",
    });
  for (const partition of ["all", "builders", "ui", "oxlint", "original-project"]) {
    const result = spawnSync("/bin/bash", ["-c", validate.run], {
      cwd: root,
      env: { ...process.env, VIZE_JS_PARTITION: partition },
      encoding: "utf8",
      timeout: 10_000,
    });
    assert.equal(result.status, 0, result.stderr);
    for (const prior of ["success", "failure", "cancelled", "skipped"]) {
      const selected = [all, group, build, qualify].filter((step) =>
        enabled(step, partition, prior),
      );
      const expected =
        prior !== "success"
          ? []
          : partition === "all"
            ? [all, qualify]
            : partition === "original-project"
              ? [build, qualify]
              : [group];
      assert.deepEqual(
        selected.map(({ name }) => name),
        expected.map(({ name }) => name),
      );
    }
  }
  for (const partition of ["", "none", "all; exit 0", "builders,ui", "unknown"]) {
    const result = spawnSync("/bin/bash", ["-c", validate.run], {
      cwd: root,
      env: { ...process.env, VIZE_JS_PARTITION: partition },
      encoding: "utf8",
      timeout: 10_000,
    });
    assert.equal(result.status, 1, result.stderr);
    assert.match(result.stderr, /AssertionError.*unknown full JS partition/);
  }
  assert.equal(group.run, groupCommand);
  assert.deepEqual(group.env, { VIZE_JS_PACKAGE_GROUP: "${{ inputs.partition }}" });
  assert.equal(build.run, "vp run --workspace-root build:oxlint-qualification");
  for (const step of [validate, all, group, build, qualify])
    assert.equal(step["continue-on-error"], undefined);
  assert.deepEqual(qualify.env, { VIZE_N8N_BASELINE_MODE: "partitioned" });
});

void test("PR and merge queue preserve package tests, native declaration, type and complete UI checks", () => {
  assert.deepEqual(workflow.jobs["pr-js-packages"], {
    name: "PR JS packages",
    if: "${{ github.event_name == 'pull_request' || github.event_name == 'merge_group' }}",
    needs: "pr-source-plan",
    uses: jsChecksPath,
    with: {
      "run-js": selectedJs,
      "run-original-project": false,
      "js-plan": "${{ needs.pr-source-plan.outputs.js-plan }}",
    },
  });
  assert.deepEqual(jsChecks.on.workflow_call.inputs["run-js"], boolean);
  assert.deepEqual(jsChecks.on.workflow_call.inputs["run-original-project"], boolean);
  const jsPlanInput = jsChecks.on.workflow_call.inputs["js-plan"];
  assert.equal(jsPlanInput.type, "string");
  assert.equal(
    jsChecks.jobs["pr-js-package-tests"].strategy.matrix,
    "${{ fromJSON(inputs.js-plan).matrix }}",
  );
  assert.deepEqual(Object.keys(jsChecks.jobs), [
    "pr-js-package-tests",
    "pr-oxlint-project-qualification",
    "pr-js-package-conformance",
  ]);
  for (const [name, job] of Object.entries(jsChecks.jobs)) {
    // Every costly step is selected by the input plan. Original-project
    // qualification is available explicitly and runs in full scheduled Check.
    const tier = name === "pr-oxlint-project-qualification";
    assert.equal(job.if, tier ? "${{ inputs.run-original-project }}" : undefined, name);
    assert.equal(job.needs, undefined, name);
    const skip =
      name === "pr-js-package-tests"
        ? "${{ !(inputs.run-js && matrix.group != 'none') }}"
        : name === "pr-js-package-conformance"
          ? "${{ !(inputs.run-js && fromJSON(inputs.js-plan).conformance[matrix.surface]) }}"
          : "${{ !inputs.run-js }}";
    assert.equal(job.steps[0].if, skip, name);
    assert.match(job.steps[0].run, /skipped because this PR changes no package/, name);
    for (const step of job.steps.slice(1)) assert.ok(step.if, `${name}: ${step.name ?? step.uses}`);
  }
  const testStep = jsChecks.jobs["pr-js-package-tests"].steps.find(
    (step) => step.name === "Test JS package group",
  );
  assert.equal(testStep.if, selectedGroup);
  assert.equal(testStep.run, groupCommand);
  assert.deepEqual(testStep.env, { VIZE_JS_PACKAGE_GROUP: "${{ matrix.group }}" });
  const names = [
    "Check Fresco native declarations and consumer types",
    "Check UI package conformance",
  ];
  const steps = jsChecks.jobs["pr-js-package-conformance"].steps.filter((step) =>
    names.includes(step.name),
  );
  assert.deepEqual(
    steps.map((step) => step.name),
    names,
  );
  assert.deepEqual(
    steps.map((step) => step.if),
    [
      "${{ inputs.run-js && matrix.surface == 'fresco' && fromJSON(inputs.js-plan).conformance.fresco }}",
      "${{ inputs.run-js && matrix.surface == 'ui' && fromJSON(inputs.js-plan).conformance.ui }}",
    ],
  );
  assert.deepEqual(jsChecks.jobs["pr-js-package-conformance"].strategy.matrix.surface, [
    "fresco",
    "ui",
  ]);
  assert.ok(workflow.jobs["source-report"].needs.includes("pr-js-packages"));
  for (const result of ["failure", "cancelled", "skipped"]) {
    assert.equal(aggregateNeedsResults({ "pr-js-packages": { result } }).exitCode, 1);
  }
  const groups = JSON.parse(jsPlanInput.default).matrix.group;
  const fixture = mkdtempSync(join(tmpdir(), "vize-js-tier-"));
  try {
    const cli = recordingCli(fixture);
    for (const [event, js] of [
      ["pull_request", true],
      ["merge_group", true],
      ["pull_request", false],
      ["merge_group", false],
    ]) {
      rmSync(cli.calls, { force: true });
      const commands = js
        ? [
            ...groups.map((group) => ({ command: testStep.run, group })),
            ...steps.map((step) => ({ command: step.run })),
          ]
        : [];
      for (const { command, group } of commands) {
        const result = spawnSync("/bin/sh", ["-c", command], {
          cwd: root,
          env: { ...cli.env, ...(group ? { VIZE_JS_PACKAGE_GROUP: group } : {}) },
          encoding: "utf8",
        });
        assert.equal(result.status, 0, `${event}: ${result.stdout}\n${result.stderr}`);
      }
      const expected = js
        ? [
            ...groups.map((group) => ["run", "--workspace-root", `test:js:${group}`]),
            ["run", "--filter", "./npm/fresco-native", "check:generated"],
            ["run", "--filter", "./npm/fresco-native", "check:types"],
            ["run", "--filter", "./npm/native", "build:ci"],
            ["run", "--filter", "./npm/ui", "check"],
          ]
        : [];
      assert.deepEqual(
        cli.records().map((record) => record.args),
        expected,
      );
    }
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});

void test("public native evidence remains reachable after prior build or package test failure", () => {
  // Scheduled and manual qualification retain the complete package history.
  assert.equal(history.runs.using, "composite");
  assert.equal(history.runs.steps.length, 10);
  assert.equal(history.inputs.partition.default, "all");
  const [, execute, , , qualification, upload, style, oxlint, cleanup, fault] = history.runs.steps;
  assert.equal(
    execute.if,
    "${{ success() && job.status == 'success' && inputs.partition == 'all' }}",
  );
  assert.equal(execute.shell, "bash");
  assert.equal(
    execute.run,
    "node .github/actions/test-js-packages-with-history/native-fault-evidence.ts -- vp run --workspace-root test:js",
  );
  assert.equal(execute["continue-on-error"], undefined);
  assert.equal(
    qualification.if,
    "${{ success() && job.status == 'success' && (inputs.partition == 'all' || inputs.partition == 'original-project') }}",
  );
  assert.equal(qualification.shell, "bash");
  assert.equal(qualification.run, "node npm/oxlint/scripts/check-project-transport.mjs");
  assert.equal(qualification["continue-on-error"], undefined);
  assert.equal(qualification.id, "original-project-qualification");
  assert.equal(upload.if, "${{ always() }}");
  assert.equal(upload.uses, "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
  assert.deepEqual(upload.with, {
    name: "public-native-formatter-${{ github.sha }}-${{ github.run_attempt }}-${{ github.job }}-${{ inputs.partition }}",
    path: "npm/native/.artifacts/native/formatter-history/**",
    "if-no-files-found": "warn",
    "retention-days": 14,
  });
  assert.equal(style.if, "${{ always() }}");
  assert.equal(style.uses, upload.uses);
  assert.deepEqual(style.with, {
    name: "template-style-fixes-${{ github.sha }}-${{ github.run_attempt }}-${{ github.job }}-${{ inputs.partition }}",
    path: "target/template-style-fixes-7905.json",
    "if-no-files-found": "warn",
    "retention-days": 14,
  });
  assert.equal(oxlint.if, "${{ always() }}");
  assert.equal(oxlint.id, "original-project-evidence");
  assert.equal(oxlint.uses, upload.uses);
  assert.deepEqual(oxlint.with, {
    name: "oxlint-original-locations-${{ github.sha }}-${{ github.run_attempt }}-${{ github.job }}-${{ inputs.partition }}",
    path: [
      "target/oxlint-original-locations-7904.json",
      "target/oxlint-original-project-checks-7903.json",
      "target/oxlint-script-safe-carrier-7903.json",
      "target/oxlint-original-project-transport/**",
      "target/oxlint-original-html-binding-7903/**",
      "target/vize-tests/original-sfc-host-7004-7005-*/**",
      "",
    ].join("\n"),
    "if-no-files-found": "warn",
    "retention-days": 14,
  });
  assert.equal(
    cleanup.if,
    "${{ always() && steps.original-project-qualification.outcome == 'success' && steps.original-project-evidence.outcome == 'success' }}",
  );
  assert.equal(cleanup.shell, "bash");
  assert.equal(cleanup.run, "node npm/oxlint/scripts/project-native-staging.ts --cleanup");
  assert.deepEqual(cleanup.env, {
    VIZE_OXLINT_STAGING_DIRECTORY:
      "${{ steps.original-project-qualification.outputs.staging-directory }}",
  });
  assert.equal(cleanup["continue-on-error"], undefined);
  assert.equal(fault.if, "${{ always() }}");

  // Each PR package group uploads the same evidence under a per-group name.
  const groupJob = jsChecks.jobs["pr-js-package-tests"];
  const groupStep = (name) => groupJob.steps.find((step) => step.name === name);
  const perGroup = (name) => name.replace("${{ inputs.partition }}", "${{ matrix.group }}");
  const stagingPath = "target/oxlint-original-project-transport/**\n";
  for (const original of [upload, style, oxlint, fault]) {
    const step = groupStep(original.name.replace("bridge locations", "package locations"));
    assert.ok(step, original.name);
    assert.equal(step.if, "${{ !cancelled() && (inputs.run-js && matrix.group != 'none') }}");
    assert.equal(step.uses, original.uses);
    assert.equal(step["continue-on-error"], undefined);
    assert.deepEqual(step.with, {
      ...original.with,
      name: perGroup(original.with.name),
      // Only the qualification job writes project qualification staging.
      path: original.with.path.replace(stagingPath, ""),
    });
  }
  assert.equal(
    new Set(groupJob.steps.flatMap((step) => step.with?.name ?? [])).size,
    groupJob.steps.filter((step) => step.with?.name).length,
  );

  // The qualification job keeps the original command, guards and evidence.
  const qualify = jsChecks.jobs["pr-oxlint-project-qualification"].steps;
  const index = (predicate) => qualify.findIndex(predicate);
  const build = index((step) => step.run === "vp run --workspace-root build:oxlint-qualification");
  const run = index((step) => step.id === qualification.id);
  const native = index((step) => step.name === upload.name);
  const evidence = index((step) => step.id === oxlint.id);
  const clean = index((step) => step.name === cleanup.name);
  assert.ok(0 < build && build < run && run < native && native < evidence && evidence < clean);
  assert.equal(qualify[build].if, inputJs);
  assert.deepEqual(
    { ...qualify[run] },
    {
      name: qualification.name,
      id: qualification.id,
      if: inputJs,
      env: qualification.env,
      run: qualification.run,
    },
  );
  for (const [step, original] of [
    [qualify[native], upload],
    [qualify[evidence], oxlint],
  ]) {
    assert.equal(step.if, evidenceJs);
    assert.equal(step.uses, original.uses);
    assert.deepEqual(step.with, {
      ...original.with,
      name: original.with.name.replace("-${{ inputs.partition }}", ""),
    });
  }
  assert.equal(qualify[evidence].id, oxlint.id);
  assert.deepEqual(
    { ...qualify[clean] },
    { name: cleanup.name, if: cleanup.if, env: cleanup.env, run: cleanup.run },
  );
});

void test("Fresco checks generate declarations and invoke static TypeScript without executing the loader", () => {
  const fixture = mkdtempSync(join(tmpdir(), "vize-fresco-static-"));
  const packageDir = join(fixture, "package");
  const originalPackage = join(root, "npm/fresco-native");
  const tsc = createRequire(join(originalPackage, "package.json")).resolve("typescript/bin/tsc");
  try {
    for (const file of [
      "package.json",
      "index.js",
      "index.d.ts",
      "tsconfig.types.json",
      "tests/types/consumer.ts",
      "scripts/package.json",
      "scripts/verify-consumer-types.ts",
    ]) {
      mkdirSync(dirname(join(packageDir, file)), { recursive: true });
      copyFileSync(join(originalPackage, file), join(packageDir, file));
    }
    const cli = recordingCli(
      fixture,
      `
if(args[0]==='exec' && args[1]==='napi') {
  const out=args[args.indexOf('--output-dir')+1]; fs.mkdirSync(out,{recursive:true});
  for(const name of ['index.js','index.d.ts']) fs.copyFileSync(path.join(process.cwd(),name),path.join(out,name));
} else if(args[0]==='exec' && args[1]==='tsc') {
  process.exit(spawnSync(process.execPath,[${JSON.stringify(tsc)},...args.slice(2)],{stdio:'inherit'}).status ?? 1);
} else { throw new Error('unexpected CLI'); }
`,
    );
    const generated = spawnSync(
      process.execPath,
      [
        join(originalPackage, "scripts/verify-generated-contract.mjs"),
        "--package-dir",
        packageDir,
        "--profile",
        "ci",
      ],
      { env: cli.env, encoding: "utf8" },
    );
    assert.equal(generated.status, 0, `${generated.stdout}\n${generated.stderr}`);
    const generation = cli.records()[0];
    assert.equal(realpathSync(generation.cwd), realpathSync(packageDir));
    assert.deepEqual(generation.args.slice(0, -1), [
      "exec",
      "napi",
      "build",
      "--platform",
      "--profile",
      "ci",
      "--manifest-path",
      "../../crates/vize_fresco/Cargo.toml",
      "-p",
      "vize_fresco",
      "--features",
      "napi",
      "--output-dir",
    ]);
    assert.equal(existsSync(generation.args.at(-1)), false);
    writeFileSync(
      join(packageDir, "index.js"),
      "throw new Error('runtime loader must not execute');\n",
    );
    const checkTypes = () =>
      spawnSync(process.execPath, [join(packageDir, "scripts/verify-consumer-types.ts")], {
        env: cli.env,
        encoding: "utf8",
      });
    const typed = checkTypes();
    assert.equal(typed.status, 0, `${typed.stdout}\n${typed.stderr}`);
    const typeCall = cli.records()[1];
    assert.equal(realpathSync(typeCall.cwd), realpathSync(packageDir));
    assert.deepEqual(typeCall.args.slice(0, -1), ["exec", "tsc", "--noEmit", "-p"]);
    assert.equal(existsSync(typeCall.args.at(-1)), false);
    writeFileSync(
      join(packageDir, "tests/types/consumer.ts"),
      "export const broken: string = 1;\n",
    );
    const invalidTypes = checkTypes();
    assert.notEqual(invalidTypes.status, 0);
    assert.match(invalidTypes.stderr, /Fresco native consumer type check failed/);
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});
