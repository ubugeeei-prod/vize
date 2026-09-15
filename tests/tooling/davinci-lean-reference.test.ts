import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const formalRoot = path.join(repoRoot, "formal", "impeto");
const fixtureRoot = path.join(formalRoot, "fixtures");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(repoRoot, ...segments), "utf8");
}

test("TS-28 pins the Lean toolchain and CI package directory", () => {
  assert.equal(
    readRepoFile("formal", "impeto", "lean-toolchain").trim(),
    "leanprover/lean4:v4.33.1",
  );

  const workflow = readRepoFile(".github", "workflows", "davinci-lean.yml");
  assert.match(
    workflow,
    /leanprover\/lean-action@50fcf42d2e460296f1a34b402e990d1b24f8b596 # v1\.6\.0/u,
  );
  assert.match(workflow, /lake-package-directory:\s*formal\/impeto/u);
  assert.match(workflow, /lake exe impetoRef --check-fixtures/u);
  assert.match(workflow, /lake exe impetoRef --check-backend-fixtures/u);
});

test("TS-28 fixture ladder is declared and non-vacuous", () => {
  const main = readRepoFile("formal", "impeto", "Main.lean");
  const folios = fs
    .readdirSync(fixtureRoot)
    .filter((file) => file.endsWith(".s3.folio"))
    .sort();

  assert.ok(folios.length >= 3, "TS-28 must start with a non-vacuous fixture ladder");
  assert.ok(
    folios.some((folio) => folio.startsWith("rust-lowered-")),
    "TS-28 must include at least one fixture emitted by the Rust S2->S3 lowering path",
  );
  for (const folio of folios) {
    const stem = folio.replace(/\.s3\.folio$/u, "");
    const traces = [`${stem}.trace`, `${stem}.vdom.trace`, `${stem}.vapor.trace`];
    for (const trace of traces) {
      assert.ok(fs.existsSync(path.join(fixtureRoot, trace)), `${trace} is missing`);
      assert.match(main, new RegExp(`fixtures/${trace.replaceAll(".", "\\.")}`, "u"));
    }
    assert.match(main, new RegExp(`fixtures/${folio.replaceAll(".", "\\.")}`, "u"));
  }
  assert.match(main, /--check-backend-fixtures/u);
  assert.match(main, /--trace-vdom/u);
  assert.match(main, /--trace-vapor/u);
});

test("TS-28 Rust lowering bridge is covered by an ordinary cargo test", () => {
  const bridge = readRepoFile("crates", "vize_s2_to_s3", "tests", "lean_reference_fixture.rs");
  assert.match(bridge, /rust_lowered_fixtures_match_impeto_reference_inputs/u);
  assert.match(bridge, /formal\/impeto\/fixtures\/rust-lowered-static-dynamic\.s3\.folio/u);
  assert.match(bridge, /formal\/impeto\/fixtures\/rust-lowered-control-slots\.s3\.folio/u);
  assert.match(bridge, /S3Folio::of\(&lowered\.program\)\.print_to_string\(FolioMode::Full\)/u);
  assert.match(bridge, /reference_trace_text\(&lowered\.program\)/u);
  assert.match(bridge, /backend_trace_text\(TraceBackend::Vdom, &lowered\.program\)/u);
  assert.match(bridge, /backend_trace_text\(TraceBackend::Vapor, &lowered\.program\)/u);
});

test("TS-28 command in the suite registry names the executable runner", () => {
  const suites = readRepoFile("davinci-road", "plan", "test-suites.md");
  assert.match(
    suites,
    /\| TS-28 \| Lean reference differential\s+\| `cd formal\/impeto && lake exe impetoRef --check-fixtures && lake exe impetoRef --check-backend-fixtures`/u,
  );
});
