// Dev-only frozen oracle: official rc.9 syntax and independent Source Map codec.
import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { officialCompilerVapor, vueVaporVersion } from "./vue-vapor-release.mjs";

assert.equal(vueVaporVersion, "3.6.0-rc.9");
const ui = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const vue = createRequire(ui.resolve("vue-vapor-runtime/package.json"));
const { SourceMapGenerator } = createRequire(vue.resolve("@vue/compiler-sfc"))("source-map-js");
const samples = [
  ["empty", ""],
  ["text", "hello"],
  ["unicode", "雪🌸\u00a0a"],
  ["comment", "<!--雪🌸-->"],
  ["nested", "<div><span>hello</span><br><img></div>"],
  ["siblings", "<span>a</span><div>b</div>"],
  ["comment-root", "<!--a--><section>hello</section><!--b-->"],
  ["quoted-text", '<div>a"b\\c</div>'],
  ["attributes", `<div id="x" title='a"b\\c' data-probe="雪🌸" aria-label="hello"><span lang="ja">雪🌸</span></div>`],
  ["crlf-source", "<div><span>雪🌸</span></div>", "\r\n<!--prefix-->\r\n"],
  ["cr-source", "<div><span>雪🌸</span></div>", "\r<!--prefix-->\r"],
  ["unicode-lines", "<div>a\u2028b\u2029雪🌸</div>", "<!--prefix-->\u2028\u2029"],
];

function position(text, offset) {
  const lines = text.slice(0, offset).split(/\r\n|[\r\n\u2028\u2029]/u);
  return { line: lines.length, column: lines.at(-1).length };
}

function expected(template, source, prefix) {
  const ast = officialCompilerVapor.parse(template);
  const roots = ast.children;
  const nonComments = roots.filter((root) => root.type !== 3);
  const rootElement = nonComments.length === 1 && nonComments[0].type === 1 ? nonComments[0] : null;
  const hoists = [];
  const declarations = [];
  const links = [];
  const helpers = [];
  const js = (text) => JSON.stringify(text).slice(1, -1).replaceAll("\u2028", "\\u2028").replaceAll("\u2029", "\\u2029");
  const html = (text, attr = false) => js(text.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', attr ? "&quot;" : '"'));
  for (const [index, root] of roots.entries()) {
    if (root.type === 2) {
      declarations.push(`  const n${index} = _createTextNode("${js(root.content)}")`);
      links.push({ fragment: "render", generated: declarations.join("\n").length - js(root.content).length - 2, source: prefix + root.loc.start.offset });
      continue;
    }
    let literal = "";
    const anchor = (offset) => links.push({ fragment: "hoist", generated: hoists.join("").length + `const t${index} = _template("`.length + literal.length, source: prefix + offset });
    const append = (node) => {
      anchor(node.loc.start.offset);
      if (node.type === 3) { literal += `<!--${js(node.content)}-->`; return; }
      if (node.type === 2) { literal += html(node.content); return; }
      assert.equal(node.type, 1);
      literal += `<${node.tag}`;
      for (const prop of node.props) {
        assert.equal(prop.type, 6);
        literal += " ";
        anchor(prop.loc.start.offset);
        literal += `${prop.name}=\\"${html(prop.value?.content ?? "", true)}\\"`;
      }
      literal += ">";
      for (const child of node.children) append(child);
      if (!["br", "hr", "img"].includes(node.tag)) {
        anchor(node.loc.end.offset);
        literal += `</${node.tag}>`;
      }
    };
    append(root);
    hoists.push(`const t${index} = _template("${literal}", ${root === rootElement ? 3 : 2})\n`);
    declarations.push(`  const n${index} = t${index}()`);
  }
  if (hoists.length) helpers.push("template");
  if (roots.some((root) => root.type === 2)) helpers.push("createTextNode");
  const imports = helpers.length ? `import { ${helpers.map((name) => `${name} as _${name}`).join(", ")} } from "vue"\n` : "";
  const start = `${imports}${hoists.join("")}\nexport function render(_ctx) {\n`;
  const returned = roots.map((_, index) => `n${index}`).join(", ");
  const code = start + (declarations.length ? declarations.join("\n") + "\n" : "") + `  return ${roots.length === 1 ? returned : `[${returned}]`}\n}`;
  const map = new SourceMapGenerator({ file: "NativeVapor.vue" });
  map.setSourceContent("NativeVapor.vue", source);
  const anchors = links.map((link) => {
    const generated = link.generated + (link.fragment === "hoist" ? imports.length : start.length);
    map.addMapping({ source: "NativeVapor.vue", generated: position(code, generated), original: position(source, link.source) });
    return { generated: Buffer.byteLength(code.slice(0, generated)), source: Buffer.byteLength(source.slice(0, link.source)) };
  });
  const json = map.toJSON();
  json.sources = ["NativeVapor.vue"];
  json.sourcesContent = [source];
  return { code, map: json, anchors };
}

const fixtures = samples.map(([id, template, prefix = ""]) => {
  const source = `${prefix}<template>${template}</template>`;
  const compiled = officialCompilerVapor.compile(template, { mode: "module", sourceMap: true, filename: "NativeVapor.vue" });
  return { id, source, template, ...expected(template, source, prefix.length + "<template>".length), upstreamCode: compiled.code, upstreamMap: compiled.map };
});
writeFileSync(new URL("../../../davinci/vize_l4/tests/fixtures/native-vapor-vue-3.6.0-rc.9.json", import.meta.url), JSON.stringify({ version: vueVaporVersion, fixtures }, null, 2) + "\n");
