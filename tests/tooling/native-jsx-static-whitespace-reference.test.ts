import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import {
  checkVersions,
  execute,
  readPack,
  reference,
} from "../../tools/support/compat/davinci/jsx-js-module-reference.mjs";

const pack = readPack(
  new URL(
    "../../davinci/vize_l4/tests/fixtures/jsx-static-whitespace-vue-3.5.35.json",
    import.meta.url,
  ),
);
const fromRoot = createRequire(new URL("../../package.json", import.meta.url));
const babel = fromRoot("@babel/core");

test("static whitespace retains a fixed independent original-source denominator", () => {
  assert.equal(pack.schema, "vize.native-jsx.static-whitespace");
  checkVersions(pack);
  assert.deepEqual(
    pack.fixtures.map((fixture: any) => fixture.id),
    [
      "lf-empty-text",
      "crlf-nested-text",
      "cr-static-attribute",
      "single-tab-text",
      "unicode-separator-text",
      "empty-attribute-and-comments",
      "original-lf-refusal-now-admitted",
      "helper-collision-and-read",
    ],
  );
});

function fields(encoded: string) {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const result: number[] = [];
  let value = 0,
    shift = 0;
  for (const character of encoded) {
    const digit = alphabet.indexOf(character);
    assert(digit >= 0);
    value += (digit & 31) * 2 ** shift;
    if (digit & 32) shift += 5;
    else {
      result.push(value & 1 ? -(value >> 1) : value >> 1);
      value = shift = 0;
    }
  }
  assert.equal(shift, 0);
  return result;
}

function mapBounds(fixture: any) {
  const original = fixture.source.split(/\r\n|[\r\n\u2028\u2029]/u);
  const generated = fixture.code.split(/\r\n|[\r\n\u2028\u2029]/u);
  let sourceIndex = 0,
    sourceLine = 0,
    sourceColumn = 0,
    nameIndex = 0,
    count = 0;
  const anchors: any[] = [];
  fixture.map.mappings.split(";").forEach((line: string, generatedLine: number) => {
    let column = 0;
    for (const encoded of line.split(",").filter(Boolean)) {
      const entry = fields(encoded);
      assert(entry.length === 4 || entry.length === 5);
      column += entry[0];
      sourceIndex += entry[1];
      sourceLine += entry[2];
      sourceColumn += entry[3];
      assert.equal(sourceIndex, 0);
      assert(
        generatedLine < generated.length &&
          column >= 0 &&
          column <= generated[generatedLine].length,
      );
      assert(
        sourceLine >= 0 &&
          sourceLine < original.length &&
          sourceColumn >= 0 &&
          sourceColumn <= original[sourceLine].length,
      );
      if (entry.length === 5) {
        nameIndex += entry[4];
        assert.equal(typeof fixture.map.names[nameIndex], "string");
      }
      anchors.push({
        generated: [generatedLine, column],
        original: [sourceLine, sourceColumn],
        name: entry.length === 5 ? fixture.map.names[nameIndex] : null,
      });
      count += 1;
    }
  });
  assert(count > 0);
  return anchors;
}

function position(text: string, offset: number) {
  assert(offset >= 0);
  const lines = text.slice(0, offset).split(/\r\n|[\r\n\u2028\u2029]/u);
  return [lines.length - 1, lines.at(-1)!.length];
}

function retainedAnchors(fixture: any) {
  const actual = mapBounds(fixture);
  // The frozen intrinsic roots have independently identifiable original and
  // generated starts; a bounded but wrong source coordinate must still fail.
  for (const [original, generated] of [
    ["<div", '_vize_createVNode0("div"'],
    ["<span", '_vize_createVNode0("span"'],
  ]) {
    if (!fixture.source.includes(original)) continue;
    const expected = {
      generated: position(fixture.code, fixture.code.indexOf(generated)),
      original: position(fixture.source, fixture.source.indexOf(original)),
      name: null,
    };
    assert(actual.some((anchor) => JSON.stringify(anchor) === JSON.stringify(expected)));
  }
  const names = fixture.id === "helper-collision-and-read" ? ["message"] : [];
  assert.deepEqual(fixture.map.names, names);
  assert.deepEqual(
    actual.filter((anchor) => anchor.name !== null),
    names.map((name) => ({
      generated: position(fixture.code, fixture.code.lastIndexOf(name)),
      original: position(fixture.source, fixture.source.lastIndexOf(name)),
      name,
    })),
  );
}

for (const fixture of pack.fixtures) {
  test(`${fixture.id} preserves the complete pinned upstream map and mounted tree`, async () => {
    const measured = reference(fixture, pack);
    assert.equal(measured.code, fixture.referenceCode);
    assert.deepEqual(measured.map, fixture.referenceMap);
    assert.deepEqual(await execute(measured.code, fixture), fixture.runtime);
  });

  test(`${fixture.id} retains genuine native whole-module runtime, maps and comments`, async () => {
    assert.deepEqual(await execute(fixture.code, fixture), fixture.runtime);
    assert.equal(fixture.map.file, "Whitespace.jsx");
    assert.deepEqual(fixture.map.sources, ["Whitespace.jsx"]);
    assert.deepEqual(fixture.map.sourcesContent, [fixture.source]);
    retainedAnchors(fixture);
    if (fixture.comments) {
      const parsed = babel.parseSync(fixture.code, {
        babelrc: false,
        configFile: false,
        sourceType: "module",
      });
      assert.deepEqual(
        parsed.comments.map((comment: any) => fixture.code.slice(comment.start, comment.end)),
        fixture.comments,
      );
    }
  });
}

test("coherent source movement cannot bless stale bounded native map coordinates", () => {
  const fixture = structuredClone(pack.fixtures[0]);
  fixture.source = fixture.source.replace("return ", "return  ");
  fixture.map.sourcesContent = [fixture.source];
  assert.doesNotThrow(() => mapBounds(fixture));
  assert.throws(() => retainedAnchors(fixture));
});
