import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import { VAPOR_STYLE_API_CASE_ID } from "../../tools/support/compat/github/vapor-style-api-fixtures.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const manifestPath = "tests/_fixtures/differential/compiler/manifest.json";
const manifest = JSON.parse(fs.readFileSync(path.join(root, manifestPath), "utf8"));
const authority = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/authority.json",
    ),
    "utf8",
  ),
);
// The refined producer closure; `authority.json` keeps the original campaign.
const admission = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/admission-source-pairs.json",
    ),
    "utf8",
  ),
);
const beforeSources = new Map(
  admission.sourceFiles.map(({ path: file }) => {
    let bytes = null;
    try {
      bytes = execFileSync("git", ["show", `${authority.beforeRevision}:${file}`], {
        maxBuffer: 8 * 1024 * 1024,
        stdio: ["ignore", "pipe", "pipe"],
      });
    } catch {}
    return [file, bytes];
  }),
);
const sourcePaths = admission.sourceFiles.map((source) => source.path);
// This complete ordinary case exists only in the in-memory gate control. The
// production manifest receives no SSR case or whole-product native credit.
const ordinary = structuredClone(
  manifest.cases.find((row) => row.id === "compiler/sfc/ssr-slot-scope"),
);
const originalRoot = ordinary.inputs.root;
ordinary.id = "compiler/sfc/unit-only-ssr-control";
ordinary.inputs.root = "unit-only-ssr-control";
ordinary.reference.path = "unit-only-ssr-control/module.expected.txt";
function observe({
  missing = false,
  stale = false,
  filler = false,
  mixed = false,
  future = false,
  requestedPaths,
  rows = manifest.vaporStyleApiCases,
} = {}) {
  const previous = structuredClone(manifest),
    current = structuredClone(manifest);
  previous.vaporStyleApiCases = stale ? manifest.vaporStyleApiCases : undefined;
  current.vaporStyleApiCases = rows;
  if (missing) delete current.vaporStyleApiCases;
  if (filler) current.cases.push(ordinary);
  const paths = requestedPaths ?? [...sourcePaths];
  if (mixed) paths.push("crates/vize_atelier_sfc/src/unrelated_unit.rs");
  const result = analyzeLegacyFix({
    title: "fix(compiler): retain literal Vapor styles",
    paths,
    readFile(side, file) {
      if (file === manifestPath)
        return Buffer.from(JSON.stringify(side === "base" ? previous : current));
      if (beforeSources.has(file) && side === "base") return beforeSources.get(file);
      if (side === "base") return null;
      if (file.startsWith("tests/_fixtures/differential/compiler/unit-only-ssr-control/"))
        file = file.replace("/unit-only-ssr-control/", `/${originalRoot}/`);
      const actual = path.join(root, file);
      if (!fs.existsSync(actual)) return null;
      const bytes = fs.readFileSync(actual);
      return future && file === sourcePaths[0] ? Buffer.concat([bytes, Buffer.from("\n")]) : bytes;
    },
  });
  assert.equal(result.products.length, 1);
  return result.products[0];
}
void test("the exact actual Vapor CSR closure registers without an unrelated SSR case", () => {
  const result = observe();
  assert.equal(result.state, "new-inputs-registered", result.reason);
  assert.deepEqual(result.cases, [VAPOR_STYLE_API_CASE_ID]);
});
void test("a new ordinary SSR unit control cannot replace missing or stale CSR authority", () => {
  for (const options of [{ missing: true }, { rows: [] }, { stale: true }]) {
    const result = observe({ ...options, filler: true });
    assert.equal(result.state, "invalid-manifest-or-input");
    assert.match(result.reason, /Vapor style/);
  }
});
void test("mixed compiler paths and future source pairs retain ordinary case requirements", () => {
  assert.equal(observe({ mixed: true }).state, "new-input-missing");
  const mixed = observe({ mixed: true, filler: true });
  assert.equal(mixed.state, "new-inputs-registered", mixed.reason);
  assert.deepEqual(mixed.cases, [ordinary.id, VAPOR_STYLE_API_CASE_ID]);
  assert.equal(observe({ future: true, stale: true }).state, "new-input-missing");
  const future = observe({ future: true, stale: true, filler: true });
  assert.equal(future.state, "new-inputs-registered", future.reason);
  assert.deepEqual(future.cases, [ordinary.id]);
});
void test("the report cannot convert the source witness into native or SSR credit", () => {
  const rows = structuredClone(manifest.vaporStyleApiCases);
  rows[0].targets = ["ssr"];
  assert.equal(observe({ rows, filler: true }).state, "invalid-manifest-or-input");
  const broadened = structuredClone(manifest.vaporStyleApiCases);
  broadened[0].nativeParity = true;
  assert.equal(observe({ rows: broadened, filler: true }).state, "invalid-manifest-or-input");
});

void test("a different compiler-family request receives no Vapor source-case credit", () => {
  const result = observe({ requestedPaths: ["crates/vize_atelier_sfc/src/unrelated_unit.rs"] });
  assert.equal(result.state, "new-input-missing", result.reason);
  assert.deepEqual(result.cases, []);
});
