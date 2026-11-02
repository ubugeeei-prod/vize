import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import test from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import {
  declaredBin,
  verifyQ16PublishedLauncher,
  q16PublishedLauncherPin,
  q16Cases,
  selectQ16Cases,
  compiledQ16Artifacts,
  attestQ16CompiledCli,
  q16Package,
  verifyQ16Execution,
  verifyQ16Inputs,
  verifyQ16List,
  verifyQ16Lock,
} from "../../tools/support/compat/content-mapper/reported-q16.ts";

const actualWorkflow = fs.readFileSync(
  new URL("../../.github/workflows/content-mapper-conformance.yml", import.meta.url),
  "utf8",
);
// Keep the historical whole-workflow inverses independent of this command-local
// provider declaration and its child under the existing owner artifact path.
const standardCommand =
  "cargo test -p vize --test integration -- content_mapper_tsgo_cli:: --nocapture";
const standardRoleCommand =
  'VIZE_TEST_REPORTED_DYNAMIC_TARGETS_PROVIDER_ROLE=standard-go-d6-vue35 VIZE_TEST_REPORTED_DYNAMIC_TARGETS_CAPTURE_DIR="$GITHUB_WORKSPACE/target/vize-tests/content-mapper-directive-owners/reported-dynamic-targets" ' +
  standardCommand;
const syntaxCommand =
  "cargo test -p vize --test integration -- content_mapper_tsgo_cli::template_expression_syntax:: --nocapture";
const modelEventsCommand = syntaxCommand.replace(
  " --nocapture",
  " content_mapper_tsgo_cli::reported_model_events_11390_11391:: --nocapture",
);
const staticRefCommand = modelEventsCommand.replace(
  " --nocapture",
  " content_mapper_tsgo_cli::dynamic_template_refs::standard_tsgo_static_template_refs_read_runtime_bindings_and_keep_true_unused --nocapture",
);
const receiverFilter = " content_mapper_tsgo_cli::reported_expression_duplicates_11374::";
const receiverAdjustedWorkflow = actualWorkflow.replace(receiverFilter, "");
const workflow = receiverAdjustedWorkflow
  .replace(standardRoleCommand, standardCommand)
  .replace(staticRefCommand, modelEventsCommand)
  .replace(modelEventsCommand, syntaxCommand);
const data = parse(workflow);
const childWorkflow = fs.readFileSync(
  new URL("../../.github/workflows/content-mapper-reported-q16.yml", import.meta.url),
  "utf8",
);
const childData = parse(childWorkflow);
const delegatedCalls = {
  "reported-q16":
    "  reported-q16:\n    if: github.event_name == 'workflow_dispatch' && inputs.targeted-test == 'reported-q16'\n    uses: ./.github/workflows/content-mapper-reported-q16.yml\n    with:\n      request: ${{ toJSON(inputs) }}\n\n",
  "router-experimental":
    "  router-experimental:\n    if: github.event_name == 'schedule' || (github.event_name == 'workflow_dispatch' && inputs.targeted-test != 'generic-body-error' && inputs.targeted-test != 'reported-q16' && inputs.router-experimental-only)\n    uses: ./.github/workflows/content-mapper-reported-q16.yml\n    with:\n      request: ${{ toJSON(inputs) }}\n",
};
function restoreDelegatedJobs(value: string) {
  value = value.replace(
    'options: [all, A, A-with-declaration, B, C, D, "11373"]',
    "options: [all, A, A-with-declaration, B, C, D]",
  );
  const names = Object.keys(delegatedCalls) as (keyof typeof delegatedCalls)[];
  for (const [i, name] of names.entries()) {
    const start = childWorkflow.indexOf("  " + name + ":\n"),
      end =
        i + 1 < names.length
          ? childWorkflow.indexOf("  " + names[i + 1] + ":\n", start)
          : childWorkflow.length;
    assert(start > 0 && end > start);
    let body = childWorkflow
      .slice(start, end)
      .replace(
        'with: { node-version: "${{ env.Q16_PROVIDER_NODE }}", package-manager-cache: false }',
        'with: { node-version: "26.13.0", package-manager-cache: false }',
      );
    for (const key of [
      "targeted-test",
      "expected-source-sha",
      "router-experimental-only",
      "event-rename-only",
      "official-css-options-only",
      "published-defaults-only",
      "cut",
      "head",
      "tag",
      "source_pr",
      "release_run",
      "reported-q",
    ])
      body = body.replaceAll("fromJSON(inputs.request)." + key, "inputs." + key);
    assert.equal(value.split(delegatedCalls[name]).length, 2);
    value = value.replace(delegatedCalls[name], body);
  }
  return value;
}

function stripSelectedAfter(value: string) {
  value = restoreDelegatedJobs(value);
  const input =
      '      reported-q16-cases:\n        description: "Closed source After case set; all retains the original six tests"\n        type: choice\n        options: [all, A, A-with-declaration, B, C, D]\n        default: all\n',
    from =
      '      - name: Initialize runner-local joint evidence paths\n        shell: bash\n        run: |\n          printf \'Q16_OUT=%s/reported-q16\\nQ16_PROVIDER=%s/reported-q16-provider\\n\' "$RUNNER_TEMP" "$RUNNER_TEMP" >> "$GITHUB_ENV"',
    to =
      '      - name: Initialize runner-local joint evidence paths\n        shell: bash\n        env:\n          Q16_REQUESTED_CASE_SET: ${{ inputs.reported-q16-cases }}\n        run: |\n          printf \'Q16_OUT=%s/reported-q16\\nQ16_PROVIDER=%s/reported-q16-provider\\nQ16_CASE_SET=%s\\n\' "$RUNNER_TEMP" "$RUNNER_TEMP" "$Q16_REQUESTED_CASE_SET" >> "$GITHUB_ENV"';
  assert.equal(value.split(input).length, 2);
  assert.equal(value.split(to).length, 2);
  return value.replace(input, "").replace(to, from);
}
test("the selected After additions invert to the whole current 9fef workflow including GITHUB_ENV repair", () => {
  const before = stripSelectedAfter(workflow);
  assert.equal(Buffer.byteLength(before), 51643);
  assert.equal(
    createHash("sha256").update(before).digest("hex"),
    "a1abe8b529c1ac31c56306a8efa42269d09a5f62a27fdeb48997be6c2cc893c6",
  );
  assert.deepEqual(data.on.workflow_dispatch.inputs["reported-q16-cases"].options, [
    "all",
    "A",
    "A-with-declaration",
    "B",
    "C",
    "D",
    "11373",
  ]);
  assert.equal(data.on.workflow_dispatch.inputs["reported-q16-cases"].default, "all");
});

test("the complete original workflow is restored by only the explicit additive route inverse", () => {
  const withoutSelection = stripSelectedAfter(workflow);
  const start = withoutSelection.indexOf("  reported-q16:\n"),
    end = withoutSelection.indexOf("  exact-tsgo-project:\n", start);
  assert(start > 0 && end > start);
  let before = withoutSelection.slice(0, start) + withoutSelection.slice(end);
  const edits = [
    ["options: [full, generic-body-error, reported-q16]", "options: [full, generic-body-error]"],
    [
      "inputs.targeted-test == 'reported-q16' && '-reported-q16' || inputs.router-experimental-only && '-router-experimental'",
      "inputs.router-experimental-only && '-router-experimental'",
    ],
    [
      "if: ${{ inputs.targeted-test != 'reported-q16' && (inputs.targeted-test == 'generic-body-error' || (!inputs.router-experimental-only && !inputs.event-rename-only && !inputs.official-css-options-only && !inputs.published-defaults-only)) }}",
      "if: ${{ inputs.targeted-test == 'generic-body-error' || (!inputs.router-experimental-only && !inputs.event-rename-only && !inputs.official-css-options-only && !inputs.published-defaults-only) }}",
    ],
  ];
  for (const [a, b] of edits) {
    assert.equal(before.split(a).length, 2);
    before = before.replace(a, b);
  }
  const exclusion =
    "github.event_name == 'workflow_dispatch' && inputs.targeted-test != 'generic-body-error' && inputs.targeted-test != 'reported-q16'";
  assert.equal(before.split(exclusion).length - 1, 3);
  before = before.replaceAll(
    exclusion,
    "github.event_name == 'workflow_dispatch' && inputs.targeted-test != 'generic-body-error'",
  );
  assert.equal(Buffer.byteLength(before), 47076);
  assert.equal(
    createHash("sha256").update(before).digest("hex"),
    "5a2294bfc380438a56c5fcfec44b4f1c22c8756987d26176fb2fa555c71c4651",
  );
  assert.equal(data.on.pull_request, undefined); // schedule/manual default remains unchanged
  assert.deepEqual(data.on.workflow_dispatch.inputs["targeted-test"].options, [
    "full",
    "generic-body-error",
    "reported-q16",
  ]);
  assert.equal(data.on.workflow_dispatch.inputs["reported-role"], undefined);
});

test("actual job conditions preserve every old-mode truth-table row and isolate joint-only mode", () => {
  const evaluate = (expression: string, event: string, mode: string, flags: boolean[]) => {
    const inputs = {
      "targeted-test": mode,
      "router-experimental-only": flags[0],
      "event-rename-only": flags[1],
      "official-css-options-only": flags[2],
      "published-defaults-only": flags[3],
    };
    const body = expression
      .replace(/^\$\{\{\s*|\s*\}\}$/gu, "")
      .replace(
        /inputs\.([a-z-]+)/gu,
        (_, key: keyof typeof inputs) => `inputs[${JSON.stringify(key)}]`,
      );
    return Boolean(runInNewContext(`(${body})`, { inputs, github: { event_name: event } }));
  };
  for (const event of ["schedule", "workflow_dispatch"])
    for (let bits = 0; bits < 16; bits++)
      for (const mode of ["full", "generic-body-error", "reported-q16"]) {
        const f = [0, 1, 2, 3].map((i) => Boolean(bits & (1 << i))),
          targeted = mode === "generic-body-error",
          joint = mode === "reported-q16";
        const old = {
          "exact-tsgo-project": targeted || !f.some(Boolean),
          "event-rename":
            event === "workflow_dispatch" && !targeted && !f[0] && f[1] && !f[2] && !f[3],
          "official-css-options":
            event === "workflow_dispatch" && !targeted && !f[0] && (f[2] || f[3]),
          "router-experimental":
            event === "schedule" || (event === "workflow_dispatch" && !targeted && f[0]),
        };
        for (const [name, value] of Object.entries(old)) {
          // A scheduled event has no manual targeted input; the hypothetical reported/schedule row retains scheduled Router behavior.
          const expected = joint ? name === "router-experimental" && event === "schedule" : value;
          assert.equal(
            evaluate(data.jobs[name].if, event, mode, f),
            expected,
            `${name}/${event}/${mode}/${bits}`,
          );
        }
        assert.equal(
          evaluate(data.jobs["reported-q16"].if, event, mode, f),
          event === "workflow_dispatch" && joint,
        );
      }
  const steps = childData.jobs["reported-q16"].steps;
  assert.equal(
    steps.filter(
      (s: { run?: string }) =>
        s.run === "node tools/support/compat/content-mapper/reported-q16.ts build",
    ).length,
    1,
  );
  assert(!JSON.stringify(steps).includes("go build"));
  assert.equal(steps.at(-1).if, "always()");
  assert.equal(steps.at(-1).with["if-no-files-found"], "error");
  assert(
    steps.some(
      (s: { with?: { [key: string]: string } }) =>
        s.with?.["node-version"] === "${{ env.Q16_PROVIDER_NODE }}",
    ),
  );
});

test("dynamic-target printer roles are declared before capture and retain standard whole packets", () => {
  assert.equal(actualWorkflow.split(standardRoleCommand).length, 2);
  assert.equal(receiverAdjustedWorkflow.split(staticRefCommand).length, 2);
  assert.equal(actualWorkflow.split(receiverFilter).length, 2);
  const actual = parse(actualWorkflow);
  const steps = actual.jobs["exact-tsgo-project"].steps;
  const standard = steps.find(
    (step: { name?: string }) => step.name === "Run standard project and declaration conformance",
  );
  assert.equal(standard.run.split("\n")[0], standardRoleCommand);
  const artifact = steps.find(
    (step: { name?: string }) => step.name === "Preserve whole owner calibration observations",
  );
  assert(artifact.uses.startsWith("actions/upload-artifact@"));
  assert.equal(artifact.if, "always()");
  assert(
    artifact.with.path.split("\n").includes("target/vize-tests/content-mapper-directive-owners"),
  );
});

const source = "1".repeat(40);
const valid: NodeJS.ProcessEnv = {
  GITHUB_EVENT_NAME: "workflow_dispatch",
  Q16_TARGETED_TEST: "reported-q16",
  Q16_OTHER_MODES: "false",
  Q16_CASE_SET: "all",
  EXPECTED_SOURCE_SHA: source,
  GITHUB_SHA: source,
  GITHUB_WORKFLOW_SHA: source,
};
test("mixed public/targeted modes, wrong actual source and silent skip environments refuse", () => {
  verifyQ16Inputs(valid, source);
  for (const key of ["GITHUB_SHA", "GITHUB_WORKFLOW_SHA", "EXPECTED_SOURCE_SHA"])
    assert.throws(() => verifyQ16Inputs({ ...valid, [key]: "2".repeat(40) }, source));
  for (const e of [
    { ...valid, Q16_OTHER_MODES: "true" },
    { ...valid, Q16_TARGETED_TEST: "full" },
    { ...valid, Q16_PUBLIC_TAG: "v0.638.0" },
    { ...valid, VIZE_TEST_DISABLE_TSGO: "1" },
    { ...valid, NODE_OPTIONS: "--import=other" },
  ])
    assert.throws(() => verifyQ16Inputs(e, source));
});

test("six exact compiled tests are mandatory and a real baseline semantic failure stays failure", () => {
  const names = q16Cases.map(([, name]) => `${name}: test`).join("\n");
  verifyQ16List(names);
  for (const [, name] of q16Cases)
    assert.throws(() => verifyQ16List(names.replace(`${name}: test`, "")));
  assert.throws(() => verifyQ16List(names + `\n${q16Cases[0][1]}: test`));
  assert(
    verifyQ16Execution(
      "\nrunning 1 test\ntest result: ok. 1 passed; 0 failed; 0 ignored; 400 filtered out;\n",
      0,
    ),
  );
  assert.equal(
    verifyQ16Execution(
      "\nrunning 1 test\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 400 filtered out;\n",
      101,
    ),
    false,
  );
  assert.throws(() =>
    verifyQ16Execution("running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored;", 0),
  );
});

test("the complete canonical JavaScript launcher/manifest/SRI is accepted without a native backend theorem", () => {
  assert.equal(declaredBin({ name: "tsrs", bin: { tsrs: "bin/tsrs" } }, "tsrs"), "bin/tsrs");
  assert.throws(() => declaredBin({ name: "typescript", bin: { tsc: "bin/tsc" } }, "tsrs"));
  assert.throws(() => declaredBin({ name: "tsrs", bin: "../other" }, "tsrs"));
  const launcher = Buffer.from(
    '#!/usr/bin/env node\n// CommonJS (bin/package.json) so Node 16 runs this extensionless file; Node < 18.19/20.10 refuses extensionless\n// ES modules in a "type": "module" package. The launcher itself is ESM.\n"use strict";\nimport("../lib/tsrs.js").catch(e => {\n    console.error(`tsrs: ${e && e.stack || e}`);\n    process.exit(1);\n});\n',
  );
  const manifest = Buffer.from(
    '{\n  "name": "tsrs",\n  "version": "0.20.1-ts7.1.0-beta",\n  "license": "Apache-2.0",\n  "description": "A Rust port of the TypeScript 7 compiler (tsc), language server, and JavaScript API",\n  "keywords": [\n    "TypeScript",\n    "typecheck",\n    "tsc",\n    "rust",\n    "compiler",\n    "api"\n  ],\n  "repository": {\n    "type": "git",\n    "url": "git+https://github.com/oxc-project/tsrs.git"\n  },\n  "type": "module",\n  "preferUnplugged": true,\n  "engines": {\n    "node": ">=16.20.0"\n  },\n  "files": [\n    "bin",\n    "tsdk",\n    "lib",\n    "schemas",\n    "dist",\n    "vendor",\n    "UPSTREAM.json",\n    "NOTICE.txt"\n  ],\n  "bin": {\n    "tsrs": "./bin/tsrs"\n  },\n  "exports": {\n    "./package.json": "./package.json",\n    "./schemas/tsconfig.schema.json": "./schemas/tsconfig.schema.json",\n    "./schemas/jsconfig.schema.json": "./schemas/jsconfig.schema.json",\n    ".": {\n      "types": "./lib/version.d.cts",\n      "default": "./lib/version.cjs"\n    },\n    "./vscode": {\n      "default": "./dist/vscode/extensionApi.js"\n    },\n    "./sync": {\n      "default": "./dist/api/sync/api.js"\n    },\n    "./async": {\n      "default": "./dist/api/async/api.js"\n    },\n    "./fs": {\n      "default": "./dist/api/fs.js"\n    },\n    "./path": {\n      "default": "./dist/api/typedPaths.js"\n    },\n    "./proto": {\n      "default": "./dist/api/proto.js"\n    },\n    "./ast": {\n      "default": "./dist/ast/index.js"\n    },\n    "./ast/is": {\n      "default": "./dist/ast/is.js"\n    },\n    "./ast/factory": {\n      "default": "./dist/ast/factory.generated.js"\n    },\n    "./ast/utils": {\n      "default": "./dist/ast/utils.js"\n    },\n    "./ast/scanner": {\n      "default": "./dist/ast/scanner.js"\n    },\n    "./ast/visitor": {\n      "default": "./dist/ast/visitor.js"\n    },\n    "./ast/clone": {\n      "default": "./dist/ast/clone.js"\n    }\n  },\n  "publishConfig": {\n    "access": "public"\n  },\n  "imports": {\n    "#getExePath": "./lib/getExePath.js",\n    "#enums/*": {\n      "types": "./dist/enums/*.enum.d.ts",\n      "default": "./dist/enums/*.js"\n    },\n    "#vscode-jsonrpc/node": "./vendor/vscode-jsonrpc/lib/node/main.js"\n  },\n  "tsrs": {\n    "typescriptVersion": "7.1.0-beta",\n    "typescriptCommit": "3e9236361fa97a9e6c685f4833ac457ae2d8c50a"\n  },\n  "optionalDependencies": {\n    "@tsrs/darwin-arm64": "0.20.1-ts7.1.0-beta",\n    "@tsrs/linux-arm64-gnu": "0.20.1-ts7.1.0-beta",\n    "@tsrs/linux-arm64-musl": "0.20.1-ts7.1.0-beta",\n    "@tsrs/darwin-x64": "0.20.1-ts7.1.0-beta",\n    "@tsrs/win32-x64": "0.20.1-ts7.1.0-beta",\n    "@tsrs/linux-x64-gnu": "0.20.1-ts7.1.0-beta",\n    "@tsrs/linux-x64-musl": "0.20.1-ts7.1.0-beta"\n  }\n}',
  );
  const role = verifyQ16PublishedLauncher(
    launcher,
    manifest,
    "./bin/tsrs",
    q16PublishedLauncherPin.integrity,
  );
  assert.equal(role.binRole, "source-closed standard published JavaScript launcher");
  assert.match(role.backendIdentity, /^unclaimed/u);
  for (const bytes of [
    launcher.subarray(0, -1),
    Buffer.concat([launcher, Buffer.from(" ")]),
    Buffer.from(launcher),
  ]) {
    if (bytes.length === launcher.length) bytes[20] ^= 1;
    assert.throws(() =>
      verifyQ16PublishedLauncher(bytes, manifest, "./bin/tsrs", q16PublishedLauncherPin.integrity),
    );
  }
  assert.throws(() =>
    verifyQ16PublishedLauncher(
      launcher,
      Buffer.from("{}"),
      "./bin/tsrs",
      q16PublishedLauncherPin.integrity,
    ),
  );
  assert.throws(() =>
    verifyQ16PublishedLauncher(launcher, manifest, "./other", q16PublishedLauncherPin.integrity),
  );
  assert.throws(() => verifyQ16PublishedLauncher(launcher, manifest, "./bin/tsrs", "sha512-wrong"));
  const lock = {
    lockfileVersion: 3,
    packages: {
      "": { dependencies: q16Package.dependencies },
      "node_modules/tsrs": {
        integrity: "sha512-YQ==",
        resolved: "https://registry.npmjs.org/tsrs/-/tsrs.tgz",
      },
    },
  };
  verifyQ16Lock(lock);
  assert.throws(() => verifyQ16Lock({ ...lock, lockfileVersion: 2 }));
  assert.throws(() => verifyQ16Lock({ ...lock, packages: { "": { dependencies: {} } } }));
  assert.throws(() =>
    verifyQ16Lock({
      ...lock,
      packages: { ...lock.packages, bad: { integrity: "", resolved: "file:../other" } },
    }),
  );
});

test("closed subsets require exactly their declared compiled cases and leave original all order untouched", () => {
  const expected = {
    all: ["model", "generic", "dynamic", "nested", "declaration-unused", "declaration-fallthrough"],
    A: ["model", "declaration-unused"],
    "A-with-declaration": ["model", "declaration-unused", "declaration-fallthrough"],
    B: ["generic"],
    C: ["dynamic"],
    D: ["nested"],
    "11373": ["global-component-listener"],
  };
  for (const [key, ids] of Object.entries(expected)) {
    const selected = selectQ16Cases(key);
    assert.deepEqual(
      selected.map((r) => r[0]),
      ids,
    );
    const list = selected.map((r) => r[1] + ": test").join("\n");
    verifyQ16List(list, selected);
    assert.throws(() => verifyQ16List(list.replace(selected[0][1] + ": test", ""), selected));
  }
  assert.deepEqual(selectQ16Cases("all"), q16Cases);
  for (const bad of [undefined, null, "", "a", "all,B", "73", "unknown"])
    assert.throws(() => selectQ16Cases(bad));
  assert.throws(() => verifyQ16Inputs({ ...valid, Q16_CASE_SET: undefined }, source));
});
test("post-compile CLI image is attested from the normal Cargo binary while preserving initial build provenance", () => {
  const packageId = "path+file:///home/runner/_work/vize/vize/crates/vize#0.474.0",
    cliPath = "/home/runner/_work/vize/vize/target/ci/vize";
  const cli = {
    reason: "compiler-artifact",
    package_id: packageId,
    target: { name: "vize", kind: ["bin"] },
    profile: { test: false },
    executable: cliPath,
  };
  const harness = {
    ...cli,
    target: { name: "integration", kind: ["test"] },
    profile: { test: true },
    executable: "/home/runner/_work/vize/vize/target/ci/deps/integration-test",
  };
  const emitted = compiledQ16Artifacts([cli, harness], packageId);
  assert.equal(emitted.cli, cli);
  const initial = {
      path: cliPath,
      bytes: 263716681,
      sha256: "ceaaa1a61e1cd41ac87251977638bc3723a3126e14ae079a6ea135ff3b7c7efe",
    },
    actual = {
      path: cliPath,
      bytes: 263693177,
      sha256: "30524f0a02797976b755836271aa6f53cc9666e2a8274c3f1ae3c1f5c25454b2",
    };
  const receipt = attestQ16CompiledCli(initial, actual, cli, packageId);
  assert(receipt.changedAfterIntegrationCompile);
  assert.deepEqual(receipt.initialBuild, initial);
  assert.deepEqual(receipt.executed, actual);
  for (const messages of [
    [harness],
    [cli],
    [cli, cli, harness],
    [{ ...cli, package_id: "other" }, harness],
    [{ ...cli, profile: { test: true } }, harness],
  ])
    assert.throws(() => compiledQ16Artifacts(messages, packageId));
  assert.throws(() => attestQ16CompiledCli(initial, { ...actual, path: "/other" }, cli, packageId));
  assert.throws(() =>
    attestQ16CompiledCli(initial, actual, { ...cli, package_id: "other" }, packageId),
  );
  const runner = fs.readFileSync(
    new URL("../../tools/support/compat/content-mapper/reported-q16.ts", import.meta.url),
    "utf8",
  );
  assert(runner.includes("for (const [id, name] of selected)"));
  assert(runner.includes("identity(executedCli.path).sha256, executedCli.sha256"));
  assert(
    runner.indexOf("cliProvenance = attestQ16CompiledCli") <
      runner.indexOf("for (const [id, name] of selected)"),
  );
});

test("cohesive job extraction preserves whole former jobs, inputs and global environment without header minification", () => {
  const restored = restoreDelegatedJobs(workflow);
  assert.equal(
    createHash("sha256").update(restored).digest("hex"),
    "8c86c4c3f1a1eb8a0521f46365307e497618309ace637bf222e77871d07bc411",
  );
  const original = parse(restored);
  const unchangedOn = structuredClone(data.on);
  assert.equal(unchangedOn.workflow_dispatch.inputs["reported-q16-cases"].options.pop(), "11373");
  assert.deepEqual(unchangedOn, original.on);
  assert.deepEqual(data.env, original.env);
  assert.deepEqual(childData.env, data.env);
  assert.deepEqual(childData.permissions, data.permissions);
  assert.deepEqual(Object.keys(childData.on), ["workflow_call"]);
  assert.deepEqual(childData.on.workflow_call.inputs, {
    request: { type: "string", required: true },
  });
  for (const name of Object.keys(delegatedCalls) as (keyof typeof delegatedCalls)[]) {
    assert.deepEqual(data.jobs[name].with, { request: "${{ toJSON(inputs) }}" });
    assert.equal(data.jobs[name].uses, "./.github/workflows/content-mapper-reported-q16.yml");
    assert.equal(data.jobs[name].if, original.jobs[name].if);
  }
  assert(workflow.split("\n").length - 1 <= 726);
  assert(childWorkflow.split("\n").length - 1 < 500);
  assert.equal(
    workflow
      .slice(0, workflow.indexOf("  reported-q16:\n"))
      .replace(
        'options: [all, A, A-with-declaration, B, C, D, "11373"]',
        "options: [all, A, A-with-declaration, B, C, D]",
      ),
    restored.slice(0, restored.indexOf("  reported-q16:\n")),
  );
  for (const job of Object.keys(original.jobs).filter((key) => !(key in delegatedCalls)))
    assert.deepEqual(data.jobs[job], original.jobs[job]);
});
