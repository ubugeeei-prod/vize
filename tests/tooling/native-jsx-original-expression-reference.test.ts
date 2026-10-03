import assert from "node:assert/strict";
import { test } from "node:test";
import {
  checkVersions,
  execute,
  readPack,
  reference,
} from "../../tools/support/compat/davinci/jsx-js-module-reference.mjs";

const pack = readPack(
  new URL(
    "../../davinci/vize_l4/tests/fixtures/jsx-original-expression-vue-3.5.35.json",
    import.meta.url,
  ),
);

test("original-expression modules retain a fixed independent parser/plugin/runtime denominator", () => {
  assert.equal(pack.schema, "vize.native-jsx.original-expression");
  checkVersions(pack);
  assert.deepEqual(
    pack.fixtures.map((fixture: any) => fixture.id),
    [
      "module-and-local-scalars",
      "original-return-and-local-binary",
      "original-call-and-helper-collision",
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

function position(text: string, offset: number) {
  const lines = text.slice(0, offset).split(/\r\n|[\r\n\u2028\u2029]/u);
  return [lines.length - 1, lines.at(-1)!.length];
}

function namedLinks(fixture: any) {
  let sourceLine = 0,
    sourceColumn = 0,
    nameIndex = 0;
  const actual: unknown[] = [];
  fixture.map.mappings.split(";").forEach((line: string, generatedLine: number) => {
    let column = 0;
    for (const encoded of line.split(",").filter(Boolean)) {
      const entry = fields(encoded);
      assert(entry.length === 4 || entry.length === 5);
      column += entry[0];
      sourceLine += entry[2];
      sourceColumn += entry[3];
      if (entry.length === 5) {
        nameIndex += entry[4];
        actual.push([
          fixture.map.names[nameIndex],
          [generatedLine, column],
          [sourceLine, sourceColumn],
        ]);
      }
    }
  });
  return actual;
}

for (const fixture of pack.fixtures) {
  test(`${fixture.id} executes complete preserved original source with genuine scalar JSX reads`, async () => {
    const measured = reference(fixture, pack);
    assert.equal(measured.code, fixture.referenceCode);
    assert.deepEqual(measured.map, fixture.referenceMap);
    assert.deepEqual(await execute(measured.code, fixture), fixture.runtime);
    assert.deepEqual(await execute(fixture.code, fixture), fixture.runtime);
    assert.equal(fixture.map.file, "Original.jsx");
    assert.deepEqual(fixture.map.sources, ["Original.jsx"]);
    assert.deepEqual(fixture.map.sourcesContent, [fixture.source]);
    assert.deepEqual(
      namedLinks(fixture),
      fixture.map.names.map((name: string) => [
        name,
        position(fixture.code, fixture.code.lastIndexOf(name)),
        position(fixture.source, fixture.source.lastIndexOf(name)),
      ]),
    );
  });
}

test("mutating an unrelated original initializer changes actual mounted output", async () => {
  const fixture = pack.fixtures[0];
  const changed = fixture.code.replace("0x10", "0x11");
  assert.notEqual(changed, fixture.code);
  assert.notDeepEqual(await execute(changed, fixture), fixture.runtime);
});
