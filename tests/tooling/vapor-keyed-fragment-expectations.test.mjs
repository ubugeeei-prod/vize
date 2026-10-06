import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { keyedExpectations } from "./support/vapor-keyed-fragment-expectations.mjs";

const root = new URL("../_fixtures/differential/compiler/vapor-keyed-fragment/", import.meta.url);
const custody = JSON.parse(readFileSync(new URL("custody.json", root)));
const originals = new Map(
  custody.files.map((file) => {
    const bytes = readFileSync(new URL(file.path, root));
    assert.equal(bytes.length, file.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), file.sha256);
    return [file.path, bytes.toString("utf8")];
  }),
);

await test("whole current value-attribute authority keeps every other original field", () => {
  const expected = keyedExpectations(originals);
  assert.deepEqual(expected.element, JSON.parse(originals.get("element.expected-current.json")));
  for (const name of [
    "component",
    "stable",
    "nested-attrs",
    "root-attrs",
    "if-attrs",
    "nested-if-attrs",
  ]) {
    assert.deepEqual(expected[name], JSON.parse(originals.get(`${name}.expected.json`)));
  }
});

await test("value-attribute authority rejects omission, incorrect defaults and other changes", () => {
  for (const change of [
    (rows) => delete rows[0].tree[0].children[2].attributes.value,
    (rows) => {
      rows[1].tree[0].children[2].attributes.value = "typed";
    },
    (rows) => {
      rows[2].same = true;
    },
    (rows) => rows.pop(),
  ]) {
    const files = new Map(originals);
    const current = JSON.parse(files.get("element.expected-current.json"));
    change(current);
    files.set("element.expected-current.json", JSON.stringify(current));
    assert.throws(() => keyedExpectations(files));
  }
});

await test("whole TSX reference authority preserves the actual original source and old golden", () => {
  const authority = JSON.parse(
    readFileSync(new URL("tsx-syntax-edges-reference-authority.json", root)),
  );
  const repository = new URL("../../", import.meta.url);
  for (const [path, bytes, hash] of [
    [new URL(authority.inputPath, root), authority.inputBytes, authority.inputSha256],
    [
      new URL(authority.originalReferencePath, root),
      authority.originalReferenceBytes,
      authority.originalReferenceSha256,
    ],
    [
      new URL(authority.referencePath, repository),
      authority.currentReferenceBytes,
      authority.currentReferenceSha256,
    ],
  ]) {
    const data = readFileSync(path);
    assert.equal(data.length, bytes);
    assert.equal(createHash("sha256").update(data).digest("hex"), hash);
  }
  const producer = readFileSync(new URL(authority.producerSourcePath, repository));
  assert.equal(createHash("sha256").update(producer).digest("hex"), authority.producerSourceSha256);
  const source = producer.toString("utf8");
  const start =
    source.indexOf('const TSX_SYNTAX_EDGES: &str = r#"') +
    'const TSX_SYNTAX_EDGES: &str = r#"'.length;
  const end = source.indexOf('"#;', start);
  assert.equal(source.slice(start, end), readFileSync(new URL(authority.inputPath, root), "utf8"));
});
