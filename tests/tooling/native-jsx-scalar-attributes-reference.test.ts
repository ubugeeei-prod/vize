import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import {
  checkVersions,
  executeSnapshots,
  readPack,
  reference,
} from "../../tools/support/compat/davinci/jsx-js-module-reference.mjs";

const pack = readPack(
  new URL(
    "../../davinci/vize_l4/tests/fixtures/jsx-scalar-attributes-vue-3.5.35.json",
    import.meta.url,
  ),
);
const fromRoot = createRequire(new URL("../../package.json", import.meta.url));
const babel = fromRoot("@babel/core");

test("scalar attributes retain a fixed independent original-source denominator", () => {
  checkVersions(pack);
  assert.equal(pack.schema, "vize.native-jsx.scalar-attributes");
  assert.deepEqual(
    pack.fixtures.map((fixture: any) => fixture.id),
    [
      "primitive-whitelist",
      "shadowed-read-updates-and-class",
      "escaped-unicode-read-and-string",
      "multiline-original-containers",
      "actual-file-helper-collisions",
      "mixed-original-attributes-and-child",
      "resolved-component-attributes",
      "original-module-read-attribute",
    ],
  );
  assert.equal(
    pack.fixtures.reduce(
      (count: number, fixture: any) => count + fixture.runtime.results.length,
      0,
    ),
    17,
  );
});

test("mounted prop updates retain every actual intermediate snapshot", () => {
  const fixture = pack.fixtures[1];
  assert.deepEqual(
    fixture.runtime.results.map((result: any) => result.tree[0].props),
    [
      { title: "雪🌸", id: "雪🌸", disabled: false, class: "front", "data-number": 0 },
      { disabled: true, class: "first active", "data-number": 0 },
      { title: "", id: "", disabled: 0, class: "ice🌸", "data-number": 0 },
      { title: 0, id: 0, disabled: false, "data-number": 0 },
    ],
  );
  assert.deepEqual(
    pack.fixtures[3].runtime.results.map((result: any) => result.tree[0].props),
    [
      { title: "line", id: "line" },
      { title: 0, id: 0 },
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
  for (const tag of ["div", "span", "Button"]) {
    const original = `<${tag}`;
    if (!fixture.source.includes(original)) continue;
    const generated = `${fixture.nodeHelper}(${tag === "Button" ? tag : JSON.stringify(tag)}`;
    const expected = {
      generated: position(fixture.code, fixture.code.indexOf(generated)),
      original: position(fixture.source, fixture.source.indexOf(original)),
      name: null,
    };
    assert(actual.some((anchor) => JSON.stringify(anchor) === JSON.stringify(expected)));
  }
  for (const literal of fixture.literals ?? []) {
    const sourceStart = fixture.source.indexOf(literal.original);
    const generatedStart = fixture.code.indexOf(literal.generated);
    assert(sourceStart >= 0 && generatedStart >= 0);
    const expected = {
      generated: position(
        fixture.code,
        generatedStart + literal.generated.lastIndexOf(literal.spelling),
      ),
      original: position(
        fixture.source,
        sourceStart + literal.original.lastIndexOf(literal.spelling),
      ),
      name: null,
    };
    assert(actual.some((anchor) => JSON.stringify(anchor) === JSON.stringify(expected)));
  }
  assert.deepEqual(fixture.map.names, fixture.mapNames);
  assert.deepEqual(
    actual.filter((anchor) => anchor.name !== null),
    fixture.reads.map((read: any) => {
      const sourceStart = fixture.source.indexOf(read.original);
      const generatedStart = fixture.code.indexOf(read.generated);
      assert(sourceStart >= 0 && generatedStart >= 0);
      return {
        generated: position(
          fixture.code,
          generatedStart + read.generated.lastIndexOf(read.spelling),
        ),
        original: position(fixture.source, sourceStart + read.original.lastIndexOf(read.spelling)),
        name: read.name,
      };
    }),
  );
}

for (const fixture of pack.fixtures) {
  test(`${fixture.id} retains complete pinned upstream modules, maps and mounted updates`, async () => {
    const measured = reference(fixture, pack);
    assert.equal(measured.code, fixture.referenceCode);
    assert.deepEqual(measured.map, fixture.referenceMap);
    assert.deepEqual(await executeSnapshots(measured.code, fixture), fixture.runtime);
  });
  test(`${fixture.id} retains genuine native modules, Read maps, comments and Vue updates`, async () => {
    assert.deepEqual(await executeSnapshots(fixture.code, fixture), fixture.runtime);
    assert.equal(fixture.map.file, "Scalar.jsx");
    assert.deepEqual(fixture.map.sources, ["Scalar.jsx"]);
    assert.deepEqual(fixture.map.sourcesContent, [fixture.source]);
    retainedAnchors(fixture);
    const parsed = babel.parseSync(fixture.code, {
      babelrc: false,
      configFile: false,
      sourceType: "module",
    });
    assert.deepEqual(
      parsed.comments.map((comment: any) => fixture.code.slice(comment.start, comment.end)),
      fixture.comments,
    );
  });
}

test("coherent source movement cannot bless stale within-bounds Read coordinates", () => {
  const fixture = structuredClone(pack.fixtures[1]);
  fixture.source = fixture.source.replace("title={message}", "title={ message}");
  fixture.reads[0].original = "title={ message}";
  fixture.map.sourcesContent = [fixture.source];
  assert.doesNotThrow(() => mapBounds(fixture));
  assert.throws(() => retainedAnchors(fixture));
});
