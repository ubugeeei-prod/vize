import assert from "node:assert/strict";
import { test } from "node:test";
import { applyWorkspaceEdit, utf16Offset, utf16Position, WorkspaceEditFailure } from "./apply.ts";

const uri = "file:///owned/E.vue";
const document = { uri, text: "😀 tone\r\n隠 hidden\nlast", version: 1 };
const range = (line: number, start: number, end: number) => ({
  start: { line, character: start },
  end: { line, character: end },
});
const replacement = (line: number, start: number, end: number, newText: string) => ({
  range: range(line, start, end),
  newText,
});

test("complete UTF16 replacements preserve unrelated Unicode and original line endings", () => {
  const edits = [replacement(1, 2, 8, "visible"), replacement(0, 3, 7, "palette")];
  const packet = { changes: { [uri]: edits } };
  const original = structuredClone({ packet, document });
  assert.deepEqual(applyWorkspaceEdit(packet, [document]), [
    { ...document, text: "😀 palette\r\n隠 visible\nlast" },
  ]);
  assert.deepEqual({ packet, document }, original);
  assert.deepEqual(applyWorkspaceEdit({ changes: { [uri]: [...edits].reverse() } }, [document]), [
    { ...document, text: "😀 palette\r\n隠 visible\nlast" },
  ]);
});

test("all declaration, default and template edits apply to an independent whole document", () => {
  const source =
    "type Props = { tone?: string };\nconst props = withDefaults(defineProps<Props>(), { tone: 'red' });\n<template>{{ tone }}</template>\n";
  const expected =
    "type Props = { palette?: string };\nconst props = withDefaults(defineProps<Props>(), { palette: 'red' });\n<template>{{ palette }}</template>\n";
  const edit = {
    documentChanges: [
      {
        textDocument: { uri, version: 1 },
        edits: [
          replacement(2, 13, 17, "palette"),
          replacement(0, 15, 19, "palette"),
          replacement(1, 51, 55, "palette"),
        ],
      },
    ],
  };
  assert.deepEqual(applyWorkspaceEdit(edit, [{ uri, text: source, version: 1 }]), [
    { uri, text: expected, version: 1 },
  ]);
});

test("hand counted UTF16 positions cover LF, CRLF, CR and the final empty line", () => {
  const text = "😀a\r\nb\rc\n";
  for (const [offset, line, character] of [
    [0, 0, 0],
    [2, 0, 2],
    [3, 0, 3],
    [5, 1, 0],
    [6, 1, 1],
    [7, 2, 0],
    [8, 2, 1],
    [9, 3, 0],
  ]) {
    assert.equal(utf16Offset(text, { line, character }), offset);
    assert.deepEqual(utf16Position(text, offset), { line, character });
  }
  for (const offset of [1, 4, -1, 10, 1.5]) assert.throws(() => utf16Position(text, offset));
  for (const position of [
    { line: 0, character: 1 },
    { line: 0, character: 4 },
    { line: 4, character: 0 },
    { line: -1, character: 0 },
    { line: 0, character: 0.5 },
  ])
    assert.throws(() => utf16Offset(text, position));
});

test("adjacent replacements and a terminal insertion use original document coordinates", () => {
  const edits = [
    replacement(0, 2, 4, "B"),
    replacement(0, 4, 4, "!"),
    replacement(0, 0, 2, "LONG"),
  ];
  assert.equal(
    applyWorkspaceEdit({ changes: { [uri]: edits } }, [{ uri, text: "abcd", version: 1 }])[0].text,
    "LONGB!",
  );
});

test("same-position insertions retain array order before an adjacent replacement", () => {
  const insertions = [replacement(0, 3, 3, "one"), replacement(0, 3, 3, "two")];
  for (const edits of [insertions, [replacement(0, 3, 7, "palette"), ...insertions]]) {
    const packet = { changes: { [uri]: edits } };
    const original = structuredClone(packet);
    assert.deepEqual(applyWorkspaceEdit(packet, [document]), [
      {
        ...document,
        text: `😀 onetwo${edits.length === 2 ? "tone" : "palette"}\r\n隠 hidden\nlast`,
      },
    ]);
    assert.deepEqual(packet, original);
  }
});

test("malformed, reversed, duplicate and overlapping edits fail with the original packet", () => {
  const invalid = [
    [replacement(0, 7, 3, "wrong")],
    [replacement(0, 3, 7, "one"), replacement(0, 3, 7, "two")],
    [replacement(0, 3, 6, "one"), replacement(0, 5, 7, "two")],
    [replacement(0, 3, 7, "one"), replacement(0, 5, 5, "inside replacement")],
    [replacement(0, 1, 2, "split emoji")],
    [replacement(0, 7, 8, "line ending")],
    [replacement(5, 0, 0, "outside")],
    [{ range: range(0, 3, 7), newText: 12 }],
    [
      {
        range: { start: { line: 0, character: "3" }, end: { line: 0, character: 7 } },
        newText: "wrong",
      },
    ],
  ];
  for (const edits of invalid) {
    const packet = { changes: { [uri]: edits } };
    const before = structuredClone(packet);
    assert.throws(
      () => applyWorkspaceEdit(packet, [document]),
      (error: unknown) => {
        assert.ok(error instanceof WorkspaceEditFailure);
        assert.equal(error.edit, packet);
        return true;
      },
    );
    assert.deepEqual(packet, before);
    assert.equal(document.text, "😀 tone\r\n隠 hidden\nlast");
  }
});

test("foreign and stale document edits cannot partially modify the owned transaction", () => {
  const first = { textDocument: { uri, version: 1 }, edits: [replacement(0, 3, 7, "palette")] };
  for (const second of [
    { textDocument: { uri: "file:///foreign/E.vue", version: 1 }, edits: [] },
    { textDocument: { uri, version: 0 }, edits: [] },
    { textDocument: { uri, version: 1 }, edits: [] },
    { kind: "rename", oldUri: uri, newUri: "file:///foreign/E.vue" },
  ]) {
    const packet = { documentChanges: [first, second] };
    assert.throws(() => applyWorkspaceEdit(packet, [document]), WorkspaceEditFailure);
    assert.equal(document.text, "😀 tone\r\n隠 hidden\nlast");
  }
  assert.throws(
    () => applyWorkspaceEdit({ changes: {}, documentChanges: [] }, [document]),
    WorkspaceEditFailure,
  );
  assert.throws(
    () => applyWorkspaceEdit({ changes: {} }, [document, document]),
    WorkspaceEditFailure,
  );
});

test("multi-document annotation edits preserve untouched documents and raw metadata", () => {
  const second = { uri: "file:///owned/Other.vue", text: "tone", version: 4 };
  const untouched = { uri: "file:///owned/Unchanged.vue", text: "tone", version: 8 };
  const packet = {
    changeAnnotations: { rename: { label: "Rename property" } },
    documentChanges: [
      {
        textDocument: { uri, version: null },
        edits: [{ ...replacement(0, 3, 7, "palette"), annotationId: "rename" }],
      },
      { textDocument: { uri: second.uri, version: 4 }, edits: [replacement(0, 0, 4, "palette")] },
    ],
  };
  const before = structuredClone(packet);
  assert.deepEqual(applyWorkspaceEdit(packet, [document, second, untouched]), [
    { ...document, text: "😀 palette\r\n隠 hidden\nlast" },
    { ...second, text: "palette" },
    untouched,
  ]);
  assert.deepEqual(packet, before);
  delete (packet.changeAnnotations as Record<string, unknown>).rename;
  assert.throws(
    () => applyWorkspaceEdit(packet, [document, second, untouched]),
    WorkspaceEditFailure,
  );
  for (const [annotationId, annotation] of [
    ["toString", {}],
    ["rename", { rename: { label: 1 } }],
  ]) {
    assert.throws(
      () =>
        applyWorkspaceEdit(
          {
            changeAnnotations: annotation,
            changes: { [uri]: [{ ...replacement(0, 3, 7, "palette"), annotationId }] },
          },
          [document],
        ),
      WorkspaceEditFailure,
    );
  }
});
