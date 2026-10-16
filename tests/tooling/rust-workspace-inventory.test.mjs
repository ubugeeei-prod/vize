import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  compareWorkspaceInventories,
  digest,
  inferWorkspaceProfileDefaults,
  normalizeWorkspaceInventory,
  readWorkspaceInventory,
  requireFullWorkspaceJobs,
  verifyWorkspaceExecution,
} from "../../tools/support/compat/github/rust-workspace-inventory.mjs";

const list = {
  "rust-suites": {
    "crate::integration": {
      "binary-id": "crate::integration",
      status: "listed",
      "package-name": "crate",
      "binary-name": "integration",
      testcases: {
        "module::case<&>": { ignored: false, "filter-match": { status: "matches" } },
        "module::ignored": { ignored: true, "filter-match": { status: "matches" } },
      },
    },
    empty: {
      "binary-id": "empty",
      status: "listed",
      "package-name": "fixture",
      "binary-name": "empty",
      testcases: {},
    },
  },
};
const source = {
  sha: "a".repeat(40),
  rustc: "rustc 1.99.1 (fixture)",
  runtime: {
    VIZE_TEST_REQUIRE_TSGO: "1",
    VIZE_TEST_DISABLE_TSGO: null,
    VIZE_NUXT_CONFIG_ITERATIONS: "100",
  },
  inferredProfileDefaults: {
    basis: "committed-manifest-config-and-runtime-environment",
    debugAssertions: true,
    overflowChecks: true,
  },
};
const receipt = (profile) => ({
  source,
  cargoProfile: profile,
  binaries: normalizeWorkspaceInventory(list),
});
const timing = {
  source,
  sourceSha: source.sha,
  cargoProfile: "ci",
  runner: "nextest",
  inventoryOnly: false,
  status: "passed",
  phases: [
    {
      name: "inventory",
      exitCode: 0,
      args: [
        "nextest",
        "list",
        "--locked",
        "--workspace",
        "--cargo-profile",
        "ci",
        "--profile",
        "full",
        "--run-ignored",
        "all",
        "--message-format",
        "json",
      ],
    },
    {
      name: "workspace",
      exitCode: 0,
      args: [
        "nextest",
        "run",
        "--locked",
        "--workspace",
        "--cargo-profile",
        "ci",
        "--profile",
        "full",
      ],
    },
    {
      name: "doctests",
      exitCode: 0,
      args: ["test", "--locked", "--workspace", "--profile", "ci", "--doc"],
    },
  ],
};
const junit =
  '<testsuites><testsuite name="crate::integration"><testcase name="module::case&lt;&amp;&gt;" classname="crate::integration" /></testsuite></testsuites>';

void test("profile defaults are inferred only after manifest, config and environment override checks", () => {
  const manifest = '[profile.ci]\ninherits = "dev"\ndebug = false\n';
  const expected = source.inferredProfileDefaults;
  assert.deepEqual(inferWorkspaceProfileDefaults(manifest, ""), expected);
  for (const extra of [
    "[profile.ci.package.example]\ndebug-assertions = false\n",
    "[profile.dev.build-override]\noverflow-checks = false\n",
    "[profile.test]\ndebug-assertions = false\n",
  ])
    assert.throws(() => inferWorkspaceProfileDefaults(manifest + extra, ""));
  for (const config of [
    "[profile.ci]\ndebug-assertions = false\n",
    "[profile.test.package.example]\noverflow-checks = false\n",
    '[build]\nrustflags = ["-C", "debug-assertions=off"]\n',
    '[target.x86_64-unknown-linux-gnu]\nrustflags = "-Coverflow-checks=no"\n',
  ])
    assert.throws(() => inferWorkspaceProfileDefaults(manifest, config));
  assert.throws(() =>
    inferWorkspaceProfileDefaults(manifest, "", { CARGO_PROFILE_CI_DEBUG_ASSERTIONS: "false" }),
  );
  assert.throws(() =>
    inferWorkspaceProfileDefaults(manifest, "", { RUSTFLAGS: "-Cdebug-assertions=off" }),
  );
});

void test("retained receipts require the complete unmodified raw nextest list", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-workspace-inventory-"));
  try {
    const raw = JSON.stringify(list);
    const value = {
      ...receipt("ci"),
      schema: "vize.rust.workspace-inventory",
      schemaVersion: 1,
      rawFile: "raw.json",
      rawSha256: digest(raw),
    };
    const file = join(directory, "inventory.json");
    writeFileSync(join(directory, "raw.json"), raw);
    writeFileSync(file, JSON.stringify(value));
    assert.deepEqual(readWorkspaceInventory(file).binaries, value.binaries);
    for (const mutate of [
      (changed) => {
        changed.rawSha256 = "0".repeat(64);
      },
      (changed) => {
        changed.binaries.pop();
      },
      (changed) => {
        changed.rawFile = "../raw.json";
      },
    ]) {
      const changed = structuredClone(value);
      mutate(changed);
      writeFileSync(file, JSON.stringify(changed));
      assert.throws(() => readWorkspaceInventory(file));
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

void test("the full report requires both actual jobs and rejects missing, failed, skipped or canceled tiers", () => {
  const needs = {
    "full-rust-workspace": { result: "success" },
    "clippy-and-test": { result: "success" },
  };
  requireFullWorkspaceJobs(needs);
  for (const name of Object.keys(needs)) {
    const missing = structuredClone(needs);
    delete missing[name];
    assert.throws(() => requireFullWorkspaceJobs(missing));
    for (const result of ["skipped", "failure", "cancelled", "", undefined]) {
      const changed = structuredClone(needs);
      changed[name].result = result;
      assert.throws(() => requireFullWorkspaceJobs(changed));
    }
  }
});

void test("same-source test/ci inventories preserve empty binaries, ignored flags and full case identities", () => {
  compareWorkspaceInventories(receipt("test"), receipt("ci"));
  assert.equal(receipt("ci").binaries.length, 2);
  verifyWorkspaceExecution(receipt("ci"), timing, junit, source.sha);
  verifyWorkspaceExecution(
    receipt("ci"),
    timing,
    junit.replace(
      "</testsuite>",
      '<system-out><![CDATA[<testcase name="source" classname="source" /><failure />]]></system-out><!-- <testcase /> --></testsuite>',
    ),
    source.sha,
  );
});

void test("profile parity rejects source, compiler, runtime, binary, case and ignored-state drift", () => {
  for (const mutate of [
    (value) => {
      value.source.sha = "b".repeat(40);
    },
    (value) => {
      value.source.rustc = "rustc 1.99.0 (other)";
    },
    (value) => {
      value.source.runtime.VIZE_TEST_REQUIRE_TSGO = "0";
    },
    (value) => {
      value.binaries.pop();
    },
    (value) => {
      value.binaries[0].cases[0].name = "other";
    },
    (value) => {
      value.binaries[0].cases[1].ignored = false;
    },
  ]) {
    const after = structuredClone(receipt("ci"));
    mutate(after);
    assert.throws(() => compareWorkspaceInventories(receipt("test"), after));
  }
  const filtered = structuredClone(list);
  filtered["rust-suites"]["crate::integration"].testcases["module::case<&>"][
    "filter-match"
  ].status = "mismatch";
  assert.throws(() => normalizeWorkspaceInventory(filtered));
  assert.throws(() => normalizeWorkspaceInventory({ "rust-suites": {} }));
  const skippedBinary = structuredClone(list);
  skippedBinary["rust-suites"].empty.status = "skipped";
  assert.throws(() => normalizeWorkspaceInventory(skippedBinary));
});

void test("full acceptance rejects inventory-only, foreign head, missing doctests and incomplete execution", () => {
  for (const mutate of [
    (value) => {
      value.inventoryOnly = true;
    },
    (value) => {
      value.sourceSha = "b".repeat(40);
    },
    (value) => {
      value.source.runtime.VIZE_TEST_REQUIRE_TSGO = "0";
    },
    (value) => {
      value.phases.pop();
    },
    (value) => {
      value.phases[2].exitCode = 1;
    },
    (value) => {
      value.phases[2].args = ["test", "--doc"];
    },
  ]) {
    const record = structuredClone(timing);
    mutate(record);
    assert.throws(() => verifyWorkspaceExecution(receipt("ci"), record, junit, source.sha));
  }
  for (const xml of [
    "<testsuites />",
    junit.replace("module::case", "other"),
    junit.replace("<testcase", "<failure /><testcase"),
  ])
    assert.throws(() => verifyWorkspaceExecution(receipt("ci"), timing, xml, source.sha));
});

void test("an unchanged required case identity cannot be reported as skipped execution", () => {
  verifyWorkspaceExecution(receipt("ci"), timing, junit, source.sha);
  const skipped = junit.replace(" />", "><skipped /></testcase>");
  assert.notEqual(skipped, junit);
  assert.throws(
    () => verifyWorkspaceExecution(receipt("ci"), timing, skipped, source.sha),
    /skipped a required case/,
  );
});
