import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import {
  bridgeProjectArgs,
  needsOriginalProjectChecks,
  replaceConfigArgs,
  splitProjectConfig,
} from "./project-checks.ts";
import { mergeProjectOutput, projectOutputFormat } from "./project-output.ts";

void test("project phases preserve whole original options and carry Vize alone across the bridge", () => {
  const config = {
    file: "/project/oxlint.config.mts",
    module: true,
    bytes: "original",
    value: {
      extends: [
        {
          plugins: ["import"],
          options: { typeAware: true },
          rules: {
            "import/no-self-import": "warn",
            "vize/vue/no-v-html": ["error", { option: true }],
          },
        },
      ],
      settings: {
        vize: {
          preset: "incremental",
          rules: { "vue/no-v-html": ["error", { option: true }] },
        },
      },
      overrides: [
        { files: ["src/*.vue"], rules: { "vize/vue/no-v-html": "off", "no-debugger": "warn" } },
      ],
    },
  };
  const originalBytes = JSON.stringify(config);
  assert.equal(needsOriginalProjectChecks(config, []), true);
  const phases = splitProjectConfig(config);
  assert.deepEqual(phases.original.value, {
    ...config.value,
    rules: {},
    extends: [
      {
        ...config.value.extends[0],
        rules: { "import/no-self-import": "warn", "vize/vue/no-v-html": "off" },
      },
    ],
    overrides: config.value.overrides,
  });
  assert.deepEqual(phases.bridge.value, {
    settings: config.value.settings,
    rules: {},
    plugins: [],
    categories: { correctness: "off" },
    extends: [
      {
        plugins: [],
        options: { typeAware: false, typeCheck: false },
        rules: { "vize/vue/no-v-html": ["error", { option: true }] },
        categories: { correctness: "off" },
      },
    ],
    overrides: [{ files: ["src/*.vue"], rules: { "vize/vue/no-v-html": "off" }, plugins: [] }],
  });
  assert.equal(JSON.stringify(config), originalBytes);
  assert.deepEqual(
    bridgeProjectArgs([
      "--type-aware",
      "--import-plugin",
      "--tsconfig",
      "project.json",
      "--format=json",
      "--",
    ]),
    ["--format=json", "--"],
  );
  assert.deepEqual(replaceConfigArgs(["--format=json", "--"], "/sibling.mts"), [
    "--config",
    "/sibling.mts",
    "--format=json",
    "--",
  ]);
  for (const flag of [
    "--deny",
    "--warn=all",
    "--max-warnings=1",
    "--type-check-only",
    "--report-unused-disable-directives",
    "--report-unused-disable-directives-severity=error",
    "--fix",
    "--fix-suggestions",
    "--fix-dangerously",
  ])
    assert.throws(() => bridgeProjectArgs([flag]), /CLI severity\/limit overrides/u);
});

void test("complete JSON union preserves foreign fields, core diagnostics and original file count", () => {
  const core = {
    filename: "src/Panel.vue",
    code: "no-debugger",
    labels: [{ span: { offset: 30, length: 9, line: 3, column: 1 } }],
    message: "complete original",
  };
  const template = {
    filename: "src/Panel.vue",
    code: "vize(vue/no-v-html)",
    labels: [{ span: { offset: 90, length: 13, line: 7, column: 8 } }],
    message: "complete template",
  };
  const original = {
    status: 1,
    stderr: "source stderr\n",
    stdout: JSON.stringify({
      diagnostics: [core],
      number_of_files: 12,
      number_of_rules: 25,
      threads_count: 4,
      start_time: 123,
      number_of_warnings: 2,
      number_of_errors: 1,
      opaque: { complete: true },
    }),
  };
  const bridge = {
    status: 1,
    stderr: "bridge stderr\n",
    stdout: JSON.stringify({
      diagnostics: [template],
      number_of_files: 3,
      number_of_rules: 51,
      threads_count: 8,
      start_time: 12,
      number_of_warnings: 3,
      number_of_errors: 4,
    }),
  };
  const merged = mergeProjectOutput(original, bridge, ["-f", "json"]);
  assert.equal(merged.status, 1);
  assert.equal(merged.stderr, "source stderr\nbridge stderr\n");
  assert.deepEqual(JSON.parse(merged.stdout), {
    diagnostics: [core, template],
    number_of_files: 12,
    number_of_rules: 76,
    threads_count: 8,
    start_time: 135,
    number_of_warnings: 5,
    number_of_errors: 5,
    opaque: { complete: true },
  });
  assert.equal(
    mergeProjectOutput(
      { ...original, stdout: "core whole\n" },
      { ...bridge, stdout: "template whole\n" },
      [],
    ).stdout,
    "core whole\ntemplate whole\n",
  );
  assert.throws(() => projectOutputFormat(["-f", "junit"]), /support default/u);
  assert.throws(
    () => mergeProjectOutput(original, { ...bridge, stdout: "{}" }, ["-f", "json"]),
    /complete Oxlint JSON/u,
  );
  assert.equal(
    mergeProjectOutput({ ...original, status: 0 }, bridge, ["-f", "json"]).status,
    1,
    "a Vize-only error must fail the complete run",
  );
  assert.deepEqual(
    mergeProjectOutput(
      { status: null, stdout: "", stderr: "native IPC failed\n" },
      { ...bridge, stderr: "" },
      ["-f", "json"],
    ),
    { status: 1, stdout: "", stderr: "native IPC failed\n" },
  );
  assert.throws(
    () => mergeProjectOutput({ ...original, status: 0, stdout: "" }, bridge, ["-f", "json"]),
    /empty successful/u,
  );
  assert.deepEqual(
    mergeProjectOutput(
      { status: 2, stdout: "native config failure\n", stderr: "source failure\n" },
      { ...bridge, stderr: "" },
      ["-f", "json"],
    ),
    { status: 2, stdout: "native config failure\n", stderr: "source failure\n" },
  );
});

void test("an unavailable bridge retains every original JSON field and fatal row", () => {
  const fixture = JSON.parse(
    readFileSync(
      new URL(
        "../../../../tests/_fixtures/differential/lint/oxlint-script-safe-carrier-7903/project-failure.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  const original = {
    stdout: JSON.stringify(fixture.original),
    stderr: "original stderr\n",
    status: 1,
  };
  for (const status of [1, 2, null]) {
    for (const stdout of ["", "unavailable bridge packet\n"]) {
      const bridge = { stdout, stderr: "bridge stderr\n", status };
      const merged = mergeProjectOutput(original, bridge, ["-f", "json"]);
      assert.equal(merged.stdout, original.stdout);
      assert.equal(merged.stderr, original.stderr + bridge.stderr + stdout);
      assert.equal(merged.status, Math.max(original.status, status ?? 1));
      assert.deepEqual(JSON.parse(merged.stdout), fixture.original);
    }
  }
});
