import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  assertEquivalentCompilerModule,
  currentCompilerModuleReference,
} from "../differential/compiler-module-reference.ts";

const root = fileURLToPath(new URL("../_fixtures/differential/compiler/", import.meta.url));
const manifest = JSON.parse(fs.readFileSync(path.join(root, "manifest.json"), "utf8")) as {
  cases: { id: string; inputs: unknown; reference: { path: string; sha256: string } }[];
};
const ids = [
  "compiler/project/imported-props-path-alias-11794",
  "compiler/sfc/nested-dynamic-vfor-11728",
  "compiler/sfc/recovered-attribute-boundary-11731",
];

test("three raw emitter references authenticate every input and complete archived module", () => {
  for (const id of ids) {
    const fixture = manifest.cases.find((entry) => entry.id === id);
    assert.ok(fixture);
    const archived = fs.readFileSync(path.join(root, fixture.reference.path));
    const current = currentCompilerModuleReference(root, fixture, archived);
    assert.notDeepEqual(current, archived);
    assertEquivalentCompilerModule(archived.toString("utf8"), current.toString("utf8"));
    for (const [input, before] of [
      [{ ...fixture, inputs: {} }, archived],
      [{ ...fixture, reference: { ...fixture.reference, sha256: "changed" } }, archived],
      [fixture, Buffer.from("changed")],
    ] as const)
      assert.throws(() => currentCompilerModuleReference(root, input, before), {
        code: "ERR_ASSERTION",
      });
  }
  const untouched = Buffer.from("unchanged");
  assert.equal(
    currentCompilerModuleReference(
      root,
      { id: "unregistered", inputs: {}, reference: { sha256: "unused" } },
      untouched,
    ),
    untouched,
  );
});

test("independent AST checks reject semantic changes while retaining property identity", () => {
  assertEquivalentCompilerModule("const item = { value };", "const item = { value: value };\n");
  for (const [before, after] of [
    ["const item = { required: false };", "const item = { required: true };"],
    ["const item = $props.rows;", "const item = $props.rowz;"],
    ["const item = { __proto__ };", "const item = { __proto__: __proto__ };"],
    ["const item = r === 0 ? 'th' : 'td';", "const item = r !== 0 ? 'th' : 'td';"],
    ["const item = merge(a, b);", "const item = merge(b, a);"],
    ["const item = 1;", "let item = 1;"],
  ])
    assert.throws(() => assertEquivalentCompilerModule(before, after), { code: "ERR_ASSERTION" });
});

test("a synthetic manifest with the same case ID keeps its complete independent reference", () => {
  const archived = Buffer.from("export const fixture = true;\n");
  for (const id of ids) {
    const synthetic = {
      id,
      inputs: { root: "independent-protocol-fixture", files: [] },
      reference: { sha256: createHash("sha256").update(archived).digest("hex") },
    };
    assert.equal(currentCompilerModuleReference(root, synthetic, archived), archived);
  }
});
