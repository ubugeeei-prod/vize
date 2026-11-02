import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { parse } from "yaml";
import {
  q16Package,
  q16Profile,
  verifyQ16ProviderNode,
  verifyQ16Lock,
  verifyQ16RequiredEnvironment,
  q16Cases,
  selectQ16Cases,
  verifyQ16List,
  runQ16,
} from "../../tools/support/compat/content-mapper/reported-q16.ts";
const workflow = fs.readFileSync(
  new URL("../../.github/workflows/content-mapper-conformance.yml", import.meta.url),
  "utf8",
);
test("11373 is a closed independent profile with the existing report dispatch", () => {
  const config = parse(workflow) as {
    on: {
      workflow_dispatch: {
        inputs: {
          "reported-q16-cases": { type: unknown; options: unknown; default: unknown };
        };
      };
    };
    jobs: { "reported-q16": unknown };
  };
  const cases = config.on.workflow_dispatch.inputs["reported-q16-cases"];
  assert.equal(cases.type, "choice");
  assert.deepEqual(cases.options, ["all", "A", "A-with-declaration", "B", "C", "D", "11373"]);
  assert.equal(cases.default, "all");
  assert.deepEqual(config.jobs["reported-q16"], {
    if: "github.event_name == 'workflow_dispatch' && inputs.targeted-test == 'reported-q16'",
    uses: "./.github/workflows/content-mapper-reported-q16.yml",
    with: { request: "${{ toJSON(inputs) }}" },
  });
  for (const key of ["all", "A", "A-with-declaration", "B", "C", "D"]) {
    assert.deepEqual(q16Profile(key).package, q16Package);
    verifyQ16ProviderNode("v26.13.0", key);
    assert.throws(() => verifyQ16ProviderNode("v26.12.1", key));
  }
  const profile = q16Profile("11373");
  assert.deepEqual(profile.package.dependencies, {
    tsrs: "0.20.1-ts7.1.0-beta",
    typescript: "5.9.3",
    vue: "3.5.43",
    "vue-tsc": "3.2.6",
  });
  assert.deepEqual(profile.reported, {
    vize: "0.640.0",
    node: "26.12.1",
    platform: "macOS",
    lock: "not supplied",
  });
  verifyQ16ProviderNode("v26.12.1", "11373");
  assert.throws(() => verifyQ16ProviderNode("v26.13.0", "11373"));
  for (const value of [undefined, "", "11373,B", "latest"]) assert.throws(() => q16Profile(value));
  const lock = {
    lockfileVersion: 3,
    packages: { "": { dependencies: profile.package.dependencies } },
  };
  verifyQ16Lock(lock, profile.package.dependencies);
  assert.throws(() => verifyQ16Lock(lock));
  assert.throws(() =>
    verifyQ16Lock(
      { ...lock, packages: { "": { dependencies: q16Package.dependencies } } },
      profile.package.dependencies,
    ),
  );
  const e: NodeJS.ProcessEnv = {
    VIZE_TEST_REQUIRE_TSGO: "1",
    VIZE_TEST_REQUIRE_GLOBAL_COMPONENT_LISTENER: "1",
    CORSA_PATH: "/owned/tsrs",
    VIZE_TEST_CONTENT_MAPPER_TSGO: "/owned/tsrs",
    VIZE_TEST_CONTENT_MAPPER_VUE: "/owned/vue",
    VIZE_REPORTED_GLOBAL_COMPONENT_LISTENER_CAPTURE: "/owned/capture",
  };
  verifyQ16RequiredEnvironment(e, "11373");
  for (const key of Object.keys(e))
    assert.throws(() => verifyQ16RequiredEnvironment({ ...e, [key]: undefined }, "11373"));
  assert.throws(() =>
    verifyQ16RequiredEnvironment(
      { ...e, VIZE_TEST_REQUIRE_GLOBAL_COMPONENT_LISTENER: "0" },
      "11373",
    ),
  );
  assert.throws(() => verifyQ16RequiredEnvironment({ ...e, VIZE_TEST_DISABLE_TSGO: "1" }, "11373"));
  const only = selectQ16Cases("11373");
  assert.deepEqual(only, [
    [
      "global-component-listener",
      "content_mapper_tsgo_cli::global_component_listener_11373::reported_components_and_typed_controls_keep_real_diagnostics",
    ],
  ]);
  verifyQ16List(only[0][1] + ": test", only);
  assert.throws(() => verifyQ16List(q16Cases.map((r) => r[1] + ": test").join("\n"), only));
});

test(
  "the real prepare stage writes the selected node line and complete provider inputs",
  {
    skip: process.platform !== "linux" || process.arch !== "x64",
  },
  async () => {
    const repo = fileURLToPath(new URL("../../", import.meta.url));
    const source = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: repo,
      encoding: "utf8",
    }).trim();
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-q16-prepare-"));
    const out = path.join(directory, "out");
    const provider = path.join(directory, "provider");
    const githubEnv = path.join(directory, "github-env");
    fs.writeFileSync(githubEnv, "", { flag: "wx" });
    const environment: NodeJS.ProcessEnv = {
      Q16_OUT: out,
      Q16_PROVIDER: provider,
      Q16_CASE_SET: "11373",
      GITHUB_ENV: githubEnv,
      GITHUB_EVENT_NAME: "workflow_dispatch",
      Q16_TARGETED_TEST: "reported-q16",
      Q16_OTHER_MODES: "false",
      EXPECTED_SOURCE_SHA: source,
      GITHUB_SHA: source,
      GITHUB_WORKFLOW_SHA: source,
      NODE_OPTIONS: undefined,
      NODE_PATH: undefined,
      VIZE_TEST_DISABLE_TSGO: undefined,
    };
    for (const key of ["CUT", "HEAD", "TAG", "SOURCE_PR", "RELEASE_RUN"])
      environment[`Q16_PUBLIC_${key}`] = undefined;
    const previous = new Map(
      Object.keys(environment).map((key) => [key, process.env[key]] as const),
    );
    const previousDirectory = process.cwd();
    try {
      for (const [key, value] of Object.entries(environment))
        if (value === undefined) delete process.env[key];
        else process.env[key] = value;
      process.chdir(repo);
      await runQ16("prepare"); // Git metadata and owned files only; no provider acquisition/build.
      const line = fs.readFileSync(githubEnv, "utf8");
      const manifest = JSON.parse(fs.readFileSync(path.join(provider, "package.json"), "utf8"));
      const selected = JSON.parse(fs.readFileSync(path.join(out, "case-selection.json"), "utf8"));
      console.log("vize-q16-prepare-behavior", JSON.stringify({ line, manifest, selected }));
      assert.equal(line, "Q16_PROVIDER_NODE=26.12.1\n");
      assert.deepEqual(manifest, {
        private: true,
        type: "module",
        dependencies: {
          tsrs: "0.20.1-ts7.1.0-beta",
          typescript: "5.9.3",
          vue: "3.5.43",
          "vue-tsc": "3.2.6",
        },
      });
      assert.deepEqual(selected, {
        set: "11373",
        cases: [
          [
            "global-component-listener",
            "content_mapper_tsgo_cli::global_component_listener_11373::reported_components_and_typed_controls_keep_real_diagnostics",
          ],
        ],
      });
    } finally {
      for (const [key, value] of previous)
        if (value === undefined) delete process.env[key];
        else process.env[key] = value;
      process.chdir(previousDirectory);
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);

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
  e.VIZE_TEST_REPORTED_DYNAMIC_TARGETS_PROVIDER_ROLE = "reported-tsrs-vue44";
  for (const k of booleans) e[k] = "1";
  for (const k of paths) e[k] = "/owned/" + k;
  verifyQ16RequiredEnvironment(e);
  for (const role of [undefined, "", "unknown", "standard-go-d6-vue35"])
    assert.throws(() =>
      verifyQ16RequiredEnvironment({
        ...e,
        VIZE_TEST_REPORTED_DYNAMIC_TARGETS_PROVIDER_ROLE: role,
      }),
    );
  for (const k of [...booleans, ...paths])
    assert.throws(() => verifyQ16RequiredEnvironment({ ...e, [k]: undefined }));
  assert.throws(() => verifyQ16RequiredEnvironment({ ...e, VIZE_TEST_DISABLE_TSGO: "1" }));
});
