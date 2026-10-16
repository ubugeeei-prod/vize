import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import type { CargoMetadata } from "../../tools/support/compat/github/plan-affected-rust.ts";
import {
  isRustTestCohort,
  RUST_TEST_COHORT_IDS,
  RUST_TEST_COHORTS,
  rustTestMatrix,
} from "../../tools/support/compat/github/rust-test-cohorts.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));

// The workspace uses explicit member paths, so these inventory checks need no
// Cargo installation, registry access, or compilation in the fast Node lane.
function workspaceMetadata(): CargoMetadata {
  const manifest = readFileSync(join(root, "Cargo.toml"), "utf8");
  const members = manifest.match(/^members\s*=\s*\[([^\]]+)\]/m);
  assert.ok(members, "workspace must declare its member paths");
  const packages = [...members[1].matchAll(/"([^"]+)"/g)].map(([, path]) => {
    const manifest_path = join(root, path, "Cargo.toml");
    const source = readFileSync(manifest_path, "utf8");
    const name = source.match(/^name\s*=\s*"([^"]+)"/m)?.[1];
    assert.ok(name, `missing package name: ${path}`);
    return { id: name, name, manifest_path, dependencies: [] };
  });
  return { workspace_root: root, workspace_members: packages.map((pkg) => pkg.id), packages };
}

// Evaluate the small documented nextest filter vocabulary emitted by the
// registry, so coverage tests exercise the actual expressions handed to CI.
function matches(filter: string, pkg: string, binary: string) {
  return filter.split(" | ").some((clause) => {
    const exact = clause.match(/^package\(=([\w-]+)\)$/);
    if (exact) return exact[1] === pkg;
    const cli = clause.match(/^\(package\(=vize\) & (not )?binary\(\/(.+)\/\)\)$/);
    assert.ok(cli, `unsupported nextest clause: ${clause}`);
    const included = new RegExp(cli[2]).test(binary);
    return pkg === "vize" && (cli[1] ? !included : included);
  });
}

function owners(pkg: string, binary: string) {
  return RUST_TEST_COHORTS.filter((cohort) => matches(cohort.filter, pkg, binary)).map(
    (cohort) => cohort.id,
  );
}

test("every current workspace package has one owner and future test names remain covered", () => {
  const metadata = workspaceMetadata();
  const matrix = rustTestMatrix(
    metadata,
    metadata.packages.map((pkg) => pkg.name),
  );
  assert.deepEqual(
    matrix.include.map((entry) => entry.cohort),
    RUST_TEST_COHORT_IDS,
  );
  for (const { name } of metadata.packages) {
    assert.equal(RUST_TEST_COHORTS.filter((cohort) => cohort.packages.includes(name)).length, 1);
    for (const binary of [name, "future_regression", "check_new", "lsp_new", "fmt_new"]) {
      assert.equal(owners(name, binary).length, 1, `${name}::${binary}`);
    }
  }
});

test("every existing CLI integration binary belongs to exactly one semantic cohort", () => {
  const binaries = readdirSync(join(root, "crates/vize/tests"))
    .filter((name) => name.endsWith(".rs"))
    .map((name) => name.slice(0, -3));
  assert.ok(binaries.length > 0);
  for (const binary of binaries) assert.equal(owners("vize", binary).length, 1, binary);
  for (const [binary, cohort] of [
    ["check_new_project", "typechecker"],
    ["typecheck_new_project", "typechecker"],
    ["lsp_new_request", "language-server"],
    ["lint_new_rule", "lint-and-format"],
    ["fmt_new_document", "lint-and-format"],
    ["format_new_document", "lint-and-format"],
    ["vize", "cli-and-integrations"],
    ["future_command", "cli-and-integrations"],
    ["my_lsp_command", "cli-and-integrations"],
    ["check", "cli-and-integrations"],
  ]) {
    assert.deepEqual(owners("vize", binary), [cohort], binary);
  }
  assert.deepEqual(owners("vize_canon", "lsp_new_request"), ["typechecker"]);
  assert.deepEqual(owners("vize_maestro", "check_new_project"), ["language-server"]);
  assert.deepEqual(owners("vize_canon_extra", "check_new_project"), []);
});

test("affected selection starts only cohorts that contain selected packages", () => {
  const metadata = workspaceMetadata();
  const selected = (...packages: string[]) =>
    rustTestMatrix(metadata, packages).include.map((cohort) => cohort.cohort);
  assert.deepEqual(selected("vize_relief"), ["compiler"]);
  assert.deepEqual(selected("vize_canon"), ["typechecker"]);
  assert.deepEqual(selected("vize_maestro"), ["language-server"]);
  assert.deepEqual(selected("vize_patina", "vize_glyph"), ["lint-and-format"]);
  assert.deepEqual(selected("vize_vitrine"), ["cli-and-integrations"]);
  assert.deepEqual(selected("vize"), [
    "typechecker",
    "language-server",
    "lint-and-format",
    "cli-and-integrations",
  ]);
  assert.deepEqual(selected("vize", "vize_relief"), RUST_TEST_COHORT_IDS);
});

test("each checked Corsa provider selects all of its test binaries in the required typechecker cohort", () => {
  const metadata = workspaceMetadata();
  for (const name of [
    "vize_corsa_core",
    "vize_corsa_jsonrpc",
    "vize_corsa_client",
    "vize_corsa_lsp",
    "vize_corsa_orchestrator",
    "vize_corsa",
  ]) {
    assert.deepEqual(
      rustTestMatrix(metadata, [name]).include.map((entry) => entry.cohort),
      ["typechecker"],
    );
    for (const binary of [name, "api_async", "lsp_stdio", "future_native_regression"]) {
      assert.deepEqual(owners(name, binary), ["typechecker"], `${name}::${binary}`);
    }
  }
});

test("unknown packages and empty selections fail closed, even outside the affected set", () => {
  const metadata = workspaceMetadata();
  const unknown = {
    id: "new_product",
    name: "new_product",
    manifest_path: join(root, "crates/new_product/Cargo.toml"),
    dependencies: [],
  };
  assert.throws(
    () =>
      rustTestMatrix(
        {
          ...metadata,
          workspace_members: [...metadata.workspace_members, unknown.id],
          packages: [...metadata.packages, unknown],
        },
        ["vize_relief"],
      ),
    /exactly one Rust test cohort: new_product/,
  );
  assert.throws(() => rustTestMatrix(metadata, ["outside"]), /outside the workspace/);
  assert.throws(() => rustTestMatrix(metadata, []), /at least one selected package/);
});

test("worker identities accept only the named registry, including rejection of legacy numbers", () => {
  for (const id of RUST_TEST_COHORT_IDS) assert.equal(isRustTestCohort(id), true);
  for (const invalid of [1, "1", "1/4", "Compiler", "unknown", "", null, undefined]) {
    assert.equal(isRustTestCohort(invalid), false);
  }
});
