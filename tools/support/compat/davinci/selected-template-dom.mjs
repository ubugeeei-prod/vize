// Execute the actual selected-owner Rust output against the pinned Vue oracle.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const fixturePath = new URL(
  "../../../../davinci/vize_l4/tests/fixtures/selected-template-dom-vue-3.5.35.json",
  import.meta.url,
);
const fromUi = createRequire(new URL("../../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const compiler = fromVue("@vue/compiler-dom");
const runtime = fromVue("vue");
assert.equal(fromVue("@vue/compiler-dom/package.json").version, "3.5.35");
assert.equal(fromVue("vue/package.json").version, "3.5.35");
const options = {
  mode: "module",
  hoistStatic: false,
  prefixIdentifiers: true,
  comments: true,
  filename: "SelectedTemplate.vue",
  sourceMap: true,
  bindingMetadata: {},
  cacheHandlers: false,
};
// Audit the actual neutral table rather than copying a compiler tag table.
// The selected structural provider already refuses these special modes.
const tagSource = fs.readFileSync(
  new URL("../../../../davinci/vize_l0/src/dom_tag_config.rs", import.meta.url),
  "utf8",
);
const htmlTable = tagSource.match(/pub static HTML_TAGS:[\s\S]*?phf_set!\s*\{([\s\S]*?)\};/);
assert(htmlTable, "the actual configured HTML table is required");
const special = new Set(["template", "script", "style", "pre", "textarea", "title"]);
const tags = [...htmlTable[1].matchAll(/"([^"]+)"/g)].map((match) => match[1]);
assert(tags.length > 100 && new Set(tags).size === tags.length);
const componentRoles = tags.filter(
  (tag) =>
    !special.has(tag) && compiler.compile(`<${tag}/>`, options).code.includes("resolveComponent"),
);
assert.deepEqual(
  componentRoles,
  ["search"],
  "new Vue tag-role mismatches need genuine disposition",
);
const hash = (text) => createHash("sha256").update(text).digest("hex");
const url = (source) => `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const runtimeUrl = url(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    [
      "openBlock",
      "createElementBlock",
      "createElementVNode",
      "createTextVNode",
      "createCommentVNode",
      "Fragment",
    ]
      .map((name) => `export const ${name} = runtime.${name};`)
      .join("\n"),
);

async function load(code) {
  if (code.includes('from "vue"')) {
    code = code.replace('from "vue"', `from ${JSON.stringify(runtimeUrl)}`);
  }
  return import(url(code));
}

function shape(node) {
  if (node === null) return null;
  assert(runtime.isVNode(node));
  return {
    type:
      node.type === runtime.Fragment
        ? "fragment"
        : node.type === runtime.Text
          ? "text"
          : node.type === runtime.Comment
            ? "comment"
            : node.type,
    props: node.props ?? {},
    children: Array.isArray(node.children) ? node.children.map(shape) : node.children,
    patchFlag: node.patchFlag,
  };
}

if (process.argv[2] === "--write-fixture") {
  const sources = [
    [
      "ordered-headers",
      "<template><section id = 'hé there' hidden data-empty=\"\" data-count=二 aria-label='original'><input disabled/><span title=\"kept\">ok</span></section></template>",
      4,
    ],
    [
      "unicode-escape",
      '<template><p data-note=\'say "hi" \\雪\' title="line\n二" 名=値>雪🌸</p></template>',
      2,
    ],
    ["bare-empty", "<template><div hidden data-empty=\"\" title=''/></template>", 1],
    ["text-comment", "<template>hé<!--雪-->tail</template>", 3],
    ["empty", "<template></template>", 0],
    [
      "nested",
      "<template><div data-id=二><span title='one'>1</span><span title=\"two\">2</span></div></template>",
      5,
    ],
  ];
  const fixtures = [];
  for (const [id, source, nodes] of sources) {
    const template = source.slice("<template>".length, -"</template>".length);
    const reference = compiler.compile(template, options);
    const module = await load(reference.code);
    fixtures.push({
      id,
      source,
      template,
      nodes,
      sourceSha256: hash(source),
      templateSha256: hash(template),
      code: reference.code,
      codeSha256: hash(reference.code),
      referenceMap: reference.map,
      referenceMapSha256: hash(JSON.stringify(reference.map)),
      runtime: shape(module.render({}, [])),
    });
  }
  fs.writeFileSync(
    fixturePath,
    JSON.stringify(
      {
        schema: "vize.selected-template-dom-reference",
        version: 1,
        compiler: { name: "@vue/compiler-dom", version: "3.5.35" },
        options,
        fixtures,
      },
      null,
      2,
    ) + "\n",
  );
  process.exit(0);
}

assert(process.argv[2], "the actual Rust capture path is required");
const captured = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const pack = JSON.parse(fs.readFileSync(fixturePath, "utf8"));
assert.equal(pack.schema, "vize.selected-template-dom-reference");
assert.equal(pack.version, 1);
assert.deepEqual(pack.compiler, { name: "@vue/compiler-dom", version: "3.5.35" });
assert.deepEqual(pack.options, options);
assert.equal(pack.fixtures.length, 6);
assert(Array.isArray(captured) && captured.length === 6, "all six native outputs are required");
assert.equal(new Set(captured.map((item) => item.id)).size, 6);
assert.deepEqual(
  captured.map((item) => item.id),
  pack.fixtures.map((item) => item.id),
);
const forbidden = new Proxy(
  {},
  {
    get(_target, property) {
      throw new Error(`static selected render read ${String(property)}`);
    },
  },
);
for (const [index, fixture] of pack.fixtures.entries()) {
  const native = captured[index];
  assert.equal(native.source, fixture.source, fixture.id);
  assert.equal(native.nodes, fixture.nodes, fixture.id);
  assert.equal(hash(fixture.source), fixture.sourceSha256, fixture.id);
  assert.equal(hash(fixture.template), fixture.templateSha256, fixture.id);
  assert.equal(hash(fixture.code), fixture.codeSha256, fixture.id);
  assert.equal(hash(JSON.stringify(fixture.referenceMap)), fixture.referenceMapSha256, fixture.id);
  const reference = compiler.compile(fixture.template, options);
  assert.equal(reference.code, fixture.code, fixture.id);
  assert.deepEqual(reference.map, fixture.referenceMap, fixture.id);
  assert.equal(native.code, reference.code, `${fixture.id}: actual native bytes`);
  assert.deepEqual(native.map.sourcesContent, [native.source], fixture.id);
  assert.deepEqual(native.map.names, [], fixture.id);
  assert.equal(native.map.version, 3, fixture.id);
  assert.equal(typeof native.map.mappings, "string", fixture.id);
  if (fixture.nodes) assert(native.map.mappings.length > 0, fixture.id);
  const actualModule = await load(native.code);
  const referenceModule = await load(reference.code);
  for (const context of [{}, forbidden]) {
    const actual = actualModule.render(context, [], forbidden, forbidden, forbidden, forbidden);
    const expected = referenceModule.render(
      context,
      [],
      forbidden,
      forbidden,
      forbidden,
      forbidden,
    );
    assert.deepEqual(shape(actual), fixture.runtime, `${fixture.id}: captured runtime`);
    assert.deepEqual(shape(actual), shape(expected), `${fixture.id}: pinned runtime`);
    if (actual !== null) {
      assert.equal(actual.key, expected.key, fixture.id);
      assert.equal(actual.dynamicChildren?.length, expected.dynamicChildren?.length, fixture.id);
    }
  }
}
console.log(
  "selected-template DOM: six actual captured modules passed pinned byte/map/runtime laws",
);
