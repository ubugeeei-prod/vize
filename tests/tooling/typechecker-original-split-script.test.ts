import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const directory = path.join(root, "tests/_fixtures/differential/typechecker");
const base = path.join(directory, "split-script-original");

test("split-script history preserves the entire original project, helpers and diagnostic tuple", () => {
  const proof = JSON.parse(fs.readFileSync(path.join(base, "input-provenance.json"), "utf8"));
  const pack = JSON.parse(fs.readFileSync(path.join(base, "cases.json"), "utf8"));
  const original = fs.readFileSync(path.join(base, proof.originalTestSource.carrier));
  const excerpt = proof.projectHelperSource.excerpt;
  const helper = fs.readFileSync(path.join(base, excerpt.carrier));
  const patch = fs.readFileSync(path.join(base, proof.originalChange.carrier));
  assert.equal(proof.regressionCommit, "b3c2933be5afbdb5f2df07da093f40bc7ce72187");
  assert.equal(proof.parent, "8f1953c0637060efe9d1103f9426000a39238054");
  assert.equal(
    sha256(original),
    "c26b12f607fa918d6c2944214d7dd6d10504ae88e347d3f537ae34030dc28790",
  );
  assert.equal(sha256(helper), "b3beb7d333a17aa20c66d8be50cb26b1414d5b4fc3810f3250c56e3b1d98baab");
  assert.equal(sha256(patch), "5073b6dcc20b35596337dc455138efd842f1c9ca8dc25bebf4fb64aa29e53215");
  assert.equal(sha256(original), proof.originalTestSource.sha256);
  assert.equal(sha256(helper), excerpt.sha256);
  assert.equal(sha256(patch), proof.originalChange.sha256);
  assert.equal(helper.length, excerpt.utf8End - excerpt.utf8Start);
  assert(helper.toString().includes("fn create_project_case("));
  assert(helper.toString().includes("link_workspace_node_modules(&project_root)"));
  assert(helper.toString().includes("snapshot.sort();"));
  const diagnosticHelper = helper
    .toString()
    .split("fn snapshot_project_diagnostics(")[1]
    .split("fn link_workspace_node_modules(")[0];
  assert(!diagnosticHelper.includes(".filter("));
  assert(original.toString().includes(`fn ${proof.originalTest}()`));
  const loaded = loadTypecheckerManifest(path.join(directory, "manifest.json"));
  const planned = loaded.cases.filter((fixture) => fixture.pack === "split-script-original");
  assert.equal(planned.length, 1);
  assert.equal(planned[0].sourceRevision, proof.regressionCommit);
  assert.equal(pack.sourceRevision, proof.regressionCommit);
  assert.equal(pack.historicalIssue, 3783);
  assert.equal(planned[0].adapters.native, null);
  assert.equal(pack.diagnosticContract.native, "unsupported");
  assert.deepEqual(pack.diagnosticContract.missingFields, [
    "end",
    "relatedInformation",
    "raw backend diagnostics",
  ]);
  for (const input of proof.inputs) {
    const bytes = fs.readFileSync(path.join(base, input.source));
    assert.equal(sha256(bytes), input.sha256);
    const source = input.file === "tsconfig.json" ? helper : original;
    const offset =
      input.file === "tsconfig.json"
        ? proof.configLiteralUtf8Start - excerpt.utf8Start
        : input.originalLiteralUtf8Start;
    assert.deepEqual(bytes, source.subarray(offset, offset + bytes.length));
  }
  const source = fs.readFileSync(path.join(base, "original-project/src/App.vue.txt"), "utf8");
  assert(source.includes("export type SearchQuery = { value: string };"));
  assert(
    source.includes(
      "function refreshGridItems() {\n\tgridItems.value = customEmojis.value.map(it => ({\n\t\tid: it.id,\n\t}));\n}",
    ),
  );
  assert.equal(source.split("<script").length - 1, 2);
  assert.equal(source.split("\n")[9].slice(42, 44), "it");
  const tuple =
    /vec!\[\(\s*String::from\("([^"]+)"\),\s*Some\((\d+)\),\s*String::from\("(\d+):(\d+):error ([^"]+)"\),\s*\)\]/.exec(
      original.toString(),
    );
  assert(tuple);
  assert.deepEqual(planned[0].diagnostics, [
    {
      file: tuple[1],
      line: Number(tuple[3]),
      column: Number(tuple[4]),
      severity: 1,
      code: Number(tuple[2]),
      message: tuple[5],
    },
  ]);
  assert.deepEqual(planned[0].diagnostics, pack.cases[0].diagnostics);
  const config = JSON.parse(
    fs.readFileSync(path.join(base, "original-project/tsconfig.json.txt"), "utf8"),
  );
  assert.deepEqual(config.compilerOptions, {
    strict: true,
    target: "ES2022",
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
  });
  assert(!Object.hasOwn(config, "vueCompilerOptions"));
  assert.deepEqual(config.include, ["src/**/*"]);
  assert.equal(proof.execution, "pending fresh required T1 source-built Actions");
});
