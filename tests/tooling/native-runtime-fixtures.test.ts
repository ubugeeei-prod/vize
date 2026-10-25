import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  analyzeLegacyFix,
  gitFixtureFile,
} from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import {
  runtimeFixturePath,
  unchangedNonRuntimeErrors,
  unchangedNativeRuntimeHelp,
  unchangedGlobalDiagnosticMock,
} from "../../tools/support/compat/github/native-runtime-fixtures.ts";
import { planPrChecks } from "../../tools/support/compat/github/plan-pr-checks.ts";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";
import { loadLinterManifest } from "../differential/linter-api.ts";

type Product = "linter" | "typechecker";
type RuntimeCase = {
  id: string;
  state: string;
  adapter: string;
  targets: string[];
  input: { path: string; sha256: string };
  sources: Array<{ path: string; before: string | null; after: string }>;
  tests: Array<{ selector: string; source: string; sha256: string }>;
};
type Manifest = {
  cases: Array<{ id: string }>;
  cohorts?: Array<{ id: string; cases: { path: string } }>;
  runtimeCases: RuntimeCase[];
};
const root = new URL("../../", import.meta.url);
const baseline = "d8979e5f946293845aa1e824c5a0045a90ff058e";
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const pathFor = (product: Product) => `tests/_fixtures/differential/${product}/manifest.json`;
const manifestFor = (product: Product) =>
  JSON.parse(readFileSync(new URL(pathFor(product), root), "utf8")) as Manifest;
const images = JSON.parse(
  readFileSync(new URL("tests/tooling/fixtures/native-runtime-source-images.json", root), "utf8"),
) as {
  base: Record<string, string | null>;
  head: Record<string, string>;
};
function sourceImage(side: "base" | "head", path: string): Buffer | null {
  const text = images[side][path];
  return typeof text === "string" ? Buffer.from(text) : null;
}
function analyze(
  product: Product,
  options: {
    rows?: unknown;
    previous?: unknown;
    paths?: string[];
    cases?: object[];
    files?: Record<string, Buffer | null>;
  } = {},
) {
  const manifest = manifestFor(product);
  const row = manifest.runtimeCases[0];
  const paths = options.paths ?? row.sources.map(({ path }) => path);
  return analyzeLegacyFix({
    title: "fix(runtime): select the project native compiler",
    paths,
    readFile: (side, path) => {
      if (path === pathFor(product))
        return Buffer.from(
          JSON.stringify({
            ...manifest,
            cases: side === "base" ? manifest.cases : [...manifest.cases, ...(options.cases ?? [])],
            runtimeCases: side === "base" ? (options.previous ?? []) : (options.rows ?? [row]),
          }),
        );
      if (side === "base") return sourceImage(side, path);
      if (Object.hasOwn(options.files ?? {}, path)) return options.files?.[path] ?? null;
      if (Object.hasOwn(images.head, path)) return sourceImage("head", path);
      try {
        return readFileSync(new URL(path, root));
      } catch {
        return null;
      }
    },
  }).products.find((entry) => entry.product === product);
}

void test("actual Rust graph registrations qualify exact source pairs outside language denominators", () => {
  for (const product of ["linter", "typechecker"] as const) {
    const manifest = manifestFor(product);
    assert.deepEqual(analyze(product), {
      product,
      state: "new-inputs-registered",
      cases: [manifest.runtimeCases[0].id],
    });
    const loaded =
      product === "typechecker"
        ? loadTypecheckerManifest(fileURLToPath(new URL(pathFor(product), root)))
        : loadLinterManifest(fileURLToPath(new URL(pathFor(product), root)), fileURLToPath(root));
    assert.deepEqual(loaded.manifest.cases, manifest.cases);
    // Linter input-only registrations were already outside its executable
    // cohort denominator. Preserve both inventories rather than equating them.
    const planned =
      product === "typechecker"
        ? manifest.cases.map(({ id }) => id)
        : manifest.cohorts!.flatMap((cohort) => {
            const cases = JSON.parse(
              readFileSync(new URL(cohort.cases.path, root), "utf8"),
            ) as Array<{ id: string }>;
            return cases.map(({ id }) => `linter/${cohort.id}/${id}`);
          });
    assert.deepEqual(
      loaded.cases.map((entry: { id: string }) => entry.id).toSorted(),
      planned.toSorted(),
    );
    assert(!loaded.cases.some((entry: { id: string }) => entry.id === manifest.runtimeCases[0].id));
  }
});

void test("stale, missing, inactive and forged runtime identities cannot qualify", () => {
  for (const product of ["linter", "typechecker"] as const) {
    const row = manifestFor(product).runtimeCases[0];
    assert.equal(analyze(product, { previous: [row] })?.state, "new-input-missing");
    assert.equal(analyze(product, { rows: [] })?.state, "new-input-missing");
    for (const rows of [
      [row, row],
      [{ ...row, state: "draft" }],
      [{ ...row, id: `${row.id}-forged` }],
      [{ ...row, adapter: "other" }],
      [{ ...row, dialectCoverage: { state: "verified" } }],
    ]) {
      assert.equal(analyze(product, { rows })?.state, "invalid-manifest-or-input");
    }
  }
});

void test("missing, malformed, empty and hash-drifted graph packs fail closed", () => {
  for (const product of ["linter", "typechecker"] as const) {
    assert.equal(
      analyze(product, { files: { [runtimeFixturePath]: null } })?.state,
      "invalid-manifest-or-input",
    );
    assert.equal(
      analyze(product, { files: { [runtimeFixturePath]: Buffer.from("drift") } })?.state,
      "invalid-manifest-or-input",
    );
    for (const raw of [
      "[]",
      "null",
      "{}",
      "{",
      '{"schema":"vize.native-runtime-selection","version":1}',
    ]) {
      const bytes = Buffer.from(raw);
      const row = structuredClone(manifestFor(product).runtimeCases[0]);
      row.input.sha256 = hash(bytes);
      assert.equal(
        analyze(product, { rows: [row], files: { [runtimeFixturePath]: bytes } })?.state,
        "invalid-manifest-or-input",
      );
    }
  }
});

void test("source, selector and test body mutations reject even when their supplied hash is updated", () => {
  for (const product of ["linter", "typechecker"] as const) {
    for (const mutate of [
      (row: RuntimeCase) => {
        row.sources[0].before = "0".repeat(64);
      },
      (row: RuntimeCase) => {
        row.sources[0].after = "0".repeat(64);
      },
      (row: RuntimeCase) => {
        row.sources[0].path = "crates/vize_canon/src/batch.rs";
      },
      (row: RuntimeCase) => {
        row.tests[0].selector += "_forged";
      },
      (row: RuntimeCase) => {
        row.tests[0].source = "crates/vize_canon/src/batch.rs";
      },
      (row: RuntimeCase) => {
        row.tests.pop();
      },
    ]) {
      const row = structuredClone(manifestFor(product).runtimeCases[0]);
      mutate(row);
      assert.equal(analyze(product, { rows: [row] })?.state, "invalid-manifest-or-input");
    }
    const row = structuredClone(manifestFor(product).runtimeCases[0]);
    const path = row.sources[0].path;
    const bytes = Buffer.concat([
      sourceImage("head", path)!,
      Buffer.from("\n// unrelated future edit\n"),
    ]);
    row.sources[0].after = hash(bytes);
    assert.equal(
      analyze(product, { rows: [row], files: { [path]: bytes } })?.state,
      "invalid-manifest-or-input",
    );
  }
});

void test("mixed and future product edits still need normal language inputs", () => {
  for (const product of ["linter", "typechecker"] as const) {
    const row = manifestFor(product).runtimeCases[0];
    const other =
      product === "linter" ? "crates/vize_patina/src/lib.rs" : "crates/vize_canon/src/batch.rs";
    const paths = [...row.sources.map(({ path }) => path), other];
    assert.equal(analyze(product, { paths })?.state, "new-input-missing");
    const bytes = Buffer.from("<template><div /></template>\n");
    const path = "runtime-gate-unit-control/App.vue.txt";
    const cases = [
      {
        id: `${product}/runtime-gate-unit-control/new`,
        state: "active",
        targets: [product === "linter" ? "lint" : "batch"],
        inputs: [{ path, sha256: hash(bytes) }],
      },
    ];
    const files = { [`tests/_fixtures/differential/${product}/${path}`]: bytes };
    assert.equal(analyze(product, { paths, cases, files })?.state, "new-inputs-registered");
    assert.equal(
      analyze(product, { paths: [other], rows: [], cases, files })?.state,
      "new-inputs-registered",
    );
    assert.equal(analyze(product, { paths: [other], rows: [] })?.state, "new-input-missing");
  }
});

void test("mixed batch errors retain every unrelated enum, conversion and detection byte", () => {
  const path = "crates/vize_canon/src/batch/error.rs";
  const before = sourceImage("base", path)?.toString("utf8");
  assert(before);
  const after = sourceImage("head", path)!.toString("utf8");
  assert(unchangedNonRuntimeErrors(before, after));
  assert(
    !unchangedNonRuntimeErrors(before, after.replace("SfcParse(String)", "SfcParse(PathBuf)")),
  );
  assert(
    !unchangedNonRuntimeErrors(
      before,
      after.replace("pub fn detect_package_manager", "pub fn changed_package_manager"),
    ),
  );
});

void test("the normal planner selects all three actual Rust fixture producer cohorts", () => {
  const names = ["vize_carton", "vize_canon", "vize_patina"];
  const cargoMetadata = {
    workspace_root: "/repo",
    workspace_members: names,
    packages: names.map((name) => ({
      id: name,
      name,
      manifest_path: `/repo/crates/${name}/Cargo.toml`,
      dependencies: [],
    })),
  };
  for (const eventName of ["pull_request", "merge_group"]) {
    const plan = planPrChecks({
      paths: [runtimeFixturePath],
      eventName,
      labels: [],
      workspace: [],
      cargoMetadata,
    });
    assert.equal(plan.source.rust, true);
    assert.deepEqual(plan.rust.packages.toSorted(), names.toSorted());
    assert.deepEqual(
      plan.rust.testMatrix.include.map(({ cohort }) => cohort),
      ["compiler", "typechecker", "lint-and-format"],
    );
  }
});

void test("the Git reader distinguishes new-on-disk files from unreadable revisions", () => {
  assert.equal(
    gitFixtureFile(baseline, "tests/tooling/native-runtime-fixtures.test.ts", fileURLToPath(root)),
    null,
  );
  assert.deepEqual(
    gitFixtureFile("HEAD", "Cargo.toml", fileURLToPath(root)),
    readFileSync(new URL("Cargo.toml", root)),
  );
  assert.throws(
    () => gitFixtureFile("0".repeat(40), "Cargo.toml", fileURLToPath(root)),
    /git show failed/,
  );
});

void test("the type-aware help correction retains all unrelated source and packet bytes", () => {
  for (const path of [
    "crates/vize_patina/src/linter/native_type_aware.rs",
    "crates/vize_patina/src/linter/native_type_aware/plain_module_tests/packets.rs",
  ]) {
    const before = sourceImage("base", path)!.toString("utf8");
    const after = sourceImage("head", path)!.toString("utf8");
    assert(unchangedNativeRuntimeHelp(before, after));
    assert(!unchangedNativeRuntimeHelp(before, `${after}\n// unrelated source edit\n`));
  }
});

void test("explicit fake runtime selection retains every old source and global diagnostic assertion", () => {
  const path = "crates/vize_canon/src/batch/executor/tests.rs";
  const before = sourceImage("base", path)!.toString("utf8");
  const after = sourceImage("head", path)!.toString("utf8");
  assert(unchangedGlobalDiagnosticMock(before, after));
  assert(!unchangedGlobalDiagnosticMock(before, before));
  assert(!unchangedGlobalDiagnosticMock(before, `${after}\n// unrelated test edit\n`));
  assert(
    !unchangedGlobalDiagnosticMock(
      before,
      after.replace("assert_eq!(result.exit_code, 2);", "assert_eq!(result.exit_code, 0);"),
    ),
  );
});
