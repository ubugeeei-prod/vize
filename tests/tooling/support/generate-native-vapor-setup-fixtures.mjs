// Independent complete native spelling/map model; stock code is separate evidence.
import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { compileVaporSetupSfcReference } from "./native-vapor-setup-oracle.mjs";

const ui = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const vue = createRequire(ui.resolve("vue-vapor-runtime/package.json"));
const { SourceMapGenerator } = createRequire(vue.resolve("@vue/compiler-sfc"))("source-map-js");
const filename = "NativeVaporSetup.vue";
const bytes = (text) => Buffer.byteLength(text);
const position = (text, offset) => {
  const lines = Buffer.from(text)
    .subarray(0, offset)
    .toString("utf8")
    .split(/\r\n|[\r\n\u2028\u2029]/u);
  return { line: lines.length, column: lines.at(-1).length };
};
const definitions = [
  { id: "mutable-js", script: "let count=1", expression: "count", name: "count", mutable: true },
  {
    id: "var-unicode-crlf",
    prefix: "<!--雪🌸-->\r\n",
    script: "/* import x from 'vue' */\r\nvar 雪='🌸'",
    expression: "雪",
    name: "雪",
    mutable: true,
  },
  { id: "const-boolean", script: "const enabled=true", expression: "enabled", name: "enabled" },
  { id: "const-lf", script: String.raw`const count='a\nb'`, expression: "count", name: "count" },
  {
    id: "const-valid-pair",
    script: String.raw`const count='a\ud83c\udf38b'`,
    expression: "count",
    name: "count",
  },
  {
    id: "const-replacement",
    script: String.raw`const count='a\ufffdb'`,
    expression: "count",
    name: "count",
  },
  {
    id: "const-ts-number",
    ts: true,
    script: "const count:number=1",
    annotations: [":number"],
    expression: "count",
    name: "count",
  },
  {
    id: "let-ts-unused",
    ts: true,
    script: "const unused:string='unused'; let count:number=2",
    annotations: [":string", ":number"],
    expression: "count",
    name: "count",
    mutable: true,
  },
  {
    id: "var-ts-boolean",
    ts: true,
    script: "var enabled:boolean=false",
    annotations: [":boolean"],
    expression: "enabled",
    name: "enabled",
    mutable: true,
  },
  { id: "scalar-number", script: "const unused=1", expression: "123" },
  { id: "scalar-boolean", script: "const unused=1", expression: "false" },
  { id: "scalar-null", script: "const unused=1", expression: "null" },
  { id: "scalar-bigint", script: "const unused=1", expression: "12n" },
  { id: "scalar-string", script: "const unused=1", expression: "'a<雪🌸'" },
  { id: "scalar-empty-string", script: "const unused=1", expression: "''" },
  {
    id: "unused-static",
    script: "const unused=1",
    template: "<div><span>static</span><!--雪🌸--></div>",
    html: "<div><span>static</span><!--雪🌸--></div>",
    root: true,
    staticAnchors: [
      [0, 0],
      [5, 5],
      [11, 11],
      [17, 24],
      [24, 24],
      [
        bytes("<div><span>static</span><!--雪🌸-->"),
        bytes("<div><span>static</span><!--雪🌸--></div>"),
      ],
    ],
  },
  {
    id: "module-comment",
    script: "const unused='import x';",
    template: "<!--import { template as _template } from vue-->",
    html: "<!--import { template as _template } from vue-->",
    staticAnchors: [[0, 0]],
  },
];

const fixtures = [];
for (const fixture of definitions) {
  const scriptTag = fixture.ts ? '<script setup lang="ts">' : "<script setup>";
  const template = fixture.template ?? `{{${fixture.expression}}}`;
  const source =
    (fixture.prefix ?? "") +
    scriptTag +
    fixture.script +
    "</script><template>" +
    template +
    "</template>";
  const scriptStart = bytes((fixture.prefix ?? "") + scriptTag);
  const templateStart = bytes(source.slice(0, source.indexOf("<template>") + "<template>".length));
  const helpers = ["template", "defineVaporComponent"];
  if (fixture.expression !== undefined) {
    if (fixture.mutable) helpers.push("renderEffect", "unref");
    helpers.push("setText", "toDisplayString");
  }
  const preamble = `import { ${helpers.map((name) => `${name} as _${name}`).join(", ")} } from "vue"\n`;
  const prelude =
    fixture.expression !== undefined
      ? 'const t0 = _template(" ")\n'
      : `const t0 = _template(${JSON.stringify(fixture.html)}, ${fixture.root ? 3 : 2})\n`;
  let code = preamble + prelude + ";\n";
  const links = (fixture.staticAnchors ?? []).map(([generated, authored]) => ({
    generated: bytes(preamble + 'const t0 = _template("') + generated,
    generatedEnd: bytes(preamble + 'const t0 = _template("') + generated,
    source: templateStart + authored,
    sourceEnd: templateStart + authored,
    name: null,
  }));
  code += "const _sfc_main = _defineVaporComponent({\n  __multiRoot: false,\n  setup(__props) {\n";
  const copy = (text, start, name = null, end = start + bytes(text)) => {
    links.push({
      generated: bytes(code),
      generatedEnd: bytes(code) + bytes(text),
      source: start,
      sourceEnd: end,
      name,
    });
    code += text;
  };
  let cursor = 0;
  for (const annotation of fixture.annotations ?? []) {
    const start = fixture.script.indexOf(annotation, cursor);
    assert.ok(start >= cursor);
    if (start > cursor)
      copy(
        fixture.script.slice(cursor, start),
        scriptStart + bytes(fixture.script.slice(0, cursor)),
      );
    cursor = start + annotation.length;
  }
  copy(fixture.script.slice(cursor), scriptStart + bytes(fixture.script.slice(0, cursor)));
  code += "\n;\n\n    const n0 = t0()";
  if (fixture.expression !== undefined) {
    code +=
      "\n    " +
      (fixture.mutable ? "_renderEffect(() => " : "") +
      "_setText(n0, _toDisplayString(" +
      (fixture.mutable ? "_unref(" : "");
    copy(fixture.expression, templateStart + 2, fixture.name ?? null);
    code += "\n" + (fixture.mutable ? ")" : "") + "))" + (fixture.mutable ? ")" : "");
  }
  code += "\n    return n0\n  }\n})\nexport default _sfc_main\n";
  const map = new SourceMapGenerator({ file: filename });
  map.setSourceContent(filename, source);
  for (const link of links)
    map.addMapping({
      source: filename,
      generated: position(code, link.generated),
      original: position(source, link.source),
      ...(link.name === null ? {} : { name: link.name }),
    });
  const mapJson = map.toJSON();
  mapJson.sources = [filename];
  mapJson.sourcesContent = [source];
  fixtures.push({
    id: fixture.id,
    source,
    code,
    map: mapJson,
    links,
    reference: await compileVaporSetupSfcReference(source, fixture.id),
  });
}
writeFileSync(
  new URL(
    "../../../crates/vize_atelier_sfc/tests/fixtures/native_vapor_setup_sfc_vue_3_6_rc9.json",
    import.meta.url,
  ),
  JSON.stringify(
    {
      schema: "vize.native-vapor.primitive-setup-sfc",
      version: "3.6.0-rc.9",
      pluginVersion: "6.0.7",
      filename,
      declarationPlacement:
        "Original pure declarations stay inside native setup; independent native expectations, no stock const-hoisting/code equality claim.",
      fixtures,
    },
    null,
    2,
  ) + "\n",
);
