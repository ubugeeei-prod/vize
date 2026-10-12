import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const root = fileURLToPath(new URL("../../", import.meta.url));
const source = readFileSync(
  path.join(root, "tools/benchmarks/scripts/typechecker-template-emit-projection-receipt.mjs"),
  "utf8",
);
assert.equal(
  createHash("sha256").update(source).digest("hex"),
  "fa593b2f07e71486f4bccbd5a44f4f62927f621d82c2453be2541d224858fa73",
);
const imports =
  [
    'import assert from "node:assert/strict";',
    'import { createHash } from "node:crypto";',
    'import { readFileSync, readdirSync, writeFileSync } from "node:fs";',
    'import { join } from "node:path";',
  ].join("\n") + "\n";
assert(source.startsWith(imports));
assert.equal(source.split("\nconst proofs = [];\n").length, 2);
const context = {
  assert,
  createHash,
  Buffer,
  join: path.join,
  readFileSync: (file, options) => readFileSync(path.join(root, file), options),
  process: { env: {} },
};
vm.runInNewContext(
  source.split("\nconst proofs = [];\n")[0].slice(imports.length) +
    "\nglobalThis.controls = { rewind, rewindInitializer, offset, declarations, navigationDeclaration };",
  context,
);
const { rewind, rewindInitializer, offset, declarations, navigationDeclaration } = context.controls;
const model = declarations["define-model"];
const dual = declarations["dual-scripts-emits"];
const old = "const $emit = __ctx.$emit;";
const text = `prefix\n${model}\n${navigationDeclaration}suffix`;

void test("the old initializer-only receipt leaves the exact new helper; the successor restores both", () => {
  assert.equal(Buffer.byteLength(navigationDeclaration), 1138);
  const expected = `prefix\n${old}\nsuffix`;
  const previous = rewindInitializer(text, model);
  assert.equal(previous.delta, 97);
  assert.equal(Buffer.byteLength(previous.text) - Buffer.byteLength(expected), 1138);
  assert.throws(() => assert.equal(previous.text, expected));
  assert.equal(rewind(text, model).text, expected);
  assert.equal(rewind(text, model).delta, 97);
});

void test("missing, partial, duplicate, mutated and unknown helper families refuse", () => {
  for (const damaged of [
    text.replace(navigationDeclaration, ""),
    text.replace(navigationDeclaration, navigationDeclaration.slice(0, -1)),
    text + navigationDeclaration,
    text.replace("__VizeEmptyVueNavigationSlots<C>", "__VizeEmptyVueNavigationSlots<Cx>"),
    text + "\n__VizeNavigationSlots",
  ])
    assert.throws(() => rewind(damaged, model));
  assert.throws(() => rewind(text, "unknown initializer"));
});

void test("generated endpoints compose both byte inverses and refuse inserted helper coordinates", () => {
  const edit = rewind(text, model);
  assert.equal(offset(edit.navigation.cut - 1, edit), edit.navigation.cut - 1 - 97);
  assert.equal(offset(edit.navigation.end, edit), edit.navigation.cut - 97);
  for (const value of [
    edit.navigation.cut,
    edit.navigation.cut + 1,
    edit.navigation.end - 1,
    -1,
    "not a number",
    0.5,
    Number.MAX_SAFE_INTEGER + 1,
  ])
    assert.throws(() => offset(value, edit));
  assert.throws(() => offset(edit.cut + 1, edit));
});

void test("unchanged dual and declaration-absent script paths reject unexpected helper text", () => {
  const dualText = `prefix\n${dual}\nsuffix`;
  const edit = rewind(dualText, dual);
  assert.equal(edit.text, `prefix\n${old}\nsuffix`);
  assert.equal(edit.delta, 50);
  assert.equal(edit.navigation.delta, 0);
  assert.equal(
    rewind("export const script = 1;\n", model, true).text,
    "export const script = 1;\n",
  );
  assert.throws(() => rewind(dualText + navigationDeclaration, dual));
  assert.throws(() => rewind(navigationDeclaration, model, true));
});
