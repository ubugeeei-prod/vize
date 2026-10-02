import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../davinci/vize_l4/tests/fixtures/native-dom-file-vue-3.5.35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const hash = (text: string) => createHash("sha256").update(text).digest("hex");
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const compiler = fromVue("@vue/compiler-dom");
const runtime = fromVue("vue");
const url = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const runtimeUrl = url(
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

test("eight genuine File DOM byte-law references retain complete pinned code and raw maps", () => {
  assert.equal(pack.schema, "vize.native-dom-file-reference");
  assert.equal(pack.version, 1);
  assert.equal(pack.compiler.name, "@vue/compiler-dom");
  assert.equal(pack.compiler.version, "3.5.35");
  assert.equal(fromVue("@vue/compiler-dom/package.json").version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.deepEqual(pack.options, {
    mode: "module",
    hoistStatic: false,
    prefixIdentifiers: true,
    comments: true,
    filename: "FileDom.vue",
    sourceMap: true,
    bindingMetadata: {},
    cacheHandlers: false,
  });
  assert.equal(pack.fixtures.length, 8);
  assert.equal(new Set(pack.fixtures.map((fixture: any) => fixture.id)).size, 8);
  for (const fixture of pack.fixtures) {
    assert.equal(hash(fixture.source), fixture.sourceSha256, fixture.id);
    assert.equal(hash(fixture.template), fixture.templateSha256, fixture.id);
    assert.equal(hash(fixture.code), fixture.codeSha256, fixture.id);
    assert.equal(
      hash(JSON.stringify(fixture.referenceMap)),
      fixture.referenceMapSha256,
      fixture.id,
    );
    const measured = compiler.compile(fixture.template, pack.options);
    assert.equal(measured.code, fixture.code, fixture.id);
    assert.deepEqual(measured.map, fixture.referenceMap, fixture.id);
    const repeated = compiler.compile(fixture.template, pack.options);
    assert.equal(repeated.code, measured.code);
    assert.deepEqual(repeated.map, measured.map);
  }
});

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
  test(`${fixture.id} complete reference module preserves authored literal semantics`, async () => {
    // The Rust genuine File pipeline compares every native byte to this code.
    // Only the dev loader resolves the import address; all render bytes remain.
    assert(fixture.code.includes('from "vue"'));
    const loaded = await import(
      url(fixture.code.replace('from "vue"', `from ${JSON.stringify(runtimeUrl)}`))
    );
    const forbiddenAccess = new Proxy(
      {},
      {
        get(_target, property) {
          throw new Error(`literal render read ${String(property)}`);
        },
      },
    );
    for (const context of [{}, forbiddenAccess]) {
      const node = loaded.render(
        context,
        [],
        forbiddenAccess,
        forbiddenAccess,
        forbiddenAccess,
        forbiddenAccess,
      );
      assert.deepEqual(shape(node), fixture.runtime);
      assert.equal(node.key, null);
      assert.deepEqual(node.dynamicChildren, []);
    }
  });
}
