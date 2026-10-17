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
  rustTestCohortForBinary,
  rustTestMatrix,
} from "../../tools/support/compat/github/rust-test-cohorts.ts";
import { rustTestTargetProjection } from "../../tools/support/compat/github/run-rust-test-cohort.ts";

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
    assert.equal(rustTestCohortForBinary("vize", binary), cohort);
  }
  assert.deepEqual(owners("vize_canon", "lsp_new_request"), ["typechecker"]);
  assert.deepEqual(owners("vize_maestro", "check_new_project"), ["language-server"]);
  assert.deepEqual(owners("vize_canon_extra", "check_new_project"), []);
});

type Target = { name: string; kind: string[]; test: boolean; "required-features"?: string[] };
const target = (name: string, kind = "test", test = true): Target => ({ name, kind: [kind], test });
function targetMetadata(entries: Record<string, Target[]>) {
  const packages = Object.entries(entries).map(([name, targets]) => ({
    id: name,
    name,
    manifest_path: join(root, "crates", name, "Cargo.toml"),
    dependencies: [],
    targets,
  }));
  return { workspace_root: root, workspace_members: packages.map((pkg) => pkg.id), packages };
}
function projection(metadata: ReturnType<typeof targetMetadata>, cohort = "language-server") {
  return rustTestTargetProjection(
    metadata,
    metadata.packages.map((pkg) => pkg.name),
    cohort,
  );
}

test("Cargo projection keeps shared unit targets and discovers only the cohort's integration targets", () => {
  const metadata = targetMetadata({
    vize: [
      target("vize", "lib"),
      target("vize", "bin"),
      target("lsp_new"),
      target("check_new"),
      target("other"),
    ],
    vize_maestro: [
      target("vize_maestro", "lib"),
      target("future_request"),
      target("disabled", "test", false),
    ],
    vize_l1: [target("vize_l1", "lib"), target("compiler_regression")],
  });
  assert.deepEqual(projection(metadata).args, [
    "--lib",
    "--bins",
    "--test",
    "future_request",
    "--test",
    "lsp_new",
  ]);
  assert.deepEqual(projection(metadata, "compiler").args, [
    "--lib",
    "--bins",
    "--test",
    "compiler_regression",
  ]);
  assert.deepEqual(projection(metadata, "typechecker").args, [
    "--lib",
    "--bins",
    "--test",
    "check_new",
  ]);
  assert.deepEqual(projection(metadata, "cli-and-integrations").args, [
    "--lib",
    "--bins",
    "--test",
    "other",
  ]);
  assert.deepEqual(projection(metadata, "lint-and-format").args, ["--lib", "--bins"]);
  assert.throws(() => projection(metadata, "unknown"), /not selected/);
});

test("duplicate Cargo target names stay unique and cannot force disabled or gated targets", () => {
  const metadata = targetMetadata({
    vize: [target("vize", "lib"), target("lsp_shared")],
    vize_maestro: [target("vize_maestro", "lib"), target("lsp_shared")],
    vize_canon: [target("vize_canon", "lib"), target("lsp_shared")],
  });
  assert.deepEqual(projection(metadata).args, ["--lib", "--test", "lsp_shared"]);
  metadata.packages[2].targets[1].test = false;
  assert.match(projection(metadata).reason ?? "", /disabled/);
  metadata.packages[2].targets[1].test = true;
  metadata.packages[2].targets[1]["required-features"] = ["native"];
  assert.match(projection(metadata).reason ?? "", /feature-gated/);
});

test("Cargo remains authoritative for required features, including future gated targets", () => {
  const metadata = targetMetadata({
    vize: [
      target("vize", "lib"),
      target("lsp_new"),
      { ...target("check_gated"), "required-features": ["future-feature"] },
    ],
    vize_maestro: [target("vize_maestro", "lib")],
  });
  assert.deepEqual(projection(metadata).args, ["--lib", "--test", "lsp_new"]);
  assert.deepEqual(projection(metadata, "typechecker").args, []);
  metadata.packages[0].targets.push({
    ...target("lsp_future"),
    "required-features": ["future-feature"],
  });
  assert.deepEqual(projection(metadata).args, []);
});

test("one existing cohort retains compile-only examples without changing their harness", () => {
  const metadata = targetMetadata({
    vize_l1: [target("vize_l1", "lib"), target("compiler_case")],
    vize_maestro: [
      target("vize_maestro", "lib"),
      target("lsp_case"),
      target("example_main", "example", false),
    ],
  });
  assert.equal(projection(metadata, "compiler").reason, "retain default example compilation");
  assert.deepEqual(projection(metadata).args, ["--lib", "--test", "lsp_case"]);
  assert.deepEqual(
    projection(targetMetadata({ vize_maestro: metadata.packages[1].targets })).args,
    [],
  );
  metadata.packages[0].targets.push({
    ...target("compiler_gated"),
    "required-features": ["legacy"],
  });
  assert.match(projection(metadata, "compiler").reason ?? "", /feature-gated/);
  assert.deepEqual(projection(metadata).args, ["--lib", "--test", "lsp_case"]);
});

test("library kinds, binary-only, test-only and benchmark-only packages preserve valid Cargo selection", () => {
  for (const kinds of [["cdylib", "rlib"], ["proc-macro"], ["lib"]]) {
    const metadata = targetMetadata({
      vize_maestro: [{ name: "vize_maestro", kind: kinds, test: true }],
    });
    assert.deepEqual(projection(metadata).args, ["--lib"]);
  }
  assert.deepEqual(projection(targetMetadata({ vize_maestro: [target("server", "bin")] })).args, [
    "--bins",
  ]);
  assert.deepEqual(projection(targetMetadata({ vize_maestro: [target("request")] })).args, [
    "--test",
    "request",
  ]);
  assert.deepEqual(
    projection(
      targetMetadata({ vize_benchmarks: [target("benchmark", "bench", false)] }),
      "cli-and-integrations",
    ).args,
    [],
  );
});

test("nonstandard targets preserve Cargo defaults and malformed metadata fails closed", () => {
  for (const unusual of [
    target("hidden", "lib", false),
    target("hidden", "bin", false),
    target("runnable", "example"),
    target("runnable", "bench"),
    target("future", "future-kind"),
  ]) {
    assert.deepEqual(projection(targetMetadata({ vize_maestro: [unusual] })).args, []);
  }
  assert.throws(() => projection(targetMetadata({ vize_maestro: [] })), /missing Cargo targets/);
  assert.throws(
    () => projection(targetMetadata({ vize_maestro: [target("duplicate"), target("duplicate")] })),
    /duplicate Cargo target/,
  );
  const metadata = targetMetadata({ vize_maestro: [target("request")] });
  delete (metadata.packages[0].targets[0] as Partial<Target>).test;
  assert.throws(() => projection(metadata), /invalid Cargo target metadata/);
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
