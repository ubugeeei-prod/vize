import assert from "node:assert/strict";
import { test } from "node:test";
import {
  checkVersions,
  execute,
  executeRegistered,
  readPack,
  reference,
} from "../../tools/support/compat/davinci/jsx-js-module-reference.mjs";

const pack = readPack();

test("whole native JSX module references pin Babel, its Vue plugin and the actual Vue runtime", () => {
  assert.equal(pack.schema, "vize.native-jsx.js-module");
  assert.equal(pack.babelVersion, "7.29.0");
  assert.equal(pack.pluginVersion, "2.0.1");
  assert.equal(pack.vueVersion, "3.5.35");
  checkVersions(pack);
  assert.equal(pack.fixtures.length, 11);
  assert.equal(new Set(pack.fixtures.map((fixture: any) => fixture.id)).size, 11);
});

function vlq(segment: string) {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const fields: number[] = [];
  let value = 0,
    shift = 0;
  for (const character of segment) {
    const digit = alphabet.indexOf(character);
    assert(digit >= 0);
    value += (digit & 31) * 2 ** shift;
    if (digit & 32) shift += 5;
    else {
      fields.push(value & 1 ? -(value >> 1) : value >> 1);
      value = 0;
      shift = 0;
    }
  }
  assert.equal(shift, 0);
  return fields;
}

function checkMap(fixture: any) {
  const map = fixture.map;
  assert.equal(map.version, 3);
  assert.equal(map.file, "Native雪🌸.jsx");
  assert.deepEqual(map.sources, [map.file]);
  assert.deepEqual(map.sourcesContent, [fixture.source]);
  const authored = fixture.source.split(/\r\n|[\r\n\u2028\u2029]/u);
  const generated = fixture.code.split(/\r\n|[\r\n\u2028\u2029]/u);
  let sourceIndex = 0,
    sourceLine = 0,
    sourceColumn = 0,
    nameIndex = 0,
    segments = 0;
  const namedAnchors: unknown[] = [];
  map.mappings.split(";").forEach((line: string, generatedLine: number) => {
    let generatedColumn = 0;
    for (const encoded of line.split(",").filter(Boolean)) {
      const fields = vlq(encoded);
      assert(fields.length === 4 || fields.length === 5);
      generatedColumn += fields[0];
      sourceIndex += fields[1];
      sourceLine += fields[2];
      sourceColumn += fields[3];
      assert.equal(sourceIndex, 0);
      assert(generatedLine > 0, "synthetic runtime import has no authored source");
      assert(generatedColumn >= 0 && generatedColumn <= generated[generatedLine].length);
      assert(sourceLine >= 0 && sourceLine < authored.length);
      assert(sourceColumn >= 0 && sourceColumn <= authored[sourceLine].length);
      if (fields.length === 5) {
        nameIndex += fields[4];
        assert.equal(typeof map.names[nameIndex], "string");
        // Named reads/components retain the exact original spelling, including
        // the escaped identifier fixture, at their UTF-16 map anchor.
        assert.equal(generated[generatedLine][generatedColumn], authored[sourceLine][sourceColumn]);
        namedAnchors.push([map.names[nameIndex], generatedLine, sourceLine]);
      }
      segments++;
    }
  });
  assert(segments > 0);
  if (fixture.id.startsWith("line-ending-")) {
    assert.deepEqual(namedAnchors, [["message", 3, 2]]);
  }
}

for (const fixture of pack.fixtures) {
  test(`${fixture.id} independently transforms and executes complete native/reference modules`, async () => {
    const measured = reference(fixture, pack);
    assert.equal(measured.code, fixture.referenceCode);
    assert.deepEqual(measured.map, fixture.referenceMap);
    checkMap(fixture);
    assert.deepEqual(await execute(measured.code, fixture), fixture.runtime);
    assert.deepEqual(await execute(fixture.code, fixture), fixture.runtime);
  });
}

test("factory pragmas and pinned component classification require native whole-target refusal", async () => {
  assert.deepEqual(
    pack.refusals.map((fixture: any) => fixture.id),
    ["factory-pragma", "registered-search"],
  );
  for (const fixture of pack.refusals) {
    const measured = reference(fixture, pack);
    assert.equal(measured.code, fixture.referenceCode);
    assert.deepEqual(measured.map, fixture.referenceMap);
    const runtime = fixture.registered
      ? await executeRegistered(measured.code, fixture)
      : await execute(measured.code, fixture);
    assert.deepEqual(runtime, fixture.runtime);
    const tree = fixture.registered ? runtime : runtime.results[0].tree;
    assert.deepEqual(tree, [
      {
        kind: "element",
        tag: fixture.registered ? "b" : "span",
        props: {},
        children: [{ kind: "text", text: fixture.registered ? "registered" : "factory" }],
      },
    ]);
  }
});

test("an incorrect scalar payload fails the actual runtime comparison", async () => {
  const fixture = pack.fixtures.find((fixture: any) => fixture.id === "static-scalar-comments");
  const changed = fixture.code.replace("0x10", "0x11");
  assert.notEqual(changed, fixture.code);
  assert.notDeepEqual(await execute(changed, fixture), fixture.runtime);
});

test("a helper captured by a genuine parameter fails actual module execution", async () => {
  const fixture = pack.fixtures.find((fixture: any) => fixture.id === "all-scope-helper-collision");
  const changed = fixture.code.replaceAll("_vize_createVNode2", "_vize_createVNode0");
  assert.notEqual(changed, fixture.code);
  await assert.rejects(execute(changed, fixture), /not a function/);
});
