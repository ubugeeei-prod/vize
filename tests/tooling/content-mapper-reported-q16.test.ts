import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import test from "node:test";
import { parse } from "yaml";
import {
  declaredBin,
  verifyQ16PublishedLauncher,
  q16PublishedLauncherPin,
  q16Cases,
  q16Package,
  verifyQ16Execution,
  verifyQ16Inputs,
  verifyQ16List,
  verifyQ16Lock,
  verifyQ16RequiredEnvironment,
} from "../../tools/support/compat/content-mapper/reported-q16.ts";

const workflow = fs.readFileSync(
  new URL("../../.github/workflows/content-mapper-conformance.yml", import.meta.url),
  "utf8",
);
const data = parse(workflow);
test("the complete original workflow is restored by only the explicit additive route inverse", () => {
  const start = workflow.indexOf("  reported-q16:\n"),
    end = workflow.indexOf("  exact-tsgo-project:\n", start);
  assert(start > 0 && end > start);
  let before = workflow.slice(0, start) + workflow.slice(end);
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
    return Boolean(
      new Function("inputs", "github", `return (${body});`)(inputs, { event_name: event }),
    );
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
  const steps = data.jobs["reported-q16"].steps;
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
    steps.some((s: { with?: { [key: string]: string } }) => s.with?.["node-version"] === "26.13.0"),
  );
});

const source = "1".repeat(40);
const valid: NodeJS.ProcessEnv = {
  GITHUB_EVENT_NAME: "workflow_dispatch",
  Q16_TARGETED_TEST: "reported-q16",
  Q16_OTHER_MODES: "false",
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

test("required per-family captures and host paths cannot quietly disappear", () => {
  const e: NodeJS.ProcessEnv = {};
  const booleans = [
    "VIZE_TEST_REQUIRE_TSGO",
    "VIZE_REQUIRE_REPORTED_GENERIC_COMPONENTS",
    "VIZE_TEST_REPORTED_DYNAMIC_TARGETS_REQUIRED",
    "VIZE_TEST_REQUIRE_NESTED_CALLBACK",
  ];
  const paths = [
    "CORSA_PATH",
    "VIZE_TEST_CONTENT_MAPPER_TSGO",
    "VIZE_TEST_CONTENT_MAPPER_VUE",
    "VIZE_REPORTED_MODEL_CAPTURE",
    "VIZE_REPORTED_GENERIC_EVIDENCE_DIR",
    "VIZE_TEST_REPORTED_DYNAMIC_TARGETS_CAPTURE_DIR",
    "VIZE_NESTED_CALLBACK_CAPTURE",
    "VIZE_REPORTED_DECLARATIONS_CAPTURE",
  ];
  for (const k of booleans) e[k] = "1";
  for (const k of paths) e[k] = "/owned/" + k;
  verifyQ16RequiredEnvironment(e);
  for (const k of [...booleans, ...paths])
    assert.throws(() => verifyQ16RequiredEnvironment({ ...e, [k]: undefined }));
  assert.throws(() => verifyQ16RequiredEnvironment({ ...e, VIZE_TEST_DISABLE_TSGO: "1" }));
});
