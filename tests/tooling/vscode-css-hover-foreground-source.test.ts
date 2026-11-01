import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import test from "node:test";
import {
  type SourceTreeEntry,
  composeForegroundPreparationCheckout,
  invertForegroundPreparationSource,
  originalTypedSourceSha256,
  typedSourcePath,
} from "../../editors/vscode/test/css-hover-foreground-source.ts";
import { assertExactCssTree } from "../../editors/vscode/test/css-hover-source-custody.mjs";

const source = fs.readFileSync(new URL(`../../${typedSourcePath}`, import.meta.url));
const row = (
  oid: string,
  extra: Partial<SourceTreeEntry> & { unreviewed?: boolean } = {},
): SourceTreeEntry & { unreviewed?: boolean } => ({
  path: typedSourcePath,
  mode: "100644",
  type: "blob",
  oid,
  ...extra,
});
const before = row("d378281fbcda6dda2f550e21fa3ccde3905d8a7d");
const after = row("a4b1e81960029f488921f917e46589f1d7655da9");
const other = { ...before, path: "crates/vize_maestro/src/ide/style.rs", oid: "1".repeat(40) };
const sha256 = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
const blob = (bytes: Uint8Array): string =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");

await test("complete reviewed typed source reverses to the whole original blob and CSS functions", () => {
  const original = invertForegroundPreparationSource(source);
  assert.equal(sha256(original), originalTypedSourceSha256);
  assert.equal(blob(original), before.oid);
  const currentText = source.toString("utf8");
  const originalText = original.toString("utf8");
  // The complete prefix includes both local CSS/static dispatch functions.
  assert.equal(
    currentText.slice(0, currentText.indexOf("    async fn hover_request_with_preparation(")),
    originalText.slice(0, originalText.indexOf("    pub(super) async fn hover_request(")),
  );
  const expected = [before, other];
  const actual = [after, other];
  const names: string[] = [];
  const composed = composeForegroundPreparationCheckout(expected, actual, (name) => {
    names.push(name);
    return source;
  });
  assert.deepEqual(names, [typedSourcePath]);
  assert.deepEqual(composed.entries, actual);
  assert.deepEqual(expected, [before, other]);
  assert.deepEqual(composed.layer, {
    name: "same-document-foreground-preparation-11347",
    reviewedRevision: "1248e5a903c6de56f2e4630650a048744d135edf",
    sourceSha256: "752179a706971407290c7144720bbcbc63261031bf6b5378bdb8cc9427b601ac",
    inverseSha256: originalTypedSourceSha256,
    changes: [{ path: typedSourcePath, before, after }],
  });
  assertExactCssTree(actual, composed.entries);
});

await test("the old profile is unchanged and never reads a new source body", () => {
  for (const expected of [[before, other], [other]]) {
    const result = composeForegroundPreparationCheckout(expected, expected, () => {
      throw new Error("old profile must not read source");
    });
    assert.equal(result.entries, expected);
    assert.deepEqual(result, { entries: expected, layer: null });
    assertExactCssTree(expected, result.entries);
  }
});

await test("missing duplicate moved and changed readiness or CSS bytes cannot pass the inverse", () => {
  const text = source.toString("utf8");
  const marker = "        let ctx = ctx.with_foreground_preparation(preparation);\n";
  const mutants = [
    text.replace(marker, ""),
    text.replace(marker, marker + marker),
    text.replace(marker, "") + marker,
    text.replace("hover_request_with_preparation", "hover_unreviewed"),
    text.replace("if ctx.is_in_style()", "if !ctx.is_in_style()"),
    text.replace("return Ok(Some(response));", "return Ok(None);"),
    text + "// unreviewed code\n",
    text.replaceAll("\n", "\r\n"),
  ];
  for (const mutant of mutants) {
    assert.notEqual(mutant, text);
    assert.throws(() => invertForegroundPreparationSource(Buffer.from(mutant)));
  }
  for (const bytes of [Buffer.alloc(0), source.subarray(0, -1), text, null])
    assert.throws(() => invertForegroundPreparationSource(bytes));
});

await test("unknown blob mode type path or row fields remain refusals", () => {
  for (const current of [
    row("2".repeat(40)),
    row(after.oid, { mode: "100755" }),
    row(after.oid, { type: "tree" }),
    row(after.oid, { unreviewed: true }),
  ])
    assert.throws(() => composeForegroundPreparationCheckout([before], [current], () => source));
  for (const expected of [row("3".repeat(40)), row(before.oid, { mode: "100755" }), after])
    assert.throws(() => composeForegroundPreparationCheckout([expected], [after], () => source));
  for (const [expected, actual] of [
    [[before, before], [after]],
    [[before], [after, after]],
  ] as const)
    assert.throws(() => composeForegroundPreparationCheckout(expected, actual, () => source));
});

await test("full actual trees retain missing renamed extra and foreign unrelated rows", () => {
  for (const actual of [
    [],
    [{ ...after, path: `${typedSourcePath}.moved` }],
    [after, other],
    [after, { ...other, oid: "4".repeat(40) }],
  ]) {
    const composed = composeForegroundPreparationCheckout([before], actual, () => source);
    assert.throws(() => assertExactCssTree(actual, composed.entries), /reviewed blob\/mode/);
  }
  assert.throws(() => assertExactCssTree([after], [before]), /reviewed blob\/mode/);
});

await test("a row identity cannot substitute foreign or absent full source bytes", () => {
  for (const bytes of [Buffer.from("foreign"), Buffer.alloc(0), undefined])
    assert.throws(() => composeForegroundPreparationCheckout([before], [after], () => bytes));
});
