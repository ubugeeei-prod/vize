import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import test from "node:test";
import {
  assertExactCssTree,
  composeForegroundPreparationCheckout,
  invertForegroundPreparationSource,
  originalTypedSourceSha256,
  typedSourcePath,
  type SourceTreeEntry,
} from "../../editors/vscode/test/css-hover-foreground-source.ts";
import {
  foregroundTypedSourceSha256,
  initialProjectTypedEntry,
  initialProjectTypedSourceSha256,
  invertInitialProjectSource,
} from "../../editors/vscode/test/css-hover-initial-project-source.ts";

const load = (name: string): Buffer =>
  fs.readFileSync(new URL(`./fixtures/css-hover-initial-project/${name}`, import.meta.url));
const source = load("initial-project.rs.txt");
const baseSource = load("foreground-before.rs.txt");
const before: SourceTreeEntry = {
  path: typedSourcePath,
  mode: "100644",
  type: "blob",
  oid: "d378281fbcda6dda2f550e21fa3ccde3905d8a7d",
};
const foreground = { ...before, oid: "a4b1e81960029f488921f917e46589f1d7655da9" };
const current = initialProjectTypedEntry;
const other = { ...before, path: "crates/vize_maestro/src/ide/style.rs", oid: "1".repeat(40) };
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
const blob = (bytes: Uint8Array): string =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
const refuse = (action: () => unknown): void => assert.throws(action, assert.AssertionError);

await test("complete new source restores both exact foreground and legacy bodies", () => {
  assert.equal(hash(source), initialProjectTypedSourceSha256);
  assert.equal(blob(source), current.oid);
  const restored = invertInitialProjectSource(source);
  assert.deepEqual(restored, baseSource);
  assert.equal(hash(restored), foregroundTypedSourceSha256);
  assert.equal(blob(restored), foreground.oid);
  const legacy = invertForegroundPreparationSource(restored);
  assert.equal(hash(legacy), originalTypedSourceSha256);
  assert.equal(blob(legacy), before.oid);
  refuse(() => invertForegroundPreparationSource(source));
  refuse(() => invertInitialProjectSource(baseSource));
});

await test("current composition binds the exact row, reader, both source layers and whole tree", () => {
  const expected = [before, other];
  const actual = [current, other];
  const reads: string[] = [];
  const result = composeForegroundPreparationCheckout(expected, actual, (name) => {
    reads.push(name);
    return source;
  });
  assert.deepEqual(reads, [typedSourcePath]);
  assert.deepEqual(expected, [before, other]);
  assert.deepEqual(result.entries, actual);
  assert.deepEqual(result.layer, {
    name: "initial-canonical-project-bootstrap",
    sourceSha256: initialProjectTypedSourceSha256,
    inverseSha256: originalTypedSourceSha256,
    intermediateSha256: foregroundTypedSourceSha256,
    changes: [{ path: typedSourcePath, before, after: current }],
    foregroundPreparation: {
      name: "same-document-foreground-preparation-11347",
      reviewedRevision: "1248e5a903c6de56f2e4630650a048744d135edf",
      sourceSha256: foregroundTypedSourceSha256,
      inverseSha256: originalTypedSourceSha256,
      changes: [{ path: typedSourcePath, before, after: foreground }],
    },
  });
  assertExactCssTree(actual, result.entries);
});

await test("old profiles and foreground receipts keep their literal old behavior", () => {
  for (const expected of [[before, other], [other]]) {
    const result = composeForegroundPreparationCheckout(expected, expected, () => {
      throw new Error("old profile must not read source");
    });
    assert.equal(result.entries, expected);
    assert.equal(result.layer, null);
  }
  const result = composeForegroundPreparationCheckout([before], [foreground], () => baseSource);
  assert.equal(result.layer?.name, "same-document-foreground-preparation-11347");
  assert.deepEqual(result.entries, [foreground]);
  refuse(() => composeForegroundPreparationCheckout([before], [foreground], () => source));
  refuse(() => composeForegroundPreparationCheckout([before], [current], () => baseSource));
});

await test("CSS static JSX preparation logical URI and native block mutations remain refusals", () => {
  const text = source.toString("utf8");
  const nativeBlock = `            let stamp = self.state.corsa_request_stamp();
            let (corsa_bridge, initial) = self
                .state
                .get_corsa_bridge_for_canonical_source(ctx.uri, &ctx.content)
                .await;
            ctx.attach_initial_canonical_document(corsa_bridge.as_ref(), initial, stamp);
`;
  const marker = "        let mut ctx = ctx.with_foreground_preparation(preparation);\n";
  const mutants = [
    text.replace("if ctx.is_in_style()", "if !ctx.is_in_style()"),
    text.replace("return Ok(CompletionService::complete(&ctx));", "return Ok(None);"),
    text.replace("complete_static_object_member", "complete_unreviewed_member"),
    text.replace("jsx_typecheck_enabled()", "unreviewed_jsx_enabled()"),
    text.replace("ctx.uri, &ctx.content", "uri, &ctx.content"),
    text.replace("ctx.uri, &ctx.content", "ctx.uri, &self.unreviewed_content"),
    text.replace("attach_initial_canonical_document", "attach_unreviewed_document"),
    text.replace("initial, stamp", "initial, self.state.corsa_request_stamp()"),
    text.replace(marker, ""),
    text.replace(marker, marker + marker),
    text.replace(marker, "") + marker,
    text.replace(nativeBlock, ""),
    text.replace(nativeBlock, nativeBlock + nativeBlock),
    text.replace(nativeBlock, "") + nativeBlock,
    text.replace("get_corsa_bridge_for_canonical_source", "get_corsa_bridge"),
    text.replace("HoverService::hover_with_corsa", "HoverService::hover"),
    text.replace("RenameService::rename_with_corsa", "RenameService::rename"),
    text + "// extra Rust change\n",
    text.replaceAll("\n", "\r\n"),
  ];
  for (const mutant of mutants) {
    assert.notEqual(mutant, text, "mutation must actually change source");
    refuse(() => invertInitialProjectSource(Buffer.from(mutant)));
    refuse(() =>
      composeForegroundPreparationCheckout([before], [current], () => Buffer.from(mutant)),
    );
  }
});

await test("absent truncated non-byte and misspelled source results cannot authenticate a row", () => {
  const misspelled = { soruce: source } as Record<string, unknown>;
  for (const bytes of [
    undefined,
    null,
    source.toString("utf8"),
    Buffer.alloc(0),
    source.subarray(0, -1),
    Buffer.concat([Buffer.from([0xef, 0xbb, 0xbf]), source]),
    misspelled["source"],
  ]) {
    refuse(() => invertInitialProjectSource(bytes));
    refuse(() => composeForegroundPreparationCheckout([before], [current], () => bytes));
  }
  refuse(() =>
    composeForegroundPreparationCheckout(
      [before],
      [current],
      undefined as unknown as (name: string) => unknown,
    ),
  );
});

await test("unknown blob mode type fields and original preimages remain refusals", () => {
  for (const row of [
    { ...current, oid: "2".repeat(40) },
    { ...current, mode: "100755" },
    { ...current, type: "tree" },
    { ...current, unreviewed: true },
  ])
    refuse(() => composeForegroundPreparationCheckout([before], [row], () => source));
  for (const row of [
    foreground,
    current,
    { ...before, oid: "3".repeat(40) },
    { ...before, mode: "100755" },
    { ...before, type: "tree" },
    { ...before, unreviewed: true },
  ])
    refuse(() => composeForegroundPreparationCheckout([row], [current], () => source));
  for (const [expected, actual] of [
    [[before, before], [current]],
    [[before], [current, current]],
  ] as const)
    refuse(() => composeForegroundPreparationCheckout(expected, actual, () => source));
});

await test("the entire tree still rejects missing moved extra and unrelated Rust changes", () => {
  const expected = [before, other];
  for (const actual of [
    [other],
    [{ ...current, path: `${typedSourcePath}.moved` }, other],
    [current],
    [current, other, { ...other, path: `${other.path}.extra` }],
    [current, { ...other, oid: "4".repeat(40) }],
  ]) {
    const composed = composeForegroundPreparationCheckout(expected, actual, () => source);
    refuse(() => assertExactCssTree(actual, composed.entries));
  }
});
