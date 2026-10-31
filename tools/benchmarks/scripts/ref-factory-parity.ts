/** One source-backed factory edit and its synthetic numeric map comments. */
import assert from "node:assert/strict";
import { isDeepStrictEqual } from "node:util";
import { readRefFactoryAuthority } from "./ref-factory-authority.ts";

type Edit = { start: number; end: number; replacement: Buffer };
type Range = { start: number; end: number };
type Row = {
  generated: Range;
  authored: Range;
  subSpans: { generated: Range; authored: Range }[];
  features: number;
  kind: string;
};
type Link = { source: Range; target: Range; kind: string };
export type FactoryPacket = {
  file: string;
  source: string;
  code: string;
  authoredBase: number;
  rows: Row[];
  semanticLinks: Link[];
  propDefaultKeyLinks: Link[];
  authoredBlocks: string[];
  importMapDebug: string;
  importForward: number[];
  importReverse: number[];
  compositeForward: unknown[];
};
export const factoryForward =
  '  type $RefInstanceFactory<_C> = import("vize:virtual-ts").$RefInstanceFactory<_C>;\n';
const safe = (n: number) => assert(Number.isSafeInteger(n) && n >= 0);
function point(value: number, edits: Edit[]) {
  safe(value);
  let delta = 0;
  for (const edit of edits) {
    if (value >= edit.end) delta += edit.replacement.length - edit.end + edit.start;
    else {
      assert(value <= edit.start, "coordinate enters the certified synthetic edit");
      break;
    }
  }
  return value + delta;
}
function overlaps(start: number, end: number, edit: Edit) {
  return start < edit.end && edit.start < end;
}
function range(value: Range, edits: Edit[]) {
  safe(value.start);
  safe(value.end);
  assert(value.start <= value.end);
  assert(
    edits.every((edit) => !overlaps(value.start, value.end, edit)),
    "authored span enters a synthetic edit",
  );
  return { start: point(value.start, edits), end: point(value.end, edits) };
}
function apply(bytes: Buffer, edits: Edit[]) {
  let start = 0;
  const parts: Buffer[] = [];
  for (const edit of edits) {
    assert(edit.start >= start && edit.end >= edit.start && edit.end <= bytes.length);
    parts.push(bytes.subarray(start, edit.start), edit.replacement);
    start = edit.end;
  }
  return Buffer.concat([...parts, bytes.subarray(start)]);
}
function once(bytes: Buffer, token: Buffer) {
  const at = bytes.indexOf(token);
  assert(at >= 0 && bytes.indexOf(token, at + 1) < 0, "factory must occur exactly once");
  return at;
}

export function factoryCodeEdits(
  before: string,
  after: string,
  oldFactory: string,
  importDelta: number,
) {
  safe(importDelta);
  const bytes = Buffer.from(before),
    old = Buffer.from(oldFactory),
    forward = Buffer.from(factoryForward);
  if (bytes.indexOf(old) < 0) {
    assert.equal(after, before);
    return { post: [] as Edit[], pre: [] as Edit[] };
  }
  const start = once(bytes, old);
  once(Buffer.from(after), forward);
  assert(start >= importDelta);
  const edits: Edit[] = [{ start, end: start + old.length, replacement: forward }];
  const pre = () =>
    edits.map((edit) => ({
      ...edit,
      start: edit.start - importDelta,
      end: edit.end - importDelta,
    }));
  // The generated comments name ranges already emitted before the comment.
  // Only changed numeric comments are added; every other byte stays exact.
  for (const match of before.matchAll(/\/\/ @vize-map: (\d+):(\d+) -> (\d+):(\d+)/gu)) {
    const at = Buffer.byteLength(before.slice(0, match.index));
    const a = Number(match[1]),
      b = Number(match[2]);
    assert(a <= b && b <= at, "unknown generated map-comment direction");
    const updated = `// @vize-map: ${point(a, pre())}:${point(b, pre())} -> ${match[3]}:${match[4]}`;
    if (updated !== match[0]) {
      assert(at >= start + old.length, "changed comment precedes the factory");
      edits.push({
        start: at,
        end: at + Buffer.byteLength(match[0]),
        replacement: Buffer.from(updated),
      });
      edits.sort((left, right) => left.start - right.start);
    }
  }
  assert.equal(
    apply(bytes, edits).toString("utf8"),
    after,
    "complete code admits only the exact factory/comment edits",
  );
  return { post: edits, pre: pre() };
}

function vector<T>(
  old: T[],
  edits: Edit[],
  fill: (edit: Edit, offset: number, newStart: number) => T,
  outside: (value: T) => T,
) {
  assert(old.length > 0);
  const result: T[] = new Array(point(old.length - 1, edits) + 1),
    visited = new Set<number>();
  const put = (index: number, value: T) => {
    safe(index);
    assert(index < result.length && !visited.has(index));
    visited.add(index);
    result[index] = value;
  };
  old.forEach((value, index) => {
    if (edits.every((edit) => index < edit.start || index >= edit.end))
      put(point(index, edits), outside(value));
  });
  for (const edit of edits) {
    const newStart = point(edit.start, edits);
    for (let offset = 0; offset < edit.replacement.length; offset++)
      put(newStart + offset, fill(edit, offset, newStart));
  }
  assert.equal(visited.size, result.length, "the complete vector must be covered exactly once");
  return result;
}

export function compareFactoryPacket(
  before: FactoryPacket,
  after: FactoryPacket,
  oldFactory: string,
) {
  const oldBytes = Buffer.from(before.code),
    start = oldBytes.indexOf(oldFactory);
  if (start < 0) {
    assert.deepEqual(after, before);
    return { changed: false, edits: 0 };
  }
  assert.equal(before.importForward.length, oldBytes.length + 1);
  const importDelta = start - before.importForward[start];
  safe(importDelta);
  const edits = factoryCodeEdits(before.code, after.code, oldFactory, importDelta);
  assert.equal(
    before.importMapDebug,
    after.importMapDebug,
    "all import rewrites precede this one factory in the bounded inputs",
  );
  const expected = structuredClone(before);
  expected.code = after.code;
  expected.rows = before.rows.map((row) => ({
    ...row,
    generated: range(row.generated, edits.pre),
    subSpans: row.subSpans.map((span) => ({
      ...span,
      generated: range(span.generated, edits.pre),
    })),
  }));
  const links = (rows: Link[]) =>
    rows.map((link) => ({
      ...link,
      source: range(link.source, edits.pre),
      target: range(link.target, edits.pre),
    }));
  expected.semanticLinks = links(before.semanticLinks);
  expected.propDefaultKeyLinks = links(before.propDefaultKeyLinks);
  for (const edit of edits.post) {
    for (let offset = edit.start; offset < edit.end; offset++)
      assert.equal(
        before.compositeForward[offset],
        null,
        "old edited helper/comment must be unauthored",
      );
    for (let offset = edit.start; offset < edit.end; offset++)
      assert.equal(
        before.importForward[offset],
        offset - importDelta,
        "old synthetic text must not hide an import rewrite",
      );
  }
  for (const edit of edits.pre)
    for (let offset = edit.start; offset < edit.end; offset++)
      assert.equal(
        before.importReverse[offset],
        offset + importDelta,
        "old synthetic reverse import cells must remain exact",
      );
  expected.compositeForward = vector(
    before.compositeForward,
    edits.post,
    () => null,
    (value) => value,
  );
  expected.importForward = vector(
    before.importForward,
    edits.post,
    (edit, offset) => point(edit.start - importDelta, edits.pre) + offset,
    (value) => point(value, edits.pre),
  );
  expected.importReverse = vector(
    before.importReverse,
    edits.pre,
    (edit, offset) => point(edit.start + importDelta, edits.post) + offset,
    (value) => point(value, edits.post),
  );
  assert.deepEqual(
    after,
    expected,
    "whole mappings/roles/links/import vectors/unknown metadata remain exact",
  );
  assert.equal(after.importForward.length, Buffer.byteLength(after.code) + 1);
  return { changed: true, edits: edits.post.length };
}

export function compareFactoryProjection(
  before: { schema: string; inputs: unknown; sharedPreamble: string; files: FactoryPacket[] },
  after: typeof before,
) {
  const authority = readRefFactoryAuthority();
  assert.equal(before.schema, "vize.ref-factory-projection-v1");
  assert.equal(before.sharedPreamble, authority.preamble.beforeWholeUtf8);
  assert.equal(after.sharedPreamble, authority.preamble.afterWholeUtf8);
  const oldFactory = "  " + authority.declaration.wholeUtf8;
  assert.equal(before.files.length, 2);
  assert.equal(after.files.length, 2);
  const rows = before.files.map((row, index) =>
    compareFactoryPacket(row, after.files[index], oldFactory),
  );
  assert.deepEqual(
    rows.map((row) => row.changed),
    [false, true],
  );
  const expected = { ...before, sharedPreamble: after.sharedPreamble, files: after.files };
  assert(isDeepStrictEqual(after, expected), "all outer metadata, inputs and order stay intact");
  return { completePackets: rows.length, changed: rows.filter((row) => row.changed).length, rows };
}
