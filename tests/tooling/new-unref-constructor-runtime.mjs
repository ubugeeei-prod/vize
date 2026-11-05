// #11915: run complete source-compiled or freshly installed public components.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";

const fixtures = new URL(
  "../_fixtures/differential/compiler/new-unref-constructor-11915/",
  import.meta.url,
);
const cases = JSON.parse(fs.readFileSync(new URL("cases.json", fixtures), "utf8"));
const provider = process.argv[2];
const require = createRequire(
  provider
    ? path.join(path.resolve(provider), "package.json")
    : new URL("../../npm/ui/package.json", import.meta.url),
);
const compiler = require("@vue/compiler-sfc");
const runtime = require("vue");
const dataUrl = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
const vueUrl = dataUrl(
  `import runtime from ${JSON.stringify(pathToFileURL(require.resolve("vue")).href)};\n` +
    [
      "unref",
      "isRef",
      "ref",
      "openBlock",
      "createBlock",
      "createElementBlock",
      "createVNode",
      "createElementVNode",
      "toDisplayString",
      "resolveComponent",
      "Fragment",
      "defineComponent",
      "createTextVNode",
      "createCommentVNode",
    ]
      .map((name) => `export const ${name} = runtime.${name};`)
      .join("\n"),
);
const vectorsUrl = dataUrl(fs.readFileSync(new URL("vector3.mjs.txt", fixtures), "utf8"));
const sourceFor = (row) =>
  `<script setup>\n${row.script}\n</script>\n<template><Cam :look-at="${row.expression}" /></template>\n`;
const sourceCases = cases.map((row) => ({ id: row.id, source: sourceFor(row) }));
assert.equal(sourceCases[0].source, fs.readFileSync(new URL("Layout.vue.txt", fixtures), "utf8"));
let captured;
if (provider) {
  const { compileSfc } = require("@vizejs/native");
  captured = sourceCases.map(({ id, source }) => ({
    id,
    source,
    result: compileSfc(source, { filename: "Cam.vue", isProd: true, inlineTemplate: true }),
    mapped: compileSfc(source, {
      filename: "Cam.vue",
      isProd: true,
      inlineTemplate: true,
      sourceMap: true,
    }),
  }));
} else {
  let input = "";
  for await (const chunk of process.stdin) input += chunk;
  captured = JSON.parse(input);
}
assert.deepEqual(
  captured.map(({ id, source }) => ({ id, source })),
  sourceCases,
);
const observations = [];
for (const row of captured) {
  const parsed = compiler.parse(row.source, { filename: "Cam.vue" });
  assert.deepEqual(parsed.errors, [], row.id);
  const reference = compiler.compileScript(parsed.descriptor, {
    id: "Cam",
    inlineTemplate: true,
    isProd: true,
  });
  const renders = [];
  for (const [name, code] of [
    ["vue", reference.content],
    ["vize", row.result.code],
    ...(row.mapped ? [["vize-mapped", row.mapped.code]] : []),
  ]) {
    const loadable = code
      .replaceAll('from "vue"', `from ${JSON.stringify(vueUrl)}`)
      .replaceAll("from 'vue'", `from ${JSON.stringify(vueUrl)}`)
      .replaceAll('from "./vector3.mjs"', `from ${JSON.stringify(vectorsUrl)}`)
      .replaceAll("from './vector3.mjs'", `from ${JSON.stringify(vectorsUrl)}`);
    const { default: component } = await import(dataUrl(loadable));
    const render = component.setup({}, { expose() {} });
    assert.equal(typeof render, "function", `${row.id}/${name}`);
    const vnode = render({}, []);
    const value = JSON.parse(JSON.stringify(vnode.props["look-at"]));
    assert.deepEqual(value, { x: 0, y: 0.75, z: 0 }, `${row.id}/${name}`);
    renders.push({ name, value });
  }
  observations.push({
    id: row.id,
    source: row.source,
    code: row.result.code,
    referenceCode: reference.content,
    sourceMap: row.mapped?.map,
    renders,
  });
}
// Prove the original precedence spelling is rejected by the same runtime.
const { Vector3 } = await import(vectorsUrl);
assert.throws(() => new runtime.unref(Vector3)(0, 0.75, 0), {
  name: "TypeError",
  message: "Class constructor Vector3 cannot be invoked without 'new'",
});
console.log(
  JSON.stringify({
    schema: "vize.new-unref-constructor-runtime",
    vue: compiler.version,
    native: provider ? require("@vizejs/native/package.json").version : "source-capture",
    observations,
  }),
);
