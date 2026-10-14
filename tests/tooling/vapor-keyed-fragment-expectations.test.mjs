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

await test("TSX reference authority keeps its archived inputs and binds the live producer input", () => {
  const authority = JSON.parse(
    readFileSync(new URL("tsx-syntax-edges-reference-authority.json", root)),
  );
  const successor = JSON.parse(
    readFileSync(new URL("tsx-syntax-edges-root-fallthrough-8625-successor-authority.json", root)),
  );
  const sha = (data) => createHash("sha256").update(data).digest("hex");
  // Archived fixtures are immutable; the live producer and snapshots are not
  // byte-pinned, so snapshot regeneration never fails this test.
  for (const [name, bytes, hash] of [
    [authority.inputPath, authority.inputBytes, authority.inputSha256],
    [
      authority.originalReferencePath,
      authority.originalReferenceBytes,
      authority.originalReferenceSha256,
    ],
    [
      authority.previousPreparedReferencePath,
      authority.previousPreparedReferenceBytes,
      authority.previousPreparedReferenceSha256,
    ],
  ]) {
    const data = readFileSync(new URL(name, root));
    assert.equal(data.length, bytes, name);
    assert.equal(sha(data), hash, name);
  }
  assert.equal(
    sha(readFileSync(new URL(successor.previousAuthority, root))),
    successor.previousAuthoritySha256,
  );
  assert.equal(
    sha(readFileSync(new URL(successor.previousProducerPath, root))),
    authority.producerSourceSha256,
  );
  assert.equal(successor.currentProducerPath, authority.producerSourcePath);
  const repository = new URL("../../", import.meta.url);
  const source = readFileSync(new URL(authority.producerSourcePath, repository), "utf8");
  assert(
    source.includes('assert_vapor_snapshot(\n        "tsx_syntax_edges_root_fallthrough_8625",'),
    "the live producer dispatches the #8625 successor case",
  );
  const start =
    source.indexOf('const TSX_SYNTAX_EDGES: &str = r#"') +
    'const TSX_SYNTAX_EDGES: &str = r#"'.length;
  const end = source.indexOf('"#;', start);
  assert.equal(source.slice(start, end), readFileSync(new URL(authority.inputPath, root), "utf8"));
});
