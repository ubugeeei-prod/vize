import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { planPrChecks, prCheckOutputs } from "../../tools/support/compat/github/plan-pr-checks.ts";
import {
  planAffectedJs,
  readJsWorkspace,
} from "../../tools/support/compat/github/plan-affected-js.ts";
import { planAffectedRust } from "../../tools/support/compat/github/plan-affected-rust.ts";
import { planSourceChecks } from "../../tools/support/compat/github/plan-source-checks.ts";
import { planNativeRuntime } from "../../tools/support/compat/github/plan-native-runtime.ts";
import { selectPrTier } from "../../tools/support/compat/github/select-pr-tier.ts";
import {
  differentialMatrix,
  differentialSteps,
  rustDifferentialLanes,
} from "../../tools/support/compat/github/rust-differential-lanes.ts";
import { requireRustTier } from "../../tools/support/compat/github/require-rust-tier.ts";
import { aggregateCheckNeedsResults } from "../../tools/support/compat/github/require-needs-success.ts";
import { rustCommand } from "../../tools/support/compat/github/run-affected-rust.ts";

const metadata = {
  workspace_root: "/repo",
  workspace_members: ["vize_armature", "vize_vitrine", "vize_curator", "vize_atelier_ssr"],
  packages: [
    {
      id: "vize_armature",
      name: "vize_armature",
      manifest_path: "/repo/crates/vize_armature/Cargo.toml",
      dependencies: [],
    },
    {
      id: "vize_vitrine",
      name: "vize_vitrine",
      manifest_path: "/repo/crates/vize_vitrine/Cargo.toml",
      dependencies: [{ path: "/repo/crates/vize_armature" }],
    },
    {
      id: "vize_curator",
      name: "vize_curator",
      manifest_path: "/repo/crates/vize_curator/Cargo.toml",
      dependencies: [],
    },
    {
      id: "vize_atelier_ssr",
      name: "vize_atelier_ssr",
      manifest_path: "/repo/crates/vize_atelier_ssr/Cargo.toml",
      dependencies: [{ path: "/repo/crates/vize_armature", kind: "dev" }],
    },
  ],
};
const workspace = [
  { name: "@vizejs/native", path: "npm/native", dependencies: [] },
  { name: "@vizejs/fresco-native", path: "npm/fresco-native", dependencies: [] },
  { name: "@vizejs/cli", path: "npm/cli", dependencies: ["@vizejs/native"] },
  { name: "@vizejs/ui", path: "npm/ui", dependencies: ["@vizejs/native"] },
  { name: "@vizejs/oxlint", path: "npm/oxlint", dependencies: ["@vizejs/native"] },
  { name: "@vizejs/fresco", path: "npm/fresco", dependencies: ["@vizejs/fresco-native"] },
];

test("the pure source selector retains the actual reverse dependency closure in either context", () => {
  for (const event of ["pull_request", "merge_group"]) {
    const plan = planAffectedRust(
      metadata,
      ["crates/vize_armature/src/lib.rs", "npm/cli/src/index.ts"],
      event,
    );
    assert.deepEqual(plan.packages, ["vize_armature", "vize_atelier_ssr", "vize_vitrine"]);
    assert.deepEqual(plan.differentialMatrix, { lane: ["ssr", "wasm"] });
    assert.deepEqual(rustCommand(plan, ["cargo", "test", "@packages@", "--doc"]), [
      "cargo",
      "test",
      "--package",
      "vize_armature",
      "--package",
      "vize_atelier_ssr",
      "--package",
      "vize_vitrine",
      "--doc",
    ]);
  }
});

test("shared, deleted and unknown inputs still select every Rust package", () => {
  for (const path of [
    "Cargo.lock",
    "crates/deleted/src/lib.rs",
    "tests/_fixtures/source.vue",
    ".cargo/config.toml",
    "new-runtime/lib.rs",
  ]) {
    assert.equal(planAffectedRust(metadata, [path], "merge_group").scope, "workspace", path);
  }
  assert.equal(planAffectedRust(metadata, [], "merge_group").scope, "workspace");
  const fixture = "tests/_fixtures/differential/lint/prop-declarations-448/cases.json";
  assert.equal(planSourceChecks([fixture], "merge_group").rust, true);
  assert.deepEqual(
    planAffectedJs(
      [...workspace, { name: "@vize/tests", path: "tests", dependencies: [] }],
      [fixture],
    ).groups,
    ["builders", "ui", "oxlint"],
  );
  for (const tool of [
    "tools/support/github/build-vitrine-wasm.ts",
    "tools/support/tasks/build-vitrine-wasm.ts",
    "tools/support/command-runtime.ts",
    "tools/commands/davinci/incremental-equivalence/scripts/input.ts",
    "tools/support/release/public_acceptance/gallery.ts",
  ]) {
    assert.equal(planSourceChecks([tool], "merge_group").playground, true, tool);
    assert.equal(planAffectedRust(metadata, [tool]).scope, "workspace", tool);
    assert.equal(planAffectedJs(workspace, [tool]).scope, "workspace", tool);
  }
  for (const input of ["npm/cli/pkl/Vize.pkl", "examples/html-conformance/src/App.vue"]) {
    assert.equal(planSourceChecks([input], "merge_group").rust, true, input);
    assert.equal(
      planAffectedRust(metadata, ["crates/vize_curator/src/lib.rs", input]).scope,
      "workspace",
      input,
    );
  }
});

test("Rust source affects JS only through a source-built native package", () => {
  const leaf = planAffectedRust(metadata, ["crates/vize_curator/src/lib.rs"]);
  assert.deepEqual(planAffectedJs(workspace, ["crates/vize_curator/src/lib.rs"], leaf).groups, []);
  const compiler = planAffectedRust(metadata, ["crates/vize_armature/src/lib.rs"]);
  const selected = planAffectedJs(workspace, ["crates/vize_armature/src/lib.rs"], compiler);
  assert.deepEqual(selected.groups, ["builders", "ui", "oxlint"]);
  assert.equal(selected.conformance.fresco, false);
  assert.equal(selected.conformance.ui, true);
});

test("JS workspace edges include transitive consumers and exclude unrelated groups", () => {
  const plan = planAffectedJs(workspace, ["npm/fresco-native/index.d.ts"]);
  assert.deepEqual(plan.packages, ["npm/fresco", "npm/fresco-native"]);
  assert.deepEqual(plan.groups, ["ui"]);
  assert.deepEqual(plan.conformance, { fresco: true, ui: false });
  assert.deepEqual(planAffectedJs(workspace, ["npm/oxlint/src/index.ts"]).groups, ["oxlint"]);
  assert.equal(planAffectedJs(workspace, ["npm/deleted/index.ts"]).scope, "workspace");
  assert.equal(planAffectedJs(workspace, ["pnpm-lock.yaml"]).scope, "workspace");
});

test("actual workspace manifests retain native consumers without duplicate fixture packages", () => {
  const packages = readJsWorkspace();
  assert.equal(new Set(packages.map((pkg) => pkg.name)).size, packages.length);
  const plan = planAffectedJs(packages, ["npm/native/index.js"]);
  assert.ok(plan.packages.includes("npm/builder/vite"));
  assert.ok(plan.packages.includes("npm/framework/nuxt"));
  assert.ok(plan.packages.includes("npm/oxlint"));
});

test("prose and tooling-only paths do not create affected product dependencies", () => {
  for (const event of ["pull_request", "merge_group"]) {
    assert.deepEqual(planSourceChecks(["docs/content/guide.md"], event), {
      rust: false,
      js: false,
      tooling: false,
      playground: false,
    });
    const tooling = planSourceChecks(["tools/support/release/preflight.ts"], event);
    assert.equal(tooling.rust, false);
    assert.equal(tooling.js, false);
  }
  assert.deepEqual(planAffectedJs(workspace, ["docs/content/guide.md"]).matrix, {
    group: ["none"],
  });
  assert.deepEqual(planNativeRuntime(["docs/content/guide.md"]), {
    run: false,
    matrix: { lane: ["none"] },
  });
});

test("affected native captures keep source-built runtime proofs independent of tooling suites", () => {
  assert.ok(planNativeRuntime(["davinci/vize_l4/src/targets/ssr.rs"]).matrix.lane.includes("ssr"));
  assert.ok(
    planNativeRuntime(["davinci/vize_l4/src/targets/vapor.rs"]).matrix.lane.includes("vapor"),
  );
  assert.equal(planNativeRuntime(["Cargo.lock"]).matrix.lane.length, 8);
  assert.equal(
    planNativeRuntime(["davinci/vize_l0_derive/src/lib.rs"], { packages: ["vize_l4"] }).matrix.lane
      .length,
    8,
  );
  assert.ok(
    planNativeRuntime(["davinci/vize_l4/tests/fixtures/selected-template-dom-vue-3.5.35.json"], {
      packages: ["vize_l4"],
    }).matrix.lane.includes("selected-template"),
  );
});

test("native typechecker and actual LSP checks follow their source and oracle inputs", () => {
  for (const path of [
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source.rs",
    ".github/actions/test-native-canon/action.yml",
  ])
    assert.ok(planNativeRuntime([path]).matrix.lane.includes("canon"), path);
  assert.ok(
    planNativeRuntime(["crates/vize_croquis/src/lib.rs"], {
      packages: ["vize_canon"],
    }).matrix.lane.includes("canon"),
  );
  assert.ok(
    planNativeRuntime(["crates/vize_canon/src/lib.rs"], {
      packages: ["vize_maestro", "vize"],
    }).matrix.lane.includes("lsp"),
  );
  for (const path of [
    "crates/vize_maestro/src/lib.rs",
    "tests/tooling/lsp-editor-jsconfig.test.ts",
    "tests/tooling/support/package-private-imports.ts",
    "tests/tooling/support/typecheck-dependency.ts",
    "tests/tooling/support/lsp/launch.ts",
    "tests/differential/build-receipt.ts",
    "tests/_fixtures/differential/lsp/editor-jsconfig/input.txt",
    ".github/actions/test-native-lsp/action.yml",
  ])
    assert.ok(planNativeRuntime([path]).matrix.lane.includes("lsp"), path);
  assert.deepEqual(planNativeRuntime(["tests/tooling/check-docs-format.test.ts"]).matrix, {
    lane: ["none"],
  });
});

test("normal PRs defer duplicate heavy qualification and preserve an explicit full-ci escape hatch", () => {
  const paths = ["crates/vize_atelier_sfc/src/lib.rs"];
  assert.equal(selectPrTier({ eventName: "pull_request", paths, labels: [] }), "fast");
  assert.equal(selectPrTier({ eventName: "pull_request", paths, labels: ["full-ci"] }), "heavy");
  assert.equal(selectPrTier({ eventName: "merge_group", paths, labels: [] }), "heavy");
});

test("one source plan joins actual Rust and JS closures without temporary JSON files", () => {
  const plan = planPrChecks({
    paths: ["crates/vize_armature/src/lib.rs"],
    eventName: "pull_request",
    labels: [],
    workspace,
    cargoMetadata: metadata,
  });
  const outputs = prCheckOutputs(plan);
  assert.equal(outputs.rust, "true");
  assert.deepEqual(JSON.parse(outputs["rust-plan"]).packages, [
    "vize_armature",
    "vize_atelier_ssr",
    "vize_vitrine",
  ]);
  assert.deepEqual(JSON.parse(outputs["js-plan"]).groups, ["builders", "ui", "oxlint"]);
  assert.equal(outputs.heavy, "false");
  assert.equal(
    prCheckOutputs(
      planPrChecks({
        paths: ["docs/guide.md"],
        eventName: "merge_group",
        labels: [],
        workspace,
        cargoMetadata: metadata,
      }),
    ).rust,
    "true",
  );
  assert.throws(
    () => planPrChecks({ paths: ["Cargo.lock"], eventName: "pull_request", labels: [], workspace }),
    /require Cargo workspace metadata/,
  );
});

test("product differential partitions preserve each selected command exactly once", () => {
  const all = [
    ...new Set(rustDifferentialLanes.flatMap((lane) => lane.steps.map((step) => step.package))),
  ];
  assert.ok(differentialMatrix(all).lane.includes("navigation"));
  assert.deepEqual(differentialMatrix(["vize_atelier_ssr"]), { lane: ["ssr"] });
  const commands = differentialSteps("ssr", ["vize_atelier_ssr"]);
  assert.equal(commands.length, 3);
  assert.ok(commands[0].args.includes("davinci_ssr_corpus"));
  assert.ok(commands[1].args.includes("davinci_ssr_production_cases"));
  assert.equal(commands[2].env?.VIZE_VUE_RUNTIME_PRODUCTION, "1");
  assert.throws(() => differentialSteps("unknown", all), /unknown/);
});

test("differential commands retain their original corpus and production environments", () => {
  const cwd = realpathSync(mkdtempSync(join(tmpdir(), "vize-differential-env-")));
  const entry = fileURLToPath(
    new URL("../../tools/support/compat/github/rust-differential-lanes.ts", import.meta.url),
  );
  try {
    const bin = join(cwd, "bin");
    mkdirSync(bin);
    writeFileSync(
      join(bin, "cargo"),
      `#!${process.execPath}\nconst fs = require("node:fs");
const { VIZE_DAVINCI_DIFFERENTIAL_CORPUS: corpus, VIZE_VUE_RUNTIME_PRODUCTION: production, VIZE_TEST_REQUIRE_TSGO: tsgo } = process.env;
fs.appendFileSync(process.env.COMMAND_LOG, JSON.stringify({ args: process.argv.slice(2), corpus, production, tsgo }) + "\\n");
if (process.argv.includes(process.env.FAIL_TARGET)) process.exit(42);
`,
      { mode: 0o755 },
    );
    const packages = rustDifferentialLanes.flatMap((lane) =>
      lane.steps.map((step) => step.package),
    );
    const plan = join(cwd, "plan.json");
    const log = join(cwd, "commands.jsonl");
    writeFileSync(plan, JSON.stringify({ packages }));
    const env: NodeJS.ProcessEnv = {
      ...process.env,
      PATH: `${bin}:${process.env.PATH}`,
      COMMAND_LOG: log,
      VIZE_TEST_REQUIRE_TSGO: "1",
    };
    delete env.VIZE_DAVINCI_DIFFERENTIAL_CORPUS;
    delete env.VIZE_VUE_RUNTIME_PRODUCTION;
    const expectedCalls: unknown[] = [];
    for (const lane of rustDifferentialLanes.filter((lane) => !lane.navigation)) {
      writeFileSync(log, "");
      const result = spawnSync(process.execPath, [entry, lane.name, plan], {
        cwd,
        env,
        encoding: "utf8",
      });
      assert.equal(result.status, 0, result.stderr);
      const calls = readFileSync(log, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line));
      expectedCalls.push(...calls);
      assert.equal(calls.length, lane.steps.length, lane.name);
      for (const call of calls) {
        const has = (target: string) => call.args.includes(target);
        const workspace = has("davinci_ssr_corpus") || has("davinci_markup_differential");
        assert.equal(call.corpus, workspace ? cwd : undefined, call.args.join(" "));
        const production = [
          "davinci_vapor_upstream_parity",
          "vapor_runtime_contracts",
          "vue_ssr_render",
        ].some(has);
        assert.equal(call.production, production ? "1" : undefined, call.args.join(" "));
        assert.equal(call.tsgo, "1");
      }
    }
    writeFileSync(log, "");
    const all = spawnSync(process.execPath, [entry, "--all"], { cwd, env, encoding: "utf8" });
    assert.equal(all.status, 0, all.stderr);
    assert.deepEqual(
      readFileSync(log, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line)),
      expectedCalls,
      "scheduled and affected recipes execute identical arguments and environment",
    );
    writeFileSync(log, "");
    const failure = spawnSync(process.execPath, [entry, "compiler", plan], {
      cwd,
      env: { ...env, FAIL_TARGET: "vize_atelier_jsx" },
      encoding: "utf8",
    });
    assert.equal(failure.status, 42);
    assert.equal(readFileSync(log, "utf8").trim().split("\n").length, 1);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

test("Rust report rejects skipped checks and missing or failed required work", () => {
  assert.throws(
    () => requireRustTier("merge_group", "false", {}),
    /Rust source checks must run for every pull request and merge group/,
  );
  const results = {
    "merge-rust-source": { result: "success" },
    "merge-rust-differential": { result: "success" },
    "pr-rust-build": { result: "success" },
    "pr-rust-shard": { result: "success" },
  };
  assert.equal(requireRustTier("merge_group", "true", results).exitCode, 0);
  assert.equal(
    requireRustTier("merge_group", "true", { ...results, "pr-rust-shard": { result: "skipped" } })
      .exitCode,
    1,
  );
  assert.throws(() => requireRustTier("merge_group", "true", {}), /Missing Rust tier job/);
});

test("queue product qualifications require actual success and allow their explicit PR skip", () => {
  const jobs = ["nuxt-module-qualification"];
  const success = () => Object.fromEntries(jobs.map((job) => [job, { result: "success" }]));
  assert.equal(aggregateCheckNeedsResults(success(), "merge_group").exitCode, 0);
  for (const job of jobs) {
    for (const result of ["failure", "cancelled", "skipped", "unknown"]) {
      const needs = { ...success(), [job]: { result } };
      assert.equal(aggregateCheckNeedsResults(needs, "merge_group").exitCode, 1);
      assert.equal(
        aggregateCheckNeedsResults(needs, "pull_request").exitCode,
        result === "skipped" ? 0 : 1,
      );
    }
    const missing = success();
    delete missing[job];
    for (const event of ["pull_request", "merge_group"]) {
      assert.throws(() => aggregateCheckNeedsResults(missing, event), /missing/);
      assert.throws(
        () => aggregateCheckNeedsResults({ ...success(), [job]: {} }, event),
        /reported no result/,
      );
    }
  }
});
