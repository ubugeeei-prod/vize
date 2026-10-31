/** Finite, source-backed direct-factory edit; every other byte/field is strict. */
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import { isDeepStrictEqual } from "node:util";
import type ts from "../../../npm/cli/node_modules/typescript/lib/typescript.js";

const require = createRequire(import.meta.url);
const compiler = require(
  process.env.VIZE_DIRECT_FACTORY_TYPESCRIPT ??
    "../../../npm/cli/node_modules/typescript/lib/typescript.js",
) as typeof ts;
assert.equal(compiler.version, "6.0.3");
export const factory =
  "type $RefInstanceFactory<_C> = 0 extends (1 & _C) ? (component: _C) => (props: any) => undefined : _C extends { __vizeResolveRef?: infer _F } ? _F extends (...args: any[]) => any ? (component: _C) => _F : (component: _C) => (props: any) => undefined : _C extends (props: any, ctx?: any, expose?: any, ...args: any[]) => any ? (component: _C) => _C : (component: _C) => (props: any) => undefined;\n";
export const sharedInsertion = '\ndeclare module "vize:virtual-ts" {\nexport ' + factory + "}\n";
export function comparePreamble(before: string, after: string) {
  const anchor = readFileSync(
    new URL(
      "../../../crates/vize_canon/src/virtual_ts/helpers/declarations/shared-template-refs.d.ts.txt",
      import.meta.url,
    ),
    "utf8",
  );
  assert(anchor.length > 0 && before.split(anchor).length === 2);
  assert.equal(after, before.replace(anchor, anchor + sharedInsertion));
}
const prefix = 'import("vize:virtual-ts").';
export type Edit = { start: number; end: number; replacement: Buffer };
type Span = { start: number; end: number };
type Row = { generated: Span; authored: Span; subSpans: { generated: Span; authored: Span }[] };
type Link = { source: Span; target: Span };
type Feature = { name: string; toAuthored: (number | null)[]; toGenerated: (number | null)[] };
export type Projection = {
  file: string;
  source: string;
  hoisted: boolean;
  code: string;
  preRewrite: string;
  rows: Row[];
  semanticLinks: Link[];
  propDefaultKeyLinks: Link[];
  toAuthored: (number | null)[];
  toGenerated: (number | null)[];
  features: Feature[];
  importForward: number[];
  importReverse: number[];
  importMapDebug: string;
};
export type Capture = {
  schema: string;
  inputs: { file: string; wholeUtf8: string }[];
  sharedPreamble: string;
  files: Projection[];
};

export function point(value: number, edits: Edit[]) {
  assert(Number.isSafeInteger(value) && value >= 0);
  let delta = 0;
  for (const edit of edits) {
    if (value >= edit.end) delta += edit.replacement.length - (edit.end - edit.start);
    else {
      assert(value <= edit.start, "coordinate enters changed synthetic text");
      break;
    }
  }
  return value + delta;
}
function span(value: Span, edits: Edit[]) {
  assert(value.start <= value.end);
  assert(
    edits.every((e) => !(value.start < e.end && e.start < value.end)),
    "mapped span overlaps changed synthetic text",
  );
  return { start: point(value.start, edits), end: point(value.end, edits) };
}
export function apply(bytes: Buffer, edits: Edit[]) {
  let cursor = 0;
  const parts: Buffer[] = [];
  for (const e of edits) {
    assert(e.start >= cursor && e.end >= e.start && e.end <= bytes.length);
    parts.push(bytes.subarray(cursor, e.start), e.replacement);
    cursor = e.end;
  }
  return Buffer.concat([...parts, bytes.subarray(cursor)]);
}
function byteOffset(text: string, offset: number) {
  return Buffer.byteLength(text.slice(0, offset));
}
function scaffoldEdits(text: string): { edits: Edit[]; uses: number } {
  const ast = compiler.createSourceFile(
    "generated.ts",
    text,
    compiler.ScriptTarget.Latest,
    true,
    compiler.ScriptKind.TS,
  );
  const declarations: ts.TypeAliasDeclaration[] = [],
    uses: ts.TypeReferenceNode[] = [];
  function owned(node: ts.Node) {
    let parent = node.parent;
    while (parent && !compiler.isFunctionDeclaration(parent)) parent = parent.parent;
    return parent && parent.name?.text === "__setup";
  }
  function visit(node: ts.Node) {
    if (
      compiler.isTypeAliasDeclaration(node) &&
      node.name.text === "$RefInstanceFactory" &&
      owned(node)
    )
      declarations.push(node);
    if (
      compiler.isTypeReferenceNode(node) &&
      compiler.isIdentifier(node.typeName) &&
      node.typeName.text === "$RefInstanceFactory" &&
      owned(node)
    ) {
      assert(compiler.isAsExpression(node.parent) && node.parent.type === node);
      assert(
        compiler.isCallExpression(node.parent.parent.parent),
        "only the generated factory call is admitted",
      );
      uses.push(node);
    }
    compiler.forEachChild(node, visit);
  }
  visit(ast);
  if (!declarations.length) {
    assert.equal(uses.length, 0);
    return { edits: [], uses: 0 };
  }
  assert.equal(declarations.length, 1);
  const declaration = declarations[0],
    token = "  " + factory;
  const start = byteOffset(text, declaration.getStart(ast)) - 2;
  assert.equal(
    Buffer.from(text)
      .subarray(start, start + Buffer.byteLength(token))
      .toString(),
    token,
  );
  assert(uses.length > 0);
  const edits: Edit[] = [
    { start, end: start + Buffer.byteLength(token), replacement: Buffer.alloc(0) },
  ];
  for (const use of uses) {
    assert.equal(use.typeArguments?.length, 1);
    assert(compiler.isTypeQueryNode(use.typeArguments![0]));
    const at = byteOffset(text, use.getStart(ast));
    edits.push({ start: at, end: at, replacement: Buffer.from(prefix) });
  }
  edits.sort((a, b) => a.start - b.start);
  return { edits, uses: uses.length };
}
export function codeEdits(before: string, after: string, importDelta = 0) {
  const { edits, uses } = scaffoldEdits(before);
  const pre = () =>
    edits.map((e) => ({ ...e, start: e.start - importDelta, end: e.end - importDelta }));
  for (const match of before.matchAll(/\/\/ @vize-map: (\d+):(\d+) -> (\d+):(\d+)/gu)) {
    const at = byteOffset(before, match.index),
      a = Number(match[1]),
      b = Number(match[2]);
    assert(a <= b && b <= at);
    const updated = `// @vize-map: ${point(a, pre())}:${point(b, pre())} -> ${match[3]}:${match[4]}`;
    if (updated !== match[0]) {
      edits.push({
        start: at,
        end: at + Buffer.byteLength(match[0]),
        replacement: Buffer.from(updated),
      });
      edits.sort((a, b) => a.start - b.start);
    }
  }
  const expected = apply(Buffer.from(before), edits),
    actual = Buffer.from(after);
  assert(
    expected.equals(actual),
    `whole generated bytes differ: expected ${expected.length}, actual ${actual.length}`,
  );
  return { edits, uses };
}
function vector<T>(
  old: T[],
  edits: Edit[],
  fill: (e: Edit, offset: number, start: number) => T,
  outside: (v: T) => T,
): T[] {
  const result: T[] = [];
  let cursor = 0;
  for (const edit of edits) {
    for (; cursor < edit.start; cursor++) result.push(outside(old[cursor]));
    const start = result.length;
    for (let offset = 0; offset < edit.replacement.length; offset++)
      result.push(fill(edit, offset, start));
    cursor = edit.end;
  }
  for (; cursor < old.length; cursor++) result.push(outside(old[cursor]));
  assert.equal(result.length, point(old.length - 1, edits) + 1);
  return result;
}
const translated = (values: (number | null)[], edits: Edit[]) =>
  values.map((v) => (v === null ? null : point(v, edits)));
export function compareProjection(before: Projection, after: Projection) {
  if (!before.hoisted) {
    assert.deepEqual(after, before);
    return 0;
  }
  const pre = codeEdits(before.preRewrite, after.preRewrite);
  const oldAt = Buffer.from(before.code).indexOf("  " + factory);
  const delta = oldAt < 0 ? 0 : oldAt - before.importForward[oldAt];
  assert(Number.isSafeInteger(delta) && delta >= 0);
  const post = codeEdits(before.code, after.code, delta);
  assert.equal(post.uses, pre.uses);
  const expected = structuredClone(before);
  expected.code = after.code;
  expected.preRewrite = after.preRewrite;
  expected.rows = before.rows.map((row) => ({
    ...row,
    generated: span(row.generated, pre.edits),
    subSpans: row.subSpans.map((s) => ({ ...s, generated: span(s.generated, pre.edits) })),
  }));
  const links = (rows: Link[]) =>
    rows.map((l) => ({
      ...l,
      source: span(l.source, pre.edits),
      target: span(l.target, pre.edits),
    }));
  expected.semanticLinks = links(before.semanticLinks);
  expected.propDefaultKeyLinks = links(before.propDefaultKeyLinks);
  const generated = (old: (number | null)[]) => {
    for (const e of pre.edits)
      for (let i = e.start; i < e.end; i++)
        assert.equal(old[i], null, "changed helper text must be unauthored");
    return vector(
      old,
      pre.edits,
      () => null,
      (v) => v,
    );
  };
  expected.toAuthored = generated(before.toAuthored);
  expected.toGenerated = translated(before.toGenerated, pre.edits);
  expected.features = before.features.map((f) => ({
    ...f,
    toAuthored: generated(f.toAuthored),
    toGenerated: translated(f.toGenerated, pre.edits),
  }));
  expected.importForward = vector(
    before.importForward,
    post.edits,
    (_e, offset, start) => start + offset - delta,
    (v) => point(v, pre.edits),
  );
  expected.importReverse = vector(
    before.importReverse,
    pre.edits,
    (_e, offset, start) => start + offset + delta,
    (v) => point(v, post.edits),
  );
  assert(
    isDeepStrictEqual(after, expected),
    "complete focused geometry, all feature/import cells and unknown metadata",
  );
  return pre.uses;
}
export function compareCaptures(before: Capture, after: Capture) {
  assert.equal(before.schema, "vize.direct-ref-factory-projection-v1");
  comparePreamble(before.sharedPreamble, after.sharedPreamble);
  assert.equal(before.files.length, 4);
  assert.equal(after.files.length, 4);
  const uses = before.files.map((p, i) => compareProjection(p, after.files[i]));
  assert.deepEqual(uses, [0, 0, 0, 2]);
  assert.deepEqual(after, { ...before, sharedPreamble: after.sharedPreamble, files: after.files });
  return {
    fullPackets: 4,
    defaultPacketsExact: 2,
    hoistedUses: 2,
    exhaustiveFocusedGeometry: true,
  };
}
