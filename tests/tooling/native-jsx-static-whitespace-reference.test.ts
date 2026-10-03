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
      count += 1;
    }
  });
  assert(count > 0);
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
    mapBounds(fixture);
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
