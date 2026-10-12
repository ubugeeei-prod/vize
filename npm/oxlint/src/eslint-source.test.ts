import assert from "node:assert/strict";
import { test } from "node:test";
import { assertSourceText, mapSourceOffsets, mapTextEdits } from "./eslint-source.ts";

await test("native source authority refuses lone surrogates and preserves complete pairs", () => {
  for (const source of ["", "ascii", "😀", "😀\r\n<img>"]) assertSourceText(source);
  for (const source of ["\ud800", "\udfff", "😀\ud800", "\udfff😀"]) {
    assert.throws(() => assertSourceText(source), /unpaired UTF-16 surrogate/);
  }
});

await test("UTF-8 edits after astral characters and CRLF address the original UTF-16 text", () => {
  const source = '😀\r\n<div  id="x">';
  const first = Buffer.byteLength("😀\r\n<div");
  const offsets = mapSourceOffsets(source, [first + 2, first]);
  const [edit] = mapTextEdits([{ start: first, end: first + 2, newText: " " }], offsets);
  assert.deepEqual(edit, { range: [8, 10], text: " " });
  assert.equal(
    source.slice(0, edit!.range[0]) + edit!.text + source.slice(edit!.range[1]),
    '😀\r\n<div id="x">',
  );
});

await test("byte boundaries cannot split an astral character or escape the source", () => {
  for (const offset of [-1, 0.5, NaN, 1, 2, 3, 5]) {
    assert.throws(() => mapSourceOffsets("😀", [offset]));
  }
  assert.deepEqual(
    [...mapSourceOffsets("😀", [4, 0])],
    [
      [0, 0],
      [4, 2],
    ],
  );
});

await test("overlapping, reversed and duplicate insertion edits are refused", () => {
  const offsets = mapSourceOffsets("abc", [0, 1, 2, 3]);
  for (const edits of [
    [{ start: 2, end: 1, newText: "x" }],
    [
      { start: 0, end: 2, newText: "x" },
      { start: 1, end: 3, newText: "y" },
    ],
    [
      { start: 1, end: 1, newText: "x" },
      { start: 1, end: 1, newText: "y" },
    ],
  ])
    assert.throws(() => mapTextEdits(edits, offsets));
  assert.deepEqual(
    mapTextEdits(
      [
        { start: 2, end: 3, newText: "y" },
        { start: 0, end: 1, newText: "x" },
      ],
      offsets,
    ),
    [
      { range: [0, 1], text: "x" },
      { range: [2, 3], text: "y" },
    ],
  );
});
