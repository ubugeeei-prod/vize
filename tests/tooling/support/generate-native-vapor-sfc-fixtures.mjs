// Dev-only whole-component oracle. Original standalone goldens stay unchanged.
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import {
  compileVaporSfcReference,
  pluginVersion,
  compilerVersion,
} from "./native-vapor-sfc-oracle.mjs";
const ui = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const vue = createRequire(ui.resolve("vue-vapor-runtime/package.json"));
const { SourceMapGenerator } = createRequire(vue.resolve("@vue/compiler-sfc"))("source-map-js");
const standalone = JSON.parse(
  readFileSync(
    new URL(
      "../../../davinci/vize_l4/tests/fixtures/native-vapor-vue-3.6.0-rc.9.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
// Extra original source control: import-looking authored comment text must stay
// inert in the complete emitted module and in the dev-only runtime resolver.
const lookalike = "import { template as _template } from 'vue'";
const lookalikeTemplate = `<!--${lookalike}-->`;
const lookalikeSource = `<template>${lookalikeTemplate}</template>`;
const lookalikeCode =
  `import { template as _template } from "vue"\n` +
  `const t0 = _template("${lookalikeTemplate}", 2)\n\nexport function render(_ctx) {\n` +
  "  const n0 = t0()\n  return n0\n}";
standalone.fixtures.push({
  id: "module-comment",
  source: lookalikeSource,
  template: lookalikeTemplate,
  code: lookalikeCode,
  anchors: [
    {
      generated: Buffer.byteLength(
        lookalikeCode.slice(0, lookalikeCode.indexOf(lookalikeTemplate)),
      ),
      source: 10,
    },
  ],
});
const position = (text, bytes) => {
  const lines = Buffer.from(text)
    .subarray(0, bytes)
    .toString("utf8")
    .split(/\r\n|[\r\n\u2028\u2029]/u);
  return { line: lines.length, column: lines.at(-1).length };
};
const fixtures = [];
const refusedReferences = [];
for (const fixture of standalone.fixtures) {
  const reference = await compileVaporSfcReference(fixture.source);
  if (reference.multiRoot) {
    refusedReferences.push({
      id: fixture.id,
      source: fixture.source,
      template: fixture.template,
      reference,
    });
    continue;
  }
  const imports = fixture.code.startsWith("import ")
    ? fixture.code.slice(0, fixture.code.indexOf("\n") + 1)
    : "";
  const body = fixture.code.slice(imports.length);
  const marker = "\nexport function render";
  const split = body.indexOf(marker);
  assert.ok(split >= 0);
  const hoists = body.slice(0, split);
  const render = body.slice(split + "\nexport ".length);
  const names = hoists ? ["template", "defineVaporComponent"] : ["defineVaporComponent"];
  const preamble = `import { ${names.map((name) => `${name} as _${name}`).join(", ")} } from "vue"\n`;
  const code =
    preamble +
    hoists +
    (hoists ? ";\n" : "") +
    `const _sfc_main = _defineVaporComponent({ __multiRoot: ${reference.multiRoot} })\n;\n` +
    render +
    "\n_sfc_main.render = render\nexport default _sfc_main\n";
  const anchors = fixture.anchors.map((anchor) => ({
    ...anchor,
    generated: anchor.generated + Buffer.byteLength(preamble) - Buffer.byteLength(imports),
  }));
  const map = new SourceMapGenerator({ file: "NativeVapor.vue" });
  map.setSourceContent("NativeVapor.vue", fixture.source);
  for (const anchor of anchors)
    map.addMapping({
      source: "NativeVapor.vue",
      generated: position(code, anchor.generated),
      original: position(fixture.source, anchor.source),
    });
  const json = map.toJSON();
  json.sources = ["NativeVapor.vue"];
  json.sourcesContent = [fixture.source];
  fixtures.push({
    id: fixture.id,
    source: fixture.source,
    template: fixture.template,
    code,
    map: json,
    anchors,
    reference,
  });
}
writeFileSync(
  new URL(
    "../../../crates/vize_atelier_sfc/tests/fixtures/native_vapor_sfc_vue_3_6_rc9.json",
    import.meta.url,
  ),
  JSON.stringify(
    {
      schema: "vize.native-vapor.scriptless-sfc",
      version: compilerVersion,
      pluginVersion,
      targetSelection:
        "explicit Vapor target; stock plugin parse result selects descriptor.vapor without changing original source",
      fixtures,
      refusedReferences,
    },
    null,
    2,
  ) + "\n",
);
