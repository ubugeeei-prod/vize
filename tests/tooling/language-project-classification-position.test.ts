import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
const require = createRequire(import.meta.url);
const provider = require("../../editors/vscode/test/suite/classification-provider.cjs");
const input = {
  source: "a\r\n𐐀変数",
  data: [0, 0, 1, 7, 1, 1, 0, 4, 7, 33],
  spans: [0, 1, 2049, 3, 4, 2081],
};
class Position {
  readonly line: number;
  readonly character: number;
  constructor(line: number, character: number) {
    this.line = line;
    this.character = character;
  }
}
const document = {
  getText: () => input.source,
  offsetAt: (position: Position) => {
    assert(
      position instanceof Position,
      "Invalid argument: TextDocument.offsetAt requires Position",
    );
    return (position.line === 0 ? 0 : 3) + position.character;
  },
  positionAt: (offset: number) =>
    offset < 3 ? new Position(0, offset) : new Position(1, offset - 3),
};
const legend = {
  tokenTypes: [
    "class",
    "enum",
    "interface",
    "namespace",
    "typeParameter",
    "type",
    "parameter",
    "variable",
    "enumMember",
    "property",
    "function",
    "method",
  ],
  tokenModifiers: ["declaration", "static", "async", "readonly", "defaultLibrary", "local"],
};
test("the public TextDocument contract requires genuine Position instances for decoded semantic tokens", () => {
  assert.throws(() => document.offsetAt({ line: 0, character: 0 }), /Invalid argument/);
  assert.deepEqual(
    provider.decodeTokens(document, { data: input.data }, legend, Position).spans,
    input.spans,
  );
});
