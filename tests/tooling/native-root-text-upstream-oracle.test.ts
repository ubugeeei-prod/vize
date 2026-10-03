import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";

type Span = [number, number];
type Location = {
  start: { column: number; line: number; offset: number };
  end: { column: number; line: number; offset: number };
  source: string;
};
type Node = { type: number; content?: string; loc: Location };
type ErrorValue = { code: number; message: string; loc: Location };
type TextWindow = { ordinal: number; raw: string; span: Span; content: string | null };
type RootSlot = { ordinal: number; kind: string; raw: string; span: Span };
type Fixture = {
  id: string;
  source: string;
  template: string;
  templateSpan: Span;
  nativeEligible: boolean;
  rootSlots: RootSlot[];
  rootText: TextWindow[];
  upstreamErrors: ErrorValue[];
  upstreamAst: unknown;
};
const pack = JSON.parse(
  readFileSync(
    new URL(
      "../../davinci/vize_l1/tests/fixtures/native-root-text-vue-3.5.35.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  schema: string;
  version: number;
  compiler: { name: string; version: string };
  options: { comments: boolean; whitespace: string };
  vueVersion: string;
  cases: Fixture[];
};
const require = createRequire(new URL("../package.json", import.meta.url));
const compiler = require("@vue/compiler-dom") as {
  parse(
    source: string,
    options: { comments: boolean; whitespace: string; onError(error: ErrorValue): void },
  ): { children: Node[] };
};
const version = require("@vue/compiler-dom/package.json").version;
// Preserve every enumerable AST field. A Set's complete contents use JSON arrays;
// no child, location, expression or parser field is selected out of the reference.
const plain = (value: unknown): unknown =>
  JSON.parse(JSON.stringify(value, (_key, item) => (item instanceof Set ? [...item] : item)));
const deferred = [
  "deferred-entity-ascii",
  "deferred-entity-blank",
  "deferred-entity-nbsp",
  "deferred-entity-single-decode",
  "deferred-nested-text",
  "deferred-pre-text",
  "deferred-svg-text",
  "deferred-recovered-element",
  "deferred-recovered-comment",
];

test("the complete root-text reference uses the independent pinned official compiler", () => {
  assert.equal(version, "3.5.35");
  assert.equal(pack.schema, "vize.native-root-text.upstream-reference");
  assert.equal(pack.version, 1);
  assert.equal(pack.vueVersion, "3.5.35");
  assert.deepEqual(pack.compiler, { name: "@vue/compiler-dom", version: "3.5.35" });
  assert.deepEqual(pack.options, { comments: true, whitespace: "condense" });
  assert.equal(pack.cases.length, 37);
  assert.equal(new Set(pack.cases.map(({ id }) => id)).size, 37);
  assert.equal(pack.cases.filter(({ nativeEligible }) => nativeEligible).length, 28);
  assert.deepEqual(
    pack.cases.filter(({ nativeEligible }) => !nativeEligible).map(({ id }) => id),
    deferred,
  );
});

for (const fixture of pack.cases) {
  test(`${fixture.id}: whole official parse, original root slots and omitted text windows`, () => {
    assert.equal(version, "3.5.35");
    const errors: ErrorValue[] = [];
    const ast = compiler.parse(fixture.template, {
      ...pack.options,
      onError: ({ code, message, loc }) => errors.push({ code, message, loc }),
    });
    assert.deepEqual(plain(ast), fixture.upstreamAst, "complete public parser AST");
    assert.deepEqual(plain(errors), fixture.upstreamErrors, "complete parser errors");

    const bytes = Buffer.from(fixture.source);
    const [start, end] = fixture.templateSpan;
    assert.equal(bytes.subarray(start, end).toString("utf8"), fixture.template);
    assert(bytes.subarray(0, start).toString("utf8").endsWith("<template>"));
    assert.equal(bytes.subarray(end).toString("utf8"), "</template>");
    assert.equal(fixture.rootSlots.map(({ raw }) => raw).join(""), fixture.template);
    let cursor = start;
    for (const [ordinal, slot] of fixture.rootSlots.entries()) {
      assert.equal(slot.ordinal, ordinal, "original ordinal includes omitted text");
      assert.deepEqual(slot.span, [cursor, cursor + Buffer.byteLength(slot.raw)]);
      assert.equal(bytes.subarray(...slot.span).toString("utf8"), slot.raw);
      cursor = slot.span[1];
    }
    assert.equal(cursor, end, "complete original root coverage");
    assert.deepEqual(
      fixture.rootText.map(({ ordinal, raw, span }) => ({ ordinal, raw, span })),
      fixture.rootSlots
        .filter(({ kind }) => kind === "text")
        .map(({ ordinal, raw, span }) => ({ ordinal, raw, span })),
    );
    const absoluteOffset = (offset: number) =>
      start + Buffer.byteLength(fixture.template.slice(0, offset));
    assert.deepEqual(
      ast.children
        .filter(({ type }) => type === 2)
        .map(({ content, loc }) => ({
          raw: loc.source,
          span: [absoluteOffset(loc.start.offset), absoluteOffset(loc.end.offset)],
          content,
        })),
      fixture.rootText
        .filter(({ content }) => content !== null)
        .map(({ raw, span, content }) => ({ raw, span, content })),
      "all retained text values, source custody and order; null windows are omitted",
    );
    // These full upstream observations grant no native admission to deferred
    // entity, nested/pre/foreign or recovered source families.
    assert.equal(fixture.nativeEligible, !deferred.includes(fixture.id));
    if (fixture.nativeEligible) assert.deepEqual(fixture.upstreamErrors, []);
  });
}
