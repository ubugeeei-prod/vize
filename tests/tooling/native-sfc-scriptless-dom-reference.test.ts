import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_sfc_dom_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const compiler = fromVue("@vue/compiler-dom");
const runtime = fromVue("vue");
const dataUrl = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const runtimeUrl = dataUrl(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    [
      "toDisplayString",
      "openBlock",
      "createElementBlock",
      "createElementVNode",
      "createTextVNode",
      "createCommentVNode",
      "Fragment",
      "normalizeClass",
      "normalizeStyle",
    ]
      .map((name) => `export const ${name} = runtime.${name};`)
      .join("\n"),
);

test("whole scriptless component references retain the pinned complete render output", () => {
  assert.equal(pack.schema, "vize.native-sfc.scriptless-dom-reference");
  assert.equal(pack.compiler.version, "3.5.35");
  assert.equal(fromVue("@vue/compiler-dom/package.json").version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.equal(pack.fixtures.length, 8);
  assert.equal(new Set(pack.fixtures.map((fixture: any) => fixture.id)).size, 8);
  for (const fixture of pack.fixtures) {
    const measured = compiler.compile(fixture.template, {
      mode: "module",
      hoistStatic: false,
      prefixIdentifiers: true,
      comments: true,
      filename: "FileDom.vue",
      sourceMap: true,
      bindingMetadata: {},
      cacheHandlers: false,
    });
    assert.equal(measured.code, fixture.referenceRender, fixture.id);
    assert.equal(fixture.source, `<template>${fixture.template}</template>`, fixture.id);
    assert(fixture.code.endsWith("_sfc_main.render = render\nexport default _sfc_main\n"));
    checkNativeMap(fixture);
  }
});

function decodeVlq(segment: string): number[] {
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

function checkNativeMap(fixture: any): void {
  const map = fixture.nativeMap;
  assert.deepEqual(Object.keys(map).sort(), [
    "file",
    "mappings",
    "names",
    "sources",
    "sourcesContent",
    "version",
  ]);
  assert.equal(map.version, 3);
  assert.equal(map.file, "Native雪🌸.vue");
  assert.deepEqual(map.sources, [map.file]);
  assert.deepEqual(map.sourcesContent, [fixture.source]);
  assert.deepEqual(map.names, []);
  const generated = fixture.code.split("\n");
  let sourceIndex = 0,
    sourceLine = 0,
    sourceColumn = 0,
    segments = 0;
  map.mappings.split(";").forEach((line: string, lineIndex: number) => {
    let column = 0;
    for (const segment of line.split(",").filter(Boolean)) {
      const fields = decodeVlq(segment);
      assert.equal(fields.length, 4);
      column += fields[0];
      sourceIndex += fields[1];
      sourceLine += fields[2];
      sourceColumn += fields[3];
      assert.equal(sourceIndex, 0);
      assert.equal(sourceLine, 0);
      assert(generated[lineIndex] !== undefined);
      assert(column >= 0 && column <= generated[lineIndex].length);
      assert(sourceColumn >= 10 && sourceColumn <= fixture.source.length - 11);
      segments += 1;
    }
  });
  assert(segments > 0);
}

function shape(node: any): any {
  assert(runtime.isVNode(node));
  const type =
    node.type === runtime.Fragment
      ? "fragment"
      : node.type === runtime.Text
        ? "text"
        : node.type === runtime.Comment
          ? "comment"
          : node.type;
  return {
    type,
    props: node.props ?? {},
    children: Array.isArray(node.children) ? node.children.map(shape) : node.children,
    patchFlag: node.patchFlag,
  };
}

for (const fixture of pack.fixtures) {
  test(`${fixture.id} complete component module attaches the actual render`, async () => {
    // The genuine Rust SFC pipeline compares its complete output to these bytes.
    // This dev loader changes only the pinned runtime import address.
    const loaded = await import(
      dataUrl(fixture.code.replace('from "vue"', `from ${JSON.stringify(runtimeUrl)}`))
    );
    assert.equal(typeof loaded.default.render, "function");
    assert.deepEqual(Object.keys(loaded.default), ["render"]);
    const forbidden = new Proxy(
      {},
      {
        get(_target, key) {
          throw Error(`Unexpected scriptless access ${String(key)}`);
        },
      },
    );
    for (const context of [{}, forbidden]) {
      const node = loaded.default.render(context, [], forbidden, forbidden, forbidden, forbidden);
      assert.deepEqual(shape(node), fixture.runtime, fixture.id);
      assert.equal(node.key, null);
      assert.deepEqual(node.dynamicChildren, []);
    }
  });
}
