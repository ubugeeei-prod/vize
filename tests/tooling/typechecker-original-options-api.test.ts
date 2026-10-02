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
const name = "options-api-any-instance";
const base = path.join(directory, name);
const manifest = path.join(directory, "manifest.json");

test("Options API history retains both original projects, exact tuples and explicit option", () => {
  const loaded = loadTypecheckerManifest(manifest);
  const proof = JSON.parse(fs.readFileSync(path.join(base, "input-provenance.json"), "utf8"));
  const pack = JSON.parse(fs.readFileSync(path.join(base, "cases.json"), "utf8"));
  const original = fs.readFileSync(path.join(base, proof.originalTestSource.carrier));
  const excerpt = proof.projectHelperSource.excerpt;
  const helper = fs.readFileSync(path.join(base, excerpt.carrier));
  assert.equal(sha256(original), proof.originalTestSource.sha256);
  assert.equal(sha256(helper), excerpt.sha256);
  assert.equal(helper.length, excerpt.utf8End - excerpt.utf8Start);
  assert.equal(pack.sourceRevision, proof.regressionCommit);
  assert.equal(pack.regressionCommit, proof.regressionCommit);
  assert.equal(pack.historicalIssue, 6680);
  assert.deepEqual(pack.checkerOptions, { optionsApi: true });
  assert.deepEqual(pack.checkerOptions, proof.originalCheckerOptions);
  const text = original.toString("utf8");
  for (const method of proof.originalTests) {
    const start = text.indexOf(`fn ${method}()`);
    assert(start > 0);
    const body = text.slice(start, text.indexOf("\n}", start));
    assert(body.indexOf("checker.enable_options_api();") > 0);
    assert(body.indexOf("checker.enable_options_api();") < body.indexOf("checker.scan_project()"));
    assert(!body.includes(".filter("));
  }
  for (const input of proof.inputs) {
    const bytes = fs.readFileSync(path.join(base, input.source));
    assert.equal(sha256(bytes), input.sha256);
    assert.deepEqual(
      bytes,
      original.subarray(
        input.originalLiteralUtf8Start,
        input.originalLiteralUtf8Start + bytes.length,
      ),
    );
  }
  const config = fs.readFileSync(path.join(base, proof.configuration.source));
  assert.equal(sha256(config), proof.configuration.sha256);
  const offset = proof.configLiteralUtf8Start - excerpt.utf8Start;
  assert.deepEqual(config, helper.subarray(offset, offset + config.length));
  const props = /let props_type = r#"([\s\S]*?)"#;/.exec(text)?.[1];
  assert(props);
  assert(
    text.includes("cstr!(\"13:18:error Property 'nope' does not exist on type '{props_type}'.\")"),
  );
  assert(
    text.includes("cstr!(\"14:23:error Property 'toUpperCase' does not exist on type 'number'.\")"),
  );
  assert(text.includes("Vec::<(String, Option<u32>, String)>::new()"));
  assert.deepEqual(
    pack.cases.map((item: { diagnostics: unknown }) => item.diagnostics),
    [
      [
        {
          file: "src/OptionsInvalid.vue",
          line: 13,
          column: 18,
          severity: 1,
          code: 2339,
          message: `Property 'nope' does not exist on type '${props}'.`,
        },
        {
          file: "src/OptionsInvalid.vue",
          line: 14,
          column: 23,
          severity: 1,
          code: 2339,
          message: "Property 'toUpperCase' does not exist on type 'number'.",
        },
      ],
      [],
    ],
  );
  assert.deepEqual(
    loaded.cases.filter((item) => item.pack === name).map((item) => item.checkerOptions),
    [{ optionsApi: true }, { optionsApi: true }],
  );
  assert.equal(proof.execution, "pending fresh required T1 source-built Actions");
  assert.equal(proof.native, "unsupported");
});

test("changed or omitted original option fails even with refreshed pack and manifest hashes", (t) => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typechecker-options-api-"));
  t.after(() => fs.rmSync(temp, { recursive: true }));
  fs.cpSync(directory, temp, { recursive: true });
  const packPath = path.join(temp, name, "cases.json");
  const originalPack = JSON.parse(fs.readFileSync(packPath, "utf8"));
  for (const option of [undefined, { optionsApi: false }]) {
    const changed = structuredClone(originalPack);
    changed.checkerOptions = option;
    fs.writeFileSync(packPath, `${JSON.stringify(changed, null, 2)}\n`);
    const changedManifest = JSON.parse(fs.readFileSync(manifest, "utf8"));
    for (const item of changedManifest.cases.filter((row: { pack: string }) => row.pack === name)) {
      item.checkerOptions = option;
      item.packReference.sha256 = sha256(fs.readFileSync(packPath));
    }
    fs.writeFileSync(
      path.join(temp, "manifest.json"),
      `${JSON.stringify(changedManifest, null, 2)}\n`,
    );
    assert.throws(
      () => loadTypecheckerManifest(path.join(temp, "manifest.json")),
      /original explicit checker options/,
    );
  }
});
