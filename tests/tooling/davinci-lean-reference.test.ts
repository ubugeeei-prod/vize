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
});

test("TS-28 fixture ladder is declared and non-vacuous", () => {
  const main = readRepoFile("formal", "impeto", "Main.lean");
  const folios = fs
    .readdirSync(fixtureRoot)
    .filter((file) => file.endsWith(".s3.folio"))
    .sort();

  assert.ok(folios.length >= 3, "TS-28 must start with a non-vacuous fixture ladder");
  for (const folio of folios) {
    const trace = folio.replace(/\.s3\.folio$/u, ".trace");
    assert.ok(fs.existsSync(path.join(fixtureRoot, trace)), `${trace} is missing`);
    assert.match(main, new RegExp(`fixtures/${folio.replaceAll(".", "\\.")}`, "u"));
    assert.match(main, new RegExp(`fixtures/${trace.replaceAll(".", "\\.")}`, "u"));
  }
});

test("TS-28 command in the suite registry names the executable runner", () => {
  const suites = readRepoFile("davinci-road", "plan", "test-suites.md");
  assert.match(
    suites,
    /\| TS-28 \| Lean reference differential\s+\| `cd formal\/impeto && lake exe impetoRef --check-fixtures`/u,
  );
});
