import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { LEDGER_REL } from "../../tools/support/compat/davinci/lib/lint-substrate.ts";
import { sourceInventoryLanes } from "../../tools/support/compat/davinci/source-inventories.ts";
import { planAffectedRust } from "../../tools/support/compat/github/plan-affected-rust.ts";
import type { CargoMetadata } from "../../tools/support/compat/github/plan-affected-rust.ts";
import { planPrChecks, prCheckOutputs } from "../../tools/support/compat/github/plan-pr-checks.ts";
import {
  isCensusToolingInput,
  planSourceChecks,
} from "../../tools/support/compat/github/plan-source-checks.ts";
import {
  localImportInputs,
  planToolingTests,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const ledger = "docs/davinci/plan/lint-substrate-ledger.tsv";
const toolingOnly = { rust: false, js: false, tooling: true, playground: false };
const productReader = /lint[-_]substrate|substrate[-_]ledger/u;

function metadata(): CargoMetadata {
  const dependencies: Record<string, CargoMetadata["packages"][number]["dependencies"]> = {
    vize_relief: [],
    vize_patina: [{ path: `${root}crates/vize_relief`, kind: null }],
    vize_vitrine: [{ path: `${root}crates/vize_patina`, kind: null }],
    vize_maestro: [{ path: `${root}crates/vize_patina`, kind: "build" }],
    vize: [{ path: `${root}crates/vize_maestro`, kind: "dev" }],
    vize_curator: [],
  };
  const packages = Object.entries(dependencies).map(([name, dependencies]) => ({
    id: name,
    name,
    manifest_path: `${root}crates/${name}/Cargo.toml`,
    dependencies,
  }));
  return { workspace_root: root, workspace_members: packages.map(({ id }) => id), packages };
}

const workspace = [
  { name: "@vizejs/native", path: "npm/native", dependencies: [] },
  { name: "@vizejs/vite-plugin", path: "npm/builder/vite", dependencies: ["@vizejs/native"] },
  { name: "@vizejs/fresco-native", path: "npm/fresco-native", dependencies: [] },
];

function request(paths: string[], eventName = "pull_request") {
  return planPrChecks({ paths, eventName, labels: [], workspace, cargoMetadata: metadata() });
}

void test("only the exact generated Patina inventory retains Node ownership", () => {
  assert.equal(LEDGER_REL, ledger);
  assert.equal(isCensusToolingInput(ledger), true);
  assert.deepEqual(planSourceChecks([ledger]), toolingOnly);
  const affected = planAffectedRust(metadata(), [ledger]);
  assert.equal(affected.scope, "none");
  assert.deepEqual(affected.packages, []);
  assert.deepEqual(affected.reasons, []);
  const plan = request([ledger]);
  assert.deepEqual(plan.rust.packages, ["vize_relief"]);
  assert.deepEqual(plan.js.packages, []);
  assert.deepEqual(plan.native.matrix, { lane: ["none"] });
  assert.equal(plan.sourceContracts, true);
  assert.equal(prCheckOutputs(plan).rust, "true");
  assert.equal(prCheckOutputs(plan).tooling, "true");
});

void test("the mandatory source inventory and exact stale-ledger reader remain enrolled", () => {
  assert.deepEqual(
    sourceInventoryLanes
      .flat()
      .filter(([program]) => program === "tools/support/compat/davinci/lint-substrate.ts"),
    [["tools/support/compat/davinci/lint-substrate.ts", "--check"]],
  );
  const consumer = "tests/tooling/lint-substrate-ledger.test.ts";
  assert.equal(
    localImportInputs(consumer).files.includes(
      "tools/support/compat/davinci/lib/lint-substrate.ts",
    ),
    true,
  );
  const selected = planToolingTests([ledger]).tests;
  assert.equal(selected.includes(consumer), true);
  assert.equal(selected.includes("tests/tooling/lint-substrate-source-scope.test.ts"), true);
  const checker = readFileSync(
    join(root, "tools/support/compat/davinci/lint-substrate.ts"),
    "utf8",
  );
  assert.match(checker, /readFileSync\(target, "utf8"\) !== text/u);
  assert.match(checker, /if \(!existsSync\(target\)\)/u);
});

void test("actual Rust and npm product sources retain no ledger read consumer", () => {
  const tracked = execFileSync(
    "git",
    ["ls-files", "-z", "--", "crates", "davinci", "npm", "tests/davinci_test_support"],
    { cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  ).split("\0");
  const sources = tracked.filter((path) =>
    path.startsWith("npm/")
      ? /\.(?:[cm]?[jt]sx?|json|vue|sh|py|ya?ml|toml)$/u.test(path)
      : /(?:\.rs|(?:^|\/)Cargo\.toml)$/u.test(path),
  );
  assert.equal(
    sources.some((path) => path.startsWith("crates/")),
    true,
  );
  assert.equal(
    sources.some((path) => path.startsWith("npm/")),
    true,
  );
  for (const path of sources)
    assert.doesNotMatch(readFileSync(join(root, path), "utf8"), productReader, path);
  for (const source of [
    'include_str!("../../../docs/davinci/plan/lint-substrate-ledger.tsv")',
    'fs::read_to_string(root.join("lint-substrate-ledger.tsv"))',
    'readFileSync("docs/davinci/plan/lint-substrate-ledger.tsv")',
    'import "../../../tools/support/compat/davinci/lint-substrate.ts"',
  ])
    assert.throws(() => assert.doesNotMatch(source, productReader));
});

void test("mixed Patina source preserves every actual Rust, JS, native and feature selection", () => {
  const path = "crates/vize_patina/src/rules/security/no_unsafe_url.rs";
  const before = request([path]);
  const mixed = request([path, ledger]);
  assert.deepEqual(before.rust.packages, ["vize", "vize_maestro", "vize_patina", "vize_vitrine"]);
  assert.equal(before.js.native, true);
  assert.deepEqual(before.native.deferred, ["lsp"]);
  assert.deepEqual(mixed, { ...before, source: { ...before.source, tooling: true } });
});

void test("merge groups keep full required Rust checks while source dependencies stay exact", () => {
  const pr = request([ledger]);
  const queue = request([ledger], "merge_group");
  assert.equal(queue.rust.scope, "workspace");
  assert.deepEqual(
    queue.rust.packages,
    metadata()
      .packages.map(({ name }) => name)
      .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0)),
  );
  assert.deepEqual(queue.js, pr.js);
  assert.deepEqual(queue.native.matrix, pr.native.matrix);
  assert.equal(prCheckOutputs(queue).rust, "true");
  assert.equal(queue.sourceContracts, true);
});

void test("nearby, renamed, unknown and manifest inputs remain conservative", () => {
  const all = planAffectedRust(metadata(), ["Cargo.lock"]);
  for (const path of [
    "docs/davinci/plan/lint-substrate-ledger.tsv.bak",
    "docs/davinci/plan/lint-substrate-ledger.md",
    "docs/davinci/plan/lint-substrate-ledger.ts",
    "docs/davinci/plan/lint-substrate-ledger.schema.json",
    "docs/davinci/plan/nested/lint-substrate-ledger.tsv",
    "docs/davinci/plan/Lint-substrate-ledger.tsv",
    "docs/davinci/plan/./lint-substrate-ledger.tsv",
    "docs/davinci/plan/budgets.toml",
    "docs/package.json",
    "npm/cli/schemas/vize.config.schema.json",
    "docs/scripts/new-consumer.ts",
    "tools/support/compat/davinci/lint-substrate.ts",
    "tools/support/compat/davinci/lib/lint-substrate.ts",
    "new-runtime/unowned.ts",
  ]) {
    assert.equal(isCensusToolingInput(path), false, path);
    assert.deepEqual(
      planSourceChecks([ledger, path]),
      { rust: true, js: true, tooling: true, playground: true },
      path,
    );
    const selected = planAffectedRust(metadata(), [ledger, path]);
    assert.equal(selected.scope, "workspace", path);
    assert.deepEqual(selected.packages, all.packages, path);
    assert.deepEqual(request([ledger, path]).rust, request([path]).rust, path);
    assert.deepEqual(request([ledger, path]).js, request([path]).js, path);
    assert.deepEqual(request([ledger, path]).native, request([path]).native, path);
  }
});

void test("deleted Rust owners, Pkl, shared manifests and workflow changes keep their existing policy", () => {
  for (const path of [
    "crates/deleted/src/lib.rs",
    "Cargo.lock",
    "npm/cli/pkl/new-contract.pkl",
    "tools/config/pkl/workflows/checks/01-rust-feature-tests.pkl",
    ".github/workflows/pr-source-checks.yml",
  ]) {
    const before = request([path]);
    const mixed = request([ledger, path]);
    assert.deepEqual(mixed.rust, before.rust, path);
    assert.deepEqual(mixed.js, before.js, path);
    assert.deepEqual(mixed.native, before.native, path);
    assert.deepEqual(mixed.source, { ...before.source, tooling: true }, path);
  }
});
