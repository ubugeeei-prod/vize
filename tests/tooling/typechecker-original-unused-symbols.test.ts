import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const directory = path.join(root, "tests/_fixtures/differential/typechecker");
const name = "authored-unused-symbols";
const base = path.join(directory, name);
const manifest = path.join(directory, "manifest.json");

test("unused-symbol history retains complete original function, helpers, source and absent dependencies", () => {
  const loaded = loadTypecheckerManifest(manifest);
  const proof = JSON.parse(fs.readFileSync(path.join(base, "input-provenance.json"), "utf8"));
  const pack = JSON.parse(fs.readFileSync(path.join(base, "cases.json"), "utf8"));
  const original = fs.readFileSync(path.join(base, proof.originalTestSource.carrier));
  const helper = fs.readFileSync(path.join(base, proof.projectHelperSource.carrier));
  for (const item of [proof.originalTestSource, proof.projectHelperSource, proof.originalChange]) {
    const bytes = fs.readFileSync(path.join(base, item.carrier));
    assert.equal(sha256(bytes), item.sha256);
    if (item.utf8Start !== undefined) assert.equal(bytes.length, item.utf8End - item.utf8Start);
  }
  assert(original.toString().startsWith(`#[test]\nfn ${proof.originalTest}()`));
  assert(original.toString().endsWith("    let _ = std::fs::remove_dir_all(&project_root);\n}\n"));
  assert(original.toString().includes("create_project_case_without_node_modules("));
  assert(original.toString().includes(".filter(|(file, code, _)"));
  assert(helper.toString().includes("fn snapshot_project_diagnostics("));
  assert(helper.toString().includes("snapshot.sort();"));
  assert(!helper.toString().includes("link_workspace_node_modules"));
  for (const input of proof.inputs) {
    const bytes = fs.readFileSync(path.join(base, input.source));
    assert.equal(sha256(bytes), input.sha256);
    const offset = input.originalLiteralUtf8Start - proof.originalTestSource.utf8Start;
    assert.deepEqual(bytes, original.subarray(offset, offset + bytes.length));
  }
  assert.equal(pack.regressionCommit, proof.originalSource.revision);
  assert.equal(pack.historicalIssue, 1271);
  assert.deepEqual(pack.projectOptions, { vuePackage: "absent" });
  const planned = loaded.cases.filter((item) => item.pack === name);
  assert.equal(planned.length, 6);
  for (const item of planned) assert.deepEqual(item.projectOptions, pack.projectOptions);
  assert.deepEqual(
    planned.map((item) => item.diagnostics.length),
    [1, 1, 0, 1, 0, 0],
  );
  assert.equal(proof.execution, "pending fresh required T1 source-built Actions");
});

test("linking Vue cannot replace the original missing-package contract even after rehashing", (t) => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typechecker-unused-history-"));
  t.after(() => fs.rmSync(temp, { recursive: true }));
  fs.cpSync(directory, temp, { recursive: true });
  const packPath = path.join(temp, name, "cases.json");
  const originalPack = JSON.parse(fs.readFileSync(packPath, "utf8"));
  for (const option of [undefined, { vuePackage: "linked" }]) {
    const changed = structuredClone(originalPack);
    changed.projectOptions = option;
    fs.writeFileSync(packPath, `${JSON.stringify(changed, null, 2)}\n`);
    const changedManifest = JSON.parse(fs.readFileSync(manifest, "utf8"));
    for (const item of changedManifest.cases.filter((row: { pack: string }) => row.pack === name)) {
      item.projectOptions = option;
      item.packReference.sha256 = sha256(fs.readFileSync(packPath));
    }
    fs.writeFileSync(
      path.join(temp, "manifest.json"),
      `${JSON.stringify(changedManifest, null, 2)}\n`,
    );
    assert.throws(
      () => loadTypecheckerManifest(path.join(temp, "manifest.json")),
      /absent Vue package/,
    );
  }
});
