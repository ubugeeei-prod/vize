import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expressionWidthReference } from "../differential/formatter-expression-width-reference.mjs";

const root = fileURLToPath(new URL("../..", import.meta.url));
const authority = "tests/_fixtures/differential/formatter-regressions/expression-print-width-7876";
const manifest = "tests/_fixtures/differential/formatter/manifest.json";
const original = fs.readFileSync(
  path.join(
    root,
    "tests/_fixtures/differential/formatter-regressions/directive-print-width-7876/original-deep-prebroken.input",
  ),
);
const fixture = () => ({
  id: "formatter/sfc/directive-prefix-print-width",
  input: Buffer.from(original),
  expected: Buffer.from(original),
  argv: ["fmt", "--no-config", "--write", "App.vue"],
  config: [],
});

void test("the current reference retains the exact original manifest and mismatch", () => {
  const qualified = expressionWidthReference(root, fixture());
  assert.equal(qualified.currentReference.issue, 7876);
  assert.equal(
    qualified.currentReference.inputSha256,
    "851c54f3ec30d67d54dc5b6369b06303f3222583da08d873c1144cf3a1a8cb52",
  );
  assert.equal(
    qualified.currentReference.historicalExpectedSha256,
    qualified.currentReference.inputSha256,
  );
  assert.equal(
    qualified.currentReference.currentExpectedSha256,
    "b2fb731eb8b2376c016442868a4f632d62cc617c53d50e7d4ffa40a073e28b16",
  );
  assert(!qualified.currentExpected.equals(original));
  assert.deepEqual(expressionWidthReference(root, { ...fixture(), id: "formatter/sfc/other" }), {});
});

void test("changed original bytes and invocation cannot claim the current reference", () => {
  for (const changed of [
    { input: Buffer.concat([original, Buffer.from("\n")]) },
    { expected: Buffer.concat([original, Buffer.from("\n")]) },
    { argv: ["fmt", "--write", "App.vue"] },
    { config: [{ path: "vize.config.json" }] },
  ])
    assert.throws(() => expressionWidthReference(root, { ...fixture(), ...changed }));
});

function carrier(t) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-expression-reference-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  for (const relative of [
    manifest,
    `${authority}/corpus.json`,
    `${authority}/original-deep-lf.expected`,
  ]) {
    const file = path.join(directory, relative);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.copyFileSync(path.join(root, relative), file);
  }
  return directory;
}

void test("whole manifest, corpus and expected-byte mutations are rejected", (t) => {
  for (const relative of [
    manifest,
    `${authority}/corpus.json`,
    `${authority}/original-deep-lf.expected`,
  ]) {
    const directory = carrier(t);
    fs.appendFileSync(path.join(directory, relative), "\n");
    assert.throws(() => expressionWidthReference(directory, fixture()));
  }
});

void test("an otherwise correct reference outside the source root is rejected", (t) => {
  const directory = carrier(t);
  const reference = path.join(directory, authority, "original-deep-lf.expected");
  fs.unlinkSync(reference);
  fs.symlinkSync(path.join(root, authority, "original-deep-lf.expected"), reference);
  assert.throws(() => expressionWidthReference(directory, fixture()));
});
