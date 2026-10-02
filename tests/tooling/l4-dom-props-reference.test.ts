import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const pack = JSON.parse(fs.readFileSync(new URL(
  "../../davinci/vize_l4/tests/fixtures/dom-props-vue-3.5.35.json", import.meta.url,
), "utf8"));
const hash = (text: string) => createHash("sha256").update(text).digest("hex");
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const compiler = fromVue("@vue/compiler-dom");
const vue = fromVue("vue");
const url = (text: string) => `data:text/javascript;base64,${Buffer.from(text).toString("base64")}`;
const runtimeUrl = url(
  `import vue from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n`
    + ["openBlock", "createElementBlock"].map(name => `export const ${name} = vue.${name};`).join("\n"),
);

test("fourteen retained-root property references keep complete pinned modules and raw maps", () => {
  assert.equal(pack.schema, "vize.native-dom-props-reference");
  assert.equal(pack.version, 1);
  assert.deepEqual(pack.compiler, { name: "@vue/compiler-dom", version: "3.5.35" });
  assert.equal(fromVue("@vue/compiler-dom/package.json").version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.deepEqual(pack.options, {
    mode: "module", prefixIdentifiers: true, hoistStatic: false, cacheHandlers: false,
    comments: true, filename: "PropsDom.vue", sourceMap: true, bindingMetadata: {},
  });
  assert.equal(pack.fixtures.length, 14);
  assert.equal(new Set(pack.fixtures.map((f: any) => f.id)).size, 14);
  assert.equal(pack.fixtures.filter((f: any) => f.family === "bare").length, 7);
  assert.equal(pack.fixtures.filter((f: any) => f.family === "file").length, 7);
  for (const fixture of pack.fixtures) {
    for (const field of ["source", "template", "expression", "code"]) {
      assert.equal(hash(fixture[field]), fixture[`${field}Sha256`], `${fixture.id}: ${field}`);
    }
    assert.equal(hash(JSON.stringify(fixture.referenceMap)), fixture.referenceMapSha256, fixture.id);
    const measured = compiler.compile(fixture.template, pack.options);
    assert.equal(measured.code, fixture.code, fixture.id);
    assert.deepEqual(measured.map, fixture.referenceMap, fixture.id);
    const repeat = compiler.compile(fixture.template, pack.options);
    assert.equal(repeat.code, measured.code);
    assert.deepEqual(repeat.map, measured.map);
  }
});

for (const fixture of pack.fixtures) {
  test(`${fixture.id} complete module preserves its property value and dynamic flags`, async () => {
    const loaded = await import(url(fixture.code.replace('from "vue"', `from ${JSON.stringify(runtimeUrl)}`)));
    for (const execution of fixture.executions) {
      const forbidden = new Proxy({}, { get(_target, key) { throw Error(`unexpected ${String(key)}`); } });
      const context = fixture.family === "file" ? forbidden : { msg: execution.msg };
      const node = loaded.render(context, [], forbidden, forbidden, forbidden, forbidden);
      assert(vue.isVNode(node));
      const title = node.props.title instanceof RegExp
        ? { regexp: node.props.title.source, flags: node.props.title.flags } : node.props.title;
      assert.deepEqual({ type: node.type, title, children: node.children,
        patchFlag: node.patchFlag, key: node.key, dynamicChildren: node.dynamicChildren }, execution.expected);
    }
  });
}
