import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { parse } from "yaml";
import { parse as parseToml } from "@iarna/toml";
import { test } from "node:test";
import { nativeVaporCaptureRequired } from "../../tools/support/compat/github/native-vapor-capture.mjs";
import { planNativeRuntime } from "../../tools/support/compat/github/plan-native-runtime.ts";
import { assertNativeRuntimeHook } from "./support/native-runtime-workflow.ts";

await test("original native Vapor source qualifies a fresh affected-PR capture without prose admissions", () => {
  for (const path of [
    "davinci/vize_l1/src/container/vue/descriptor/policy.rs",
    "davinci/vize_l2/src/lang/js/file/native.rs",
    "davinci/vize_l1_to_l2/src/native_file/selected.rs",
    "davinci/vize_l3/src/decision/vapor/build.rs",
    "davinci/vize_l4/src/targets/vapor/component.rs",
    "davinci/vize_l4/src/module.rs",
    "davinci/vize_l4/src/expr.rs",
    "davinci/vize_l4/src/expr/resolved.rs",
    "davinci/vize_l4/src/expr/vue.rs",
    "davinci/vize_l4/src/write/source_map.rs",
    "davinci/vize_l4/tests/native_vapor.rs",
    "crates/vize_atelier_jsx/src/vapor.rs",
    "crates/vize_atelier_jsx/tests/v_slots_forwarding.rs",
    "crates/vize_atelier_vapor/src/generate/operations/component.rs",
    "crates/vize_atelier_sfc/src/lib.rs",
    "crates/vize_atelier_sfc/src/native_vapor.rs",
    "crates/vize_atelier_sfc/src/native_vapor_setup.rs",
    "crates/vize_atelier_sfc/tests/native_vapor_sfc.rs",
    "crates/vize_atelier_sfc/tests/native_vapor_setup_sfc.rs",
    "crates/vize_atelier_sfc/tests/vapor_setup_refusals/mod.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_vapor_sfc_vue_3_6_rc9.json",
    "tests/tooling/native-vapor-sfc-reference.test.mjs",
    "tests/tooling/native-vapor-setup-sfc-reference.test.mjs",
    "tests/tooling/jsx-runtime-slots-vapor.test.mjs",
    "tests/tooling/support/davinci-runtime-trace.mjs",
    "tests/tooling/support/native-vapor-sfc-oracle.mjs",
    "tests/tooling/support/native-vapor-primary-lifecycle.mjs",
    "tests/tooling/support/native-vapor-process-capture.mjs",
    "tests/tooling/support/vue-vapor-release.mjs",
    ".github/actions/test-native-vapor/action.yml",
    ".github/workflows/check.yml",
    ".github/workflows/pr-source-checks.yml",
  ]) {
    assert.equal(nativeVaporCaptureRequired([path]), true, path);
    assert(planNativeRuntime([path]).matrix.lane.includes("vapor"), path);
  }
  for (const path of [
    "README.md",
    "docs/davinci/decisions/2026-09-27-level-restructure.md",
    "crates/vize_atelier_sfc/src/native_ssr.rs",
    "crates/vize_atelier_sfc/tests/vapor_setup_refusals_unrelated/mod.rs",
    "npm/ui/src/main.ts",
    "davinci/vize_l4/src/targets/ts.rs",
    "davinci/vize_l4/src/expr.rs.unrelated",
    "davinci/vize_l4/src/expr_unrelated/resolved.rs",
  ])
    assert.equal(nativeVaporCaptureRequired([path]), false, path);
  assert.equal(nativeVaporCaptureRequired([]), false);
});

await test("the required Vapor lane preserves mandatory whole-source payloads", () => {
  const read = (path) => readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");
  assertNativeRuntimeHook("vapor", "test-native-vapor");
  const action = read(".github/actions/test-native-vapor/action.yml");
  for (const path of [
    "native-vapor-modules.json",
    "native-vapor-runtime.json",
    "native-vapor-sfc-modules.json",
    "native-vapor-sfc-runtime.json",
    "native-vapor-primary-lifecycle.json",
    "native-vapor-processes.jsonl",
    "native-vapor-setup-sfc-modules.json",
    "native-vapor-setup-sfc-runtime.json",
    "jsx-runtime-slots-vapor.json",
    "jsx-runtime-slots-vapor.json.runtime.json",
  ])
    assert.ok(action.includes(path), path);
  assert.match(action, /cargo test --locked --profile ci -p vize_l4 --test native_vapor/);
  assert.match(
    action,
    /cargo test --locked --profile ci -p vize_atelier_sfc --test native_vapor_sfc/,
  );
  assert.match(
    action,
    /cargo test --locked --profile ci -p vize_atelier_jsx --test v_slots_forwarding/,
  );
  assert.ok(action.includes("tests/tooling/jsx-runtime-slots-vapor.test.mjs"));
  assert.match(action, /VIZE_NATIVE_VAPOR_REQUIRE_CAPTURE: "1"/);
  assert.match(action, /if-no-files-found: error/);
  assert.match(action, /if: \$\{\{ always\(\) \}\}/);
});

function assertRuntimeToolchainPreparation(workflow, configuration) {
  const runtime = workflow.jobs["pr-native-runtime"];
  const configured = configuration.toolchain;
  assert.equal(runtime["timeout-minutes"], 15);
  assert.equal(runtime["continue-on-error"], undefined);
  assert.equal(runtime.env.VIZE_NATIVE_RUST_TOOLCHAIN, configured.channel);
  assert.deepEqual(runtime.env.VIZE_NATIVE_RUST_COMPONENTS.split(","), configured.components);
  assert.deepEqual(runtime.env.VIZE_NATIVE_RUST_TARGETS.split(","), configured.targets);
  const setups = runtime.steps.filter((step) => step.uses?.startsWith("dtolnay/rust-toolchain@"));
  assert.equal(setups.length, 1);
  const setup = setups[0];
  assert.equal(setup.uses, "dtolnay/rust-toolchain@d1031067263f94b142dd6c0ce24c5eb9d02d52a0");
  assert.equal(setup.if, "${{ matrix.lane != 'none' }}");
  assert.equal(setup["continue-on-error"], undefined);
  assert.deepEqual(setup.with, {
    toolchain: "${{ env.VIZE_NATIVE_RUST_TOOLCHAIN }}",
    components: "${{ env.VIZE_NATIVE_RUST_COMPONENTS }}",
    targets: "${{ env.VIZE_NATIVE_RUST_TARGETS }}",
  });
  const verifications = runtime.steps.filter(
    (step) => step.name === "Verify configured native runtime toolchain",
  );
  assert.equal(verifications.length, 1);
  const verification = verifications[0];
  assert.equal(verification.if, setup.if);
  assert.equal(verification["continue-on-error"], undefined);
  assert.equal(verification.shell, "bash");
  assert.equal(runtime.steps.indexOf(verification), runtime.steps.indexOf(setup) + 1);
  assert(
    runtime.steps.indexOf(verification) <
      runtime.steps.findIndex((step) => step.uses === "./.github/actions/setup-rust-sticky-cache"),
  );
  return { runtime, verification };
}

const nativeRuntimeWorkflow = parse(
  readFileSync(new URL("../../.github/workflows/pr-source-checks.yml", import.meta.url), "utf8"),
);
const nativeRustConfiguration = parseToml(
  readFileSync(new URL("../../rust-toolchain.toml", import.meta.url), "utf8"),
);

void test("native runtime installs the complete repository toolchain before any Cargo test", () => {
  assertRuntimeToolchainPreparation(nativeRuntimeWorkflow, nativeRustConfiguration);
});

void test("native runtime toolchain preparation rejects missing, changed and deferred configuration", () => {
  const mutations = [
    (job) => {
      delete job.steps.find((step) => step.uses?.startsWith("dtolnay/rust-toolchain@")).with
        .components;
    },
    (job) => {
      delete job.steps.find((step) => step.uses?.startsWith("dtolnay/rust-toolchain@")).with
        .targets;
    },
    (job) => {
      job.env.VIZE_NATIVE_RUST_TOOLCHAIN = "stable";
    },
    (job) => {
      job.env.VIZE_NATIVE_RUST_COMPONENTS = "clippy,rustfmt";
    },
    (job) => {
      job.env.VIZE_NATIVE_RUST_TARGETS += ",aarch64-unknown-linux-gnu";
    },
    (job) => {
      job.steps = job.steps.filter(
        (step) => step.name !== "Verify configured native runtime toolchain",
      );
    },
    (job) => {
      const index = job.steps.findIndex(
        (step) => step.name === "Verify configured native runtime toolchain",
      );
      job.steps.push(...job.steps.splice(index, 1));
    },
    (job) => {
      job.steps.find((step) => step.name === "Verify configured native runtime toolchain").if =
        "${{ always() }}";
    },
    (job) => {
      job.steps.find((step) => step.name === "Verify configured native runtime toolchain")[
        "continue-on-error"
      ] = true;
    },
    (job) => {
      job["timeout-minutes"] = 30;
    },
    (job) => {
      job["continue-on-error"] = true;
    },
    (job) => {
      job.steps.push(
        structuredClone(job.steps.find((step) => step.uses?.startsWith("dtolnay/rust-toolchain@"))),
      );
    },
  ];
  for (const mutate of mutations) {
    const changed = structuredClone(nativeRuntimeWorkflow);
    mutate(changed.jobs["pr-native-runtime"]);
    assert.throws(() => assertRuntimeToolchainPreparation(changed, nativeRustConfiguration));
  }
  for (const field of ["channel", "components", "targets"]) {
    const changed = structuredClone(nativeRustConfiguration);
    changed.toolchain[field] =
      field === "channel" ? "1.99.2" : [...changed.toolchain[field], "new-configured-item"];
    assert.throws(() => assertRuntimeToolchainPreparation(nativeRuntimeWorkflow, changed));
  }
});

function verifyInstalledRuntimeToolchain(components, targets, fail = "") {
  const fixture = mkdtempSync(join(tmpdir(), "vize-native-runtime-toolchain-"));
  try {
    const trace = join(fixture, "calls.jsonl");
    writeFileSync(
      join(fixture, "rustup"),
      `#!${process.execPath}\n` +
        String.raw`
const fs = require('node:fs');
const args = process.argv.slice(2);
fs.appendFileSync(process.env.RUSTUP_TEST_TRACE, JSON.stringify(args) + '\n');
if (args[0] === process.env.RUSTUP_TEST_FAIL) process.exit(42);
if (JSON.stringify(args) === JSON.stringify(['run', process.env.VIZE_NATIVE_RUST_TOOLCHAIN, 'rustc', '--version', '--verbose'])) {
  console.log('rustc fixture\nhost: x86_64-unknown-linux-gnu');
} else if (JSON.stringify(args) === JSON.stringify(['component', 'list', '--toolchain', process.env.VIZE_NATIVE_RUST_TOOLCHAIN, '--installed'])) {
  console.log(process.env.RUSTUP_TEST_COMPONENTS);
} else if (JSON.stringify(args) === JSON.stringify(['target', 'list', '--toolchain', process.env.VIZE_NATIVE_RUST_TOOLCHAIN, '--installed'])) {
  console.log(process.env.RUSTUP_TEST_TARGETS);
} else {
  console.error('unexpected Rustup operation', args);
  process.exit(43);
}
`,
      { mode: 0o755 },
    );
    const { runtime, verification } = assertRuntimeToolchainPreparation(
      nativeRuntimeWorkflow,
      nativeRustConfiguration,
    );
    const result = spawnSync("/bin/bash", ["-c", verification.run], {
      encoding: "utf8",
      timeout: 10_000,
      env: {
        ...process.env,
        ...runtime.env,
        PATH: `${fixture}:${process.env.PATH}`,
        RUSTUP_TEST_TRACE: trace,
        RUSTUP_TEST_COMPONENTS: components.join("\n"),
        RUSTUP_TEST_TARGETS: targets.join("\n"),
        RUSTUP_TEST_FAIL: fail,
      },
    });
    return {
      result,
      calls: readFileSync(trace, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line)),
    };
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
}

void test("native runtime verification accepts configured installed identities without installing again", () => {
  const components = [
    "cargo-x86_64-unknown-linux-gnu",
    "clippy-x86_64-unknown-linux-gnu",
    "rust-src",
    "rustfmt-x86_64-unknown-linux-gnu",
  ];
  const targets = ["x86_64-unknown-linux-gnu", ...nativeRustConfiguration.toolchain.targets];
  const { result, calls } = verifyInstalledRuntimeToolchain(components, targets);
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(calls, [
    ["run", nativeRustConfiguration.toolchain.channel, "rustc", "--version", "--verbose"],
    ["component", "list", "--toolchain", nativeRustConfiguration.toolchain.channel, "--installed"],
    ["target", "list", "--toolchain", nativeRustConfiguration.toolchain.channel, "--installed"],
  ]);
  for (const identity of [...components.slice(1), ...nativeRustConfiguration.toolchain.targets]) {
    const missing = verifyInstalledRuntimeToolchain(
      components.filter((item) => item !== identity),
      targets.filter((item) => item !== identity),
    );
    assert.equal(missing.result.status, 1, identity);
    assert.match(missing.result.stderr, /Missing configured Rust (component|target):/);
  }
  const wrongHost = verifyInstalledRuntimeToolchain(
    components.map((item) =>
      item.replaceAll("x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"),
    ),
    targets,
  );
  assert.equal(wrongHost.result.status, 1);
  const partialTarget = verifyInstalledRuntimeToolchain(
    components,
    targets.map((item) => `${item}-unexpected`),
  );
  assert.equal(partialTarget.result.status, 1);
  for (const operation of ["run", "component", "target"]) {
    const failed = verifyInstalledRuntimeToolchain(components, targets, operation);
    assert.equal(failed.result.status, 42, operation);
  }
});
