import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { mergeOnlyToolingTests } from "../../tools/config/vite-plus/tooling-test-scopes.ts";
import {
  planToolingTests,
  toolingTestFiles,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
import { toolingTestCommand } from "../../tools/support/compat/github/run-tooling-tests.mjs";

function fixture() {
  const cwd = mkdtempSync(join(tmpdir(), "vize-tooling-inputs-"));
  for (const directory of [
    "tests/tooling/support",
    "tests/tooling/davinci",
    "tests/tooling/release",
    "tools",
    "docs",
  ]) {
    mkdirSync(join(cwd, directory), { recursive: true });
  }
  const write = (file: string, source = "") => writeFileSync(join(cwd, file), source);
  write("tests/tooling/davinci-phase4-contract-status.test.ts", 'import "./support/plan.ts";');
  write("tests/tooling/support/plan.ts", 'export { value } from "../../../tools/plan-input.mjs";');
  write("tools/plan-input.mjs", "export const value = 1;");
  write("tests/tooling/native.test.ts");
  write("tests/tooling/lsp-smoke.test.ts");
  return { cwd, write, cleanup: () => rmSync(cwd, { recursive: true, force: true }) };
}

void test("a source edit selects only the tests that import or name it", () => {
  const f = fixture();
  try {
    assert.deepEqual(planToolingTests(["davinci/vize_l1/src/lib.rs"], { cwd: f.cwd }).tests, []);
    f.write("tests/tooling/native.test.ts", 'const crate = ["davinci", "vize_l1"];');
    assert.deepEqual(planToolingTests(["davinci/vize_l1/src/lib.rs"], { cwd: f.cwd }).tests, [
      "tests/tooling/native.test.ts",
    ]);
    const docs = planToolingTests(["docs/davinci/plan/phase-4.md"], { cwd: f.cwd });
    assert.ok(docs.tests.includes("tests/tooling/davinci-phase4-contract-status.test.ts"));
  } finally {
    f.cleanup();
  }
});

void test("transitive imports and deleted inputs select the contract", () => {
  const f = fixture();
  try {
    for (const path of ["tools/plan-input.mjs", "tests/tooling/support/plan.ts"]) {
      assert.ok(
        planToolingTests([path], { cwd: f.cwd }).tests.includes(
          "tests/tooling/davinci-phase4-contract-status.test.ts",
        ),
      );
    }
    rmSync(join(f.cwd, "tools/plan-input.mjs"));
    assert.ok(
      planToolingTests(["tools/plan-input.mjs"], { cwd: f.cwd }).tests.includes(
        "tests/tooling/davinci-phase4-contract-status.test.ts",
      ),
    );
  } finally {
    f.cleanup();
  }
});

void test("unknown and shared dependency inputs restore the broad PR suite", () => {
  const f = fixture();
  try {
    for (const paths of [
      [],
      ["new-root/input.ts"],
      ["pnpm-lock.yaml"],
      ["tools/config/vite-plus/task-inputs.ts"],
    ]) {
      assert.equal(planToolingTests(paths, { cwd: f.cwd }).tests.length, 2);
    }
    // This audited plan contract has a curated scope, so an incomplete
    // closure keeps the broad inputs instead of trusting named paths.
    f.write("tests/tooling/davinci-phase4-contract-status.test.ts", "import(variable);");
    assert.deepEqual(planToolingTests(["davinci/vize_l1/src/lib.rs"], { cwd: f.cwd }).tests, [
      "tests/tooling/davinci-phase4-contract-status.test.ts",
    ]);
    // An unscoped test with a computed import runs on the paths it names.
    f.write("tests/tooling/native.test.ts", 'import(join("davinci/vize_l1", variable));');
    f.write("tests/tooling/davinci-phase4-contract-status.test.ts", "");
    assert.deepEqual(planToolingTests(["davinci/vize_l1/src/lib.rs"], { cwd: f.cwd }).tests, [
      "tests/tooling/native.test.ts",
    ]);
  } finally {
    f.cleanup();
  }
});

void test("a retired-language file arriving or leaving anywhere selects the ledger ratchet", () => {
  const f = fixture();
  const ratchet = "tests/tooling/retired-tooling-languages.test.ts";
  const add = (file: string) => {
    mkdirSync(join(f.cwd, dirname(file)), { recursive: true });
    f.write(file);
  };
  const selected = (...paths: string[]) => planToolingTests(paths, { cwd: f.cwd }).tests;
  try {
    add("tools/config/retired-tooling-languages.json");
    f.write(
      "tools/config/retired-tooling-languages.json",
      JSON.stringify({
        permanent: [{ language: "mjs", prefix: "tests/_fixtures/", reason: "test input" }],
        retiring: { py: [], lean: [], mbt: [], mjs: ["tools/listed.mjs"] },
      }),
    );
    f.write(ratchet, 'const ledger = "tools/config/retired-tooling-languages.json";');
    add("tools/listed.mjs");
    for (const file of [
      "editors/vscode/test/router-host.mjs",
      "tests/typescript-plugin/project-metadata.mjs",
      "crates/vize/scripts/probe.py",
      "davinci/proofs/Spec.lean",
      "davinci/proofs/lean-toolchain",
    ]) {
      add(file);
      assert.ok(selected(file).includes(ratchet), file);
    }
    // A rename in arrives under the new name.
    add("tools/renamed.mjs");
    assert.ok(selected("tools/renamed.ts", "tools/renamed.mjs").includes(ratchet));
    // A directly changed test keeps its own selection alongside the ratchet.
    const direct = "tests/tooling/direct.test.ts";
    add(direct);
    const combined = selected("tools/renamed.mjs", direct);
    assert.ok(combined.includes(ratchet));
    assert.ok(combined.includes(direct));
    // Exempt test inputs and listed files edited in place never select it.
    add("tests/_fixtures/evidence/input.mjs");
    assert.ok(
      !selected("tests/_fixtures/evidence/input.mjs", "tools/listed.mjs").includes(ratchet),
    );
    // A port that keeps the stale entry does.
    rmSync(join(f.cwd, "tools/listed.mjs"));
    add("tools/listed.ts");
    assert.ok(selected("tools/listed.mjs", "tools/listed.ts").includes(ratchet));
    // Invalid or absent authority must keep the test selected to report it.
    f.write("tools/config/retired-tooling-languages.json", "{");
    assert.ok(selected("tools/renamed.mjs").includes(ratchet));
    f.write("tools/config/retired-tooling-languages.json", JSON.stringify({ permanent: [] }));
    assert.ok(selected("tools/renamed.mjs").includes(ratchet));
    rmSync(join(f.cwd, "tools/config/retired-tooling-languages.json"));
    assert.ok(selected("tools/renamed.mjs").includes(ratchet));
  } finally {
    f.cleanup();
  }
});

void test("merge planning restores every scenario, including directly changed deferred files", () => {
  const f = fixture();
  try {
    const paths = ["tests/tooling/lsp-smoke.test.ts"];
    assert.ok(!planToolingTests(paths, { cwd: f.cwd }).tests.includes(paths[0]));
    assert.deepEqual(
      planToolingTests(paths, { cwd: f.cwd, tier: "merge" }).tests,
      toolingTestFiles(f.cwd),
    );
    assert.throws(() => planToolingTests(paths, { cwd: f.cwd, tier: "unknown" }), /tier must/);
  } finally {
    f.cleanup();
  }
});

void test("pure typecheck and LSP helper contracts remain in T0; explicit runtime inventory exists", () => {
  const files = toolingTestFiles();
  for (const file of mergeOnlyToolingTests) assert.ok(files.includes(file), file);
  const plan = planToolingTests(["pnpm-lock.yaml"]);
  assert.ok(plan.tests.includes("tests/tooling/typecheck-dependency-gates.test.ts"));
  assert.ok(plan.tests.includes("tests/tooling/typecheck-baseline-ambient.test.ts"));
  assert.ok(plan.tests.includes("tests/tooling/tooling-test-selection.test.ts"));
  assert.ok(!plan.tests.includes("tests/tooling/lsp-smoke.test.ts"));
  assert.ok(!plan.tests.includes("tests/tooling/typecheck-baseline-project.test.ts"));
});

void test("the runner keeps shared fixtures serial and rejects unrecognized or duplicate plans", () => {
  const available = ["tests/tooling/example.test.ts"];
  const plan = { version: 1, tier: "pr", tests: available };
  assert.deepEqual(toolingTestCommand(plan, available), [
    "--test",
    "--test-concurrency=1",
    ...available,
  ]);
  for (const tests of [["../../outside.test.ts"], [...available, ...available]]) {
    assert.throws(() => toolingTestCommand({ ...plan, tests }, available), /unrecognized/);
  }
  assert.throws(() => toolingTestCommand({ ...plan, version: 2 }, available), /invalid/);
  assert.throws(
    () => toolingTestCommand({ ...plan, tier: "merge", tests: [] }, available),
    /retain every test/,
  );
});

const strictGate = "tests/tooling/typescript-ci-gates.test.ts";
const solutionGate = "tests/tooling/typescript-tooling-project.test.ts";
function typeFixture() {
  const f = fixture();
  const write = (file: string, source: string) => {
    mkdirSync(dirname(join(f.cwd, file)), { recursive: true });
    f.write(file, source);
  };
  write(strictGate, 'const project = "tools/tsconfig.json";');
  write("package.json", '{"type":"module"}');
  write(
    "tools/configuration/strict.json",
    JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        erasableSyntaxOnly: true,
        allowJs: true,
        resolveJsonModule: true,
        module: "NodeNext",
        moduleResolution: "NodeNext",
        types: [],
      },
    }),
  );
  write(
    "tools/tsconfig.json",
    JSON.stringify({
      extends: "./configuration/strict.json",
      include: ["**/*.ts", "**/*.mts", "**/*.cts", "../npm/external/entry.ts"],
      exclude: ["fixtures/**"],
    }),
  );
  write("tools/entry.ts", 'import "../npm/dependencies/emitted.js";');
  write("npm/dependencies/emitted.ts", 'import "./actual.mjs";');
  write(
    "npm/dependencies/actual.mjs",
    'import data from "./data.json"; export const value = data.value;',
  );
  write("npm/dependencies/data.json", '{"value":1}');
  write("npm/external/entry.ts", 'export {other} from "./nested/other.mjs";');
  write("npm/external/nested/other.mjs", "export const other = 1;");
  return {
    ...f,
    write,
    selected: (path: string) => planToolingTests([path], { cwd: f.cwd }).tests,
  };
}

void test("compiler subprocess selection follows actual project roots and transitive JS/JSON inputs", () => {
  const f = typeFixture();
  try {
    for (const path of [
      "tools/new.ts",
      "tools/new.mts",
      "tools/new.cts",
      "tools/configuration/strict.json",
      "tools/tsconfig.json",
      "npm/dependencies/emitted.ts",
      "npm/dependencies/actual.mjs",
      "npm/dependencies/data.json",
      "npm/external/entry.ts",
      "npm/external/nested/other.mjs",
    ]) {
      assert.ok(f.selected(path).includes(strictGate), path);
      assert.ok(!f.selected(path).includes("tests/tooling/lsp-smoke.test.ts"), path);
    }
    for (const path of [
      "crates/vize_patina/src/lib.rs",
      "docs/readme.md",
      "tests/tooling/unrelated.test.ts",
      "tools/fixtures/unimported.ts",
      "npm/external/unrelated.ts",
    ]) {
      assert.ok(!f.selected(path).includes(strictGate), path);
    }
    assert.ok(
      f.selected("unknown-project/new.ts").includes(strictGate),
      "unknown-input fallback stays broad",
    );
    rmSync(join(f.cwd, "npm/dependencies/actual.mjs"));
    assert.ok(f.selected("npm/dependencies/actual.mjs").includes(strictGate), "deleted dependency");
    assert.ok(
      f.selected("npm/dependencies/actual.d.mts").includes(strictGate),
      "new declaration shadows emitted JS",
    );
    f.write("tools/entry.ts", 'import "./fixtures/imported.ts";');
    f.write("tools/fixtures/imported.ts", "export const value=1;");
    assert.ok(
      f.selected("tools/fixtures/imported.ts").includes(strictGate),
      "imports can enter excluded root directories",
    );
  } finally {
    f.cleanup();
  }
});

void test("include and extends mutations preserve scoped or fail-closed compiler selection", () => {
  const f = typeFixture();
  try {
    f.write(
      "tools/tsconfig.json",
      JSON.stringify({
        extends: "./configuration/strict.json",
        include: ["../npm/new-project/**/*.ts"],
      }),
    );
    f.write("npm/new-project/added.ts", "export const added=1;");
    assert.ok(f.selected("npm/new-project/added.ts").includes(strictGate));
    assert.ok(!f.selected("tools/unincluded.ts").includes(strictGate));
    f.write("tools/tsconfig.json", JSON.stringify({ extends: "./configuration/changed.json" }));
    f.write(
      "tools/configuration/changed.json",
      JSON.stringify({ files: ["../../npm/external/entry.ts"] }),
    );
    assert.ok(f.selected("tools/configuration/changed.json").includes(strictGate));
    assert.ok(f.selected("npm/external/nested/other.mjs").includes(strictGate));
    f.write("tools/tsconfig.json", '{"extends":"./missing.json"}');
    assert.ok(f.selected("tools/missing.json").includes(strictGate));
    f.write("tools/tsconfig.json", '{"extends":"./tsconfig.json"}');
    assert.ok(f.selected("tools/tsconfig.json").includes(strictGate));
  } finally {
    f.cleanup();
  }
});

void test("the actual failing helper and every configured external root select the real strict gate", () => {
  const paths = ["tools/support/compat/davinci/lib/rule-parity-plain-types.ts"];
  const project = JSON.parse(
    readFileSync(new URL("../../tools/tsconfig.json", import.meta.url), "utf8"),
  );
  paths.push(
    ...project.include
      .filter((path: string) => path.startsWith("../"))
      .map((path: string) => path.slice(3)),
  );
  const plan = planToolingTests(paths);
  assert.ok(plan.tests.includes(strictGate));
  assert.ok(!plan.tests.includes("tests/tooling/lsp-smoke.test.ts"));
  assert.ok(
    planToolingTests(["tests/tooling/native-shutdown-protocol-fixtures.test.ts"]).tests.includes(
      solutionGate,
    ),
  );
});

void test("solution checking tracks the tests project and imported JS/JSON without adding unrelated projects", () => {
  const f = typeFixture();
  try {
    f.write(solutionGate, 'const checker="tools/support/typescript/check-project.ts";');
    f.write(
      "tsconfig.json",
      JSON.stringify({ files: [], references: [{ path: "./tools" }, { path: "./tests" }] }),
    );
    f.write(
      "tests/tsconfig.json",
      JSON.stringify({
        extends: "../tools/configuration/strict.json",
        include: ["typed/**/*.ts"],
        exclude: ["typed/excluded/**"],
      }),
    );
    f.write("tests/typed/entry.ts", 'import "../../npm/test-worker/worker.mjs";');
    f.write(
      "npm/test-worker/worker.mjs",
      'import data from "./data.json"; export const result=data;',
    );
    f.write("npm/test-worker/data.json", '{"value":1}');
    for (const path of [
      "tests/typed/new.ts",
      "tests/tsconfig.json",
      "tools/configuration/strict.json",
      "npm/test-worker/worker.mjs",
      "npm/test-worker/data.json",
    ]) {
      assert.ok(f.selected(path).includes(solutionGate), path);
      if (path.startsWith("tests/typed") || path.startsWith("npm/test-worker"))
        assert.ok(!f.selected(path).includes(strictGate), path);
    }
    for (const path of [
      "tests/typed/excluded/new.ts",
      "tests/unincluded.ts",
      "npm/unreferenced/new.ts",
      "crates/vize_patina/src/lib.rs",
    ])
      assert.ok(!f.selected(path).includes(solutionGate), path);
    rmSync(join(f.cwd, "npm/test-worker/worker.mjs"));
    assert.ok(
      f.selected("npm/test-worker/worker.mjs").includes(solutionGate),
      "deleted test dependency",
    );
    assert.ok(
      f.selected("npm/test-worker/worker.d.mts").includes(solutionGate),
      "new test declaration",
    );
    const checker = fileURLToPath(
      new URL("../../tools/support/typescript/check-project.ts", import.meta.url),
    );
    f.write(
      "tests/tsconfig.json",
      JSON.stringify({
        extends: "../tools/configuration/strict.json",
        include: ["typed/witness.ts"],
      }),
    );
    const compile = () =>
      spawnSync(process.execPath, [checker, join(f.cwd, "tests/tsconfig.json")], {
        encoding: "utf8",
      });
    f.write(
      "tests/typed/witness.ts",
      "export function identity(value: number) { return value; }\n",
    );
    const correct = compile();
    assert.equal(correct.status, 0, correct.stdout + correct.stderr);
    f.write("tests/typed/witness.ts", "export function identity(value) { return value; }\n");
    assert.ok(f.selected("tests/typed/witness.ts").includes(solutionGate));
    const invalid = compile();
    assert.equal(invalid.error, undefined);
    assert.notEqual(invalid.status, 0);
    assert.match(invalid.stdout + invalid.stderr, /TS7006/u);
  } finally {
    f.cleanup();
  }
});

void test("selected strict compilation rejects the original nullable-never inference regression", () => {
  const f = typeFixture();
  try {
    f.write(
      "tools/tsconfig.json",
      JSON.stringify({ extends: "./configuration/strict.json", include: ["./invalid.ts"] }),
    );
    const source =
      'const fail = (): never => { throw new Error("rejected"); };\nconst pattern = /x/.exec("x");\nif (!pattern) fail();\nexport const value = pattern[0];\n';
    assert.ok(f.selected("tools/invalid.ts").includes(strictGate));
    const checker = fileURLToPath(
      new URL("../../tools/support/typescript/check-project.ts", import.meta.url),
    );
    const compile = () =>
      spawnSync(process.execPath, [checker, join(f.cwd, "tools/tsconfig.json")], {
        encoding: "utf8",
      });
    f.write(
      "tools/invalid.ts",
      source
        .replace("const fail = (): never =>", "function fail(): never")
        .replace("};\nconst pattern", "}\nconst pattern"),
    );
    const correct = compile();
    assert.equal(correct.error, undefined);
    assert.equal(correct.status, 0, correct.stdout + correct.stderr);
    f.write("tools/invalid.ts", source);
    const result = compile();
    assert.equal(result.error, undefined);
    assert.notEqual(result.status, 0);
    assert.match(result.stdout + result.stderr, /TS18047/u);
  } finally {
    f.cleanup();
  }
});

void test("real Nuxt host inputs reach both strict TypeScript project gates before merge", () => {
  const gates = [
    "tests/tooling/typescript-tooling-project.test.ts",
    "tests/tooling/typescript-ci-gates.test.ts",
  ];
  for (const path of [
    "tools/support/compat/nuxt/nuxt2-module-build.ts",
    "tools/support/compat/nuxt/host-types.ts",
    "tools/support/compat/nuxt/nuxt2-typecheck.ts",
    // Deleted project members still invalidate the strict project checks.
    "tools/support/compat/nuxt/deleted-typecheck-probe.ts",
  ]) {
    const plan = planToolingTests([path], { tier: "pr" });
    assert.equal(plan.reason, "import closures and named repository paths");
    for (const gate of gates) assert.ok(plan.tests.includes(gate), `${path}: ${gate}`);
  }
  const prose = planToolingTests(["docs/content/integrations/nuxt.md"], { tier: "pr" });
  assert.ok(!prose.tests.includes(gates[0]), "prose does not change the TypeScript solution");
  // The CI gate contract dynamically loads an authored historical module.
  // Preserve the selector's existing broad fallback for incomplete imports.
  assert.ok(prose.tests.includes(gates[1]));
});
