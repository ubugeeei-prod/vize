// Author immutable packets from literal source coordinates, never editor output.
import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";

type FileName = "Child.vue" | "Parent.vue" | "Other.vue";
type Origin = "child" | "v-model" | "update";
type Site = readonly [Origin, FileName, number, number, number, string, string];

interface Point {
  readonly line: number;
  readonly character: number;
}
interface Span {
  readonly start: Point;
  readonly end: Point;
}
interface AuthoredCase {
  readonly queries: readonly unknown[];
  readonly [field: string]: unknown;
}
interface Edit {
  readonly file: FileName;
  readonly range: Span;
  readonly newText: string;
}

const root = fileURLToPath(new URL(".", import.meta.url));
const files: readonly FileName[] = ["Child.vue", "Parent.vue", "Other.vue"];
// Explicit authored UTF-16 positions. The first parent site follows an astral glyph.
const sites: readonly Site[] = [
  ["child", "Child.vue", 3, 21, 29, "saveItem", "nextValue"],
  ["v-model", "Parent.vue", 10, 28, 36, "saveItem", "next-value"],
  ["v-model", "Parent.vue", 11, 17, 26, "save-item", "next-value"],
  ["update", "Parent.vue", 12, 10, 25, "update:saveItem", "update:next-value"],
  ["update", "Parent.vue", 13, 10, 26, "update:save-item", "update:next-value"],
];

function span(line: number, start: number, end: number): Span {
  return { start: { line, character: start }, end: { line, character: end } };
}

/** UTF-16 offset of a point; JavaScript strings index UTF-16 code units. */
function pointOffset(text: string, point: Point): number {
  const lines = text.split(/(?<=\n)/u);
  const line = (lines[point.line] ?? "").replace(/\r?\n$/u, "");
  const before = lines.slice(0, point.line).reduce((total, value) => total + value.length, 0);
  return before + Math.min(point.character, line.length);
}

function load(prefix: string, newline: string): Record<FileName, string> {
  const read = (name: FileName): string =>
    fs.readFileSync(`${root}${prefix}-${name}.txt`, "utf8").replaceAll("\n", newline);
  return { "Child.vue": read("Child.vue"), "Parent.vue": read("Parent.vue"), "Other.vue": read("Other.vue") };
}

function envelope(id: number, result: unknown): unknown {
  return { jsonrpc: "2.0", id, result };
}

function emptyDiagnostics(): Record<FileName, never[]> {
  return { "Child.vue": [], "Parent.vue": [], "Other.vue": [] };
}

function authorCase(origin: Origin, newline: string, newName: string): AuthoredCase {
  const sources = load("input", newline);
  const goldens = load("updated", newline);
  const references: { uri: string; range: Span }[] = [];
  const edits: Edit[] = [];
  const changes: Record<string, { range: Span; newText: string }[]> = {};
  for (const [, name, line, start, end, token, replacement] of sites) {
    const interval = span(line, start, end);
    const lo = pointOffset(sources[name], interval.start);
    const hi = pointOffset(sources[name], interval.end);
    assert.equal(sources[name].slice(lo, hi), token);
    references.push({ uri: `$${name}`, range: interval });
    edits.push({ file: name, range: interval, newText: replacement });
    (changes[`$${name}`] ??= []).push({ range: interval, newText: replacement });
  }
  // The complete text golden is independently authored, not made from a reply.
  for (const name of files) {
    let text = sources[name];
    for (const edit of edits.filter((entry) => entry.file === name).reverse()) {
      const lo = pointOffset(text, edit.range.start);
      const hi = pointOffset(text, edit.range.end);
      text = text.slice(0, lo) + edit.newText + text.slice(hi);
    }
    assert.equal(text, goldens[name]);
  }
  assert.equal(sources["Other.vue"], goldens["Other.vue"]);
  const definition = references[0];
  const queries: unknown[] = [];
  let nextId = 2;
  for (const [kind, name, line, start, end, token] of sites) {
    if (kind !== origin) continue;
    const interval = span(line, start, end);
    for (let character = start; character < end; character += 1) {
      const params = { textDocument: { uri: `$${name}` }, position: { line, character } };
      const requests: Record<string, unknown> = {};
      const replies: Record<string, unknown[]> = {};
      const methods: readonly [string, Record<string, unknown>, readonly unknown[]][] = [
        ["prepareRename", {}, [interval, { range: interval, placeholder: token }]],
        ["references", { context: { includeDeclaration: true } }, [references]],
        ["definition", {}, [definition, [definition]]],
        ["rename", { newName }, [{ changes }]],
      ];
      for (const [method, additional, results] of methods) {
        requests[method] = {
          jsonrpc: "2.0",
          id: nextId,
          method: `textDocument/${method}`,
          params: { ...params, ...additional },
        };
        replies[method] = results.map((result) => envelope(nextId, result));
        nextId += 1;
      }
      queries.push({ file: name, position: params.position, requests, allowedWholeReplies: replies });
    }
  }
  return {
    origin,
    newline,
    newName,
    sources,
    fullReferences: references,
    fullEdits: edits,
    queries,
    selectedApplicationQueryIndex: 0,
    completeGoldenFiles: goldens,
    expectedVersion2Diagnostics: emptyDiagnostics(),
    expectedIndependentVersion3Diagnostics: emptyDiagnostics(),
    negativePreservation: "Other declaration/usages and independent strings stay byte-exact",
  };
}

const cases: AuthoredCase[] = [];
for (const origin of ["child", "v-model", "update"] as const)
  for (const newline of ["\n", "\r\n"])
    for (const name of ["nextValue", "next-value"])
      cases.push(authorCase(origin, newline, name));
assert.equal(cases.length, 12);
assert.equal(
  cases.reduce((total, entry) => total + entry.queries.length, 0),
  224,
);
const authored = { schema: "vize-named-model-editor-contracts-v1", cases };
fs.writeFileSync(`${root}cases.json.txt`, `${JSON.stringify(authored, null, 2)}\n`);
