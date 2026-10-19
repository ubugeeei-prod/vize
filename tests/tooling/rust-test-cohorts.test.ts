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
function matches(filter: string, pkg: string, binary: string, name: string) {
  return filter.split(" | ").some((clause) => {
    const exact = clause.match(/^package\(=([\w-]+)\)$/);
    if (exact) return exact[1] === pkg;
    const cli = clause.match(
      /^\(package\(=vize\) & (not )?\(binary\(=integration\) & test\(\/(.+)\/\)\)\)$/,
    );
    assert.ok(cli, `unsupported nextest clause: ${clause}`);
    const included = binary === "integration" && new RegExp(cli[2]).test(name);
    return pkg === "vize" && (cli[1] ? !included : included);
  });
}

function owners(pkg: string, binary: string, name = "case") {
  return RUST_TEST_COHORTS.filter((cohort) => matches(cohort.filter, pkg, binary, name)).map(
    (cohort) => cohort.id,
  );
}

// A former CLI test file is now a module of the single integration binary.
const cliModule = (module: string) => owners("vize", "integration", `${module}::case`);

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
    for (const module of [name, "future_regression", "check_new", "lsp_new", "fmt_new"]) {
      assert.equal(owners(name, "integration", `${module}::case`).length, 1, `${name}::${module}`);
      assert.equal(owners(name, name, `${module}::case`).length, 1, `${name}::${module}`);
    }
  }
});

test("every existing CLI integration module belongs to exactly one semantic cohort", () => {
  const modules = readdirSync(join(root, "crates/vize/tests"))
    .filter((name) => name.endsWith(".rs") && name !== "main.rs")
    .map((name) => name.slice(0, -3));
  assert.ok(modules.length > 0);
  for (const module of modules) assert.equal(cliModule(module).length, 1, module);
  for (const [module, cohort] of [
    ["check_new_project", "typechecker"],
    ["typecheck_new_project", "typechecker"],
    ["lsp_new_request", "language-server"],
    ["lint_new_rule", "lint-and-format"],
    ["fmt_new_document", "lint-and-format"],
    ["format_new_document", "lint-and-format"],
    ["future_command", "cli-and-integrations"],
    ["my_lsp_command", "cli-and-integrations"],
    ["check", "cli-and-integrations"],
  ]) {
    assert.deepEqual(cliModule(module), [cohort], module);
  }
  // The library and binary unit tests are not integration modules.
  assert.deepEqual(owners("vize", "vize", "lsp_new::case"), ["cli-and-integrations"]);
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
