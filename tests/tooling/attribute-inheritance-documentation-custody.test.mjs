import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  composeInheritanceDocument,
  createInheritanceFenceVerifier,
  decodeInheritanceBody,
  inheritanceDocumentationFixture,
  loadInheritanceDocumentation,
  validateInheritanceDocumentation,
} from "./support/attribute-inheritance-documentation-custody.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const read = (name) => fs.readFileSync(path.join(root, name));
const successor = loadInheritanceDocumentation(root);
const reseal = (row, bytes) => ({
  ...row,
  base64: bytes.toString("base64"),
  bytes: bytes.length,
  sha256: createHash("sha256").update(bytes).digest("hex"),
  gitBlob: createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
});

void test("all five complete documents preserve the finite original bodies and inverse edits", () => {
  for (const row of successor.documents) {
    const before = decodeInheritanceBody(row.before).toString();
    assert.equal(composeInheritanceDocument(before, row.path), read(row.path).toString());
    assert.throws(() => composeInheritanceDocument(before, "unknown.md"));
    assert.throws(() => composeInheritanceDocument(before + before, row.path));
    const changed = structuredClone(successor);
    changed.documents.find((item) => item.path === row.path).after = reseal(
      row.after,
      Buffer.concat([decodeInheritanceBody(row.after), Buffer.from("foreign\n")]),
    );
    assert.throws(() => validateInheritanceDocumentation(changed));
  }
});

void test("the original 317-rule fence custody has one named pair and no neighboring bypass", () => {
  const verify = createInheritanceFenceVerifier(root);
  const entries = JSON.parse(decodeInheritanceBody(successor.custody)).entries;
  for (const entry of entries) {
    for (const name of [entry.path, entry.englishPath]) {
      const fences = [
        ...read(name)
          .toString()
          .matchAll(/^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm),
      ].map((match) => match[0]);
      if (entry.id !== "script/no-duplicate-attr-inheritance") {
        assert.equal(verify(entry, name, fences), false);
        continue;
      }
      assert.equal(verify(entry, name, fences), true);
      assert.throws(() => verify(entry, name, [...fences, "```vue\nforeign\n```"]));
      assert.throws(() => verify(entry, name, fences.toReversed()));
      assert.throws(() => verify(entry, "unknown.md", fences));
      assert.throws(() => verify({ ...entry, wholeCodeSha256: "foreign" }, name, fences));
    }
  }
});

void test("every live frozen fixture and whole document rejects physical mutation", () => {
  const names = [
    inheritanceDocumentationFixture,
    successor.custody.path,
    successor.publicImportPrimary.path,
    ...successor.documents.map((row) => row.path),
  ];
  for (const name of names) {
    const mutated = (filename) =>
      filename === name
        ? Buffer.concat([read(filename), Buffer.from("foreign\n")])
        : read(filename);
    assert.throws(() => loadInheritanceDocumentation(root, mutated), name);
    assert.throws(
      () =>
        loadInheritanceDocumentation(root, (filename) => {
          if (filename === name) throw new Error("missing complete body");
          return read(filename);
        }),
      name,
    );
  }
});

void test("closed source composition preserves all 323 rows and both complete historical frames", () => {
  assert.equal(successor.composition.changed.length, 11);
  const mutate = (change) => {
    const value = structuredClone(successor);
    change(value);
    assert.throws(() => validateInheritanceDocumentation(value));
  };
  mutate((value) => value.sourceAuthority.pop());
  mutate((value) => value.sourceAuthority.reverse());
  mutate((value) => {
    value.sourceAuthority[0].base64 += "foreign";
  });
  mutate((value) => {
    value.composition.inventoryEdits[0].offset++;
  });
  mutate((value) => value.composition.changed.pop());
  mutate((value) => value.composition.changed.reverse());
  mutate((value) => value.composition.pins.pop());
  mutate((value) => {
    const row = value.composition.pins[0];
    row.after = reseal(
      row.after,
      Buffer.concat([decodeInheritanceBody(row.after), Buffer.from("foreign\n")]),
    );
  });
  mutate((value) => value.documents.pop());
  mutate((value) => value.documents.push(value.documents[0]));
  mutate((value) => (value.credit.native = true));
  mutate((value) => {
    const manifest = JSON.parse(decodeInheritanceBody(value.composition.after));
    manifest.current.pop();
    value.composition.after = reseal(
      value.composition.after,
      Buffer.from(JSON.stringify(manifest)),
    );
  });
  mutate((value) => {
    const manifest = JSON.parse(decodeInheritanceBody(value.composition.after));
    manifest.literalFrames.reverse();
    value.composition.after = reseal(
      value.composition.after,
      Buffer.from(JSON.stringify(manifest)),
    );
  });
  mutate((value) => {
    const row = value.composition.changed[0];
    row.after = reseal(
      row.after,
      Buffer.concat([decodeInheritanceBody(row.after), Buffer.from("foreign\n")]),
    );
  });
});

void test("historical source checkpoints grant no live checkpoint gate to future descendants", () => {
  const historical = new Set([
    ...successor.sourceAuthority.map((row) => row.path),
    successor.composition.manifest,
    ...successor.composition.changed.map((row) => row.path),
    ...successor.composition.pins.map((row) => row.path),
  ]);
  loadInheritanceDocumentation(root, (name) => {
    assert(!historical.has(name), "the existing physical family owns live source qualification");
    return read(name);
  });
});
