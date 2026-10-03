// Verify actual original-owner output; upstream raw-map equality remains unfinished.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { checkNativeMap } from "./selected-template-literals-map.mjs";

const fixturePath = new URL(
  "../../../../davinci/vize_l4/tests/fixtures/selected-template-literal-vue-3.5.35.json",
  import.meta.url,
);
const fromUi = createRequire(new URL("../../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const compiler = fromVue("@vue/compiler-dom");
const sfc = fromVue("@vue/compiler-sfc");
const runtime = fromVue("vue");
for (const name of ["vue", "@vue/compiler-dom", "@vue/compiler-sfc"]) {
  assert.equal(fromVue(name + "/package.json").version, "3.5.35");
}
assert.equal(runtime.version, "3.5.35");
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
const sources = [
  ["decimal", "<template>{{42}}</template>", 1],
  ["hex", "<template>{{0x2a}}</template>", 1],
  ["octal-modern", "<template>{{0o52}}</template>", 1],
  ["binary-modern", "<template>{{0b101010}}</template>", 1],
  ["exponent", "<template>{{4.2e1}}</template>", 1],
  ["bigint", "<template>{{42n}}</template>", 1],
  ["boolean", "<template>{{true}}</template>", 1],
  ["null", "<template>{{null}}</template>", 1],
  ["unicode-entity", "<template>{{ '雪&amp;🌸' }}</template>", 1],
  ["escaped-string", "<template>{{ '\\0' }}</template>", 1],
  ["comment-literal", "<template>{{ /*keep*/ '雪&amp;🌸' }}</template>", 1],
  ["mixed", "<template>pré{{true}}<!--keep-->{{null}}</template>", 4],
];
const hash = (text) => createHash("sha256").update(text).digest("hex");
const data = (text) => "data:text/javascript;base64," + Buffer.from(text).toString("base64");
const runtimeUrl = data(
  "import runtime from " +
    JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href) +
    ";\n" +
    [
      "toDisplayString",
      "createCommentVNode",
      "createTextVNode",
      "Fragment",
      "openBlock",
      "createElementBlock",
    ]
      .map((name) => "export const " + name + " = runtime." + name + ";")
      .join("\n"),
);
const forbidden = new Proxy(
  {},
  {
    get(_target, property) {
      throw new Error("selected literal render read " + String(property));
    },
    has(_target, property) {
      throw new Error("selected literal render queried " + String(property));
    },
    ownKeys() {
      throw new Error("selected literal render enumerated bindings");
    },
  },
);
const load = (code) =>
  import(data(code.replace('from "vue"', "from " + JSON.stringify(runtimeUrl))));
function shape(value) {
  if (!runtime.isVNode(value)) {
    assert.equal(typeof value, "string");
    return { result: "string", value };
  }
  return {
    result: "vnode",
    type:
      value.type === runtime.Fragment
        ? "fragment"
        : value.type === runtime.Text
          ? "text"
          : value.type === runtime.Comment
            ? "comment"
            : value.type,
    props: value.props ?? {},
    children: Array.isArray(value.children) ? value.children.map(shape) : value.children,
    patchFlag: value.patchFlag,
  };
}
function render(module, context) {
  return module.render(context, [], forbidden, forbidden, forbidden, forbidden);
}
function reference(source) {
  const parsed = sfc.parse(source, { filename: options.filename });
  assert.deepEqual(parsed.errors, []);
  assert(parsed.descriptor.template);
  const template = parsed.descriptor.template.content;
  return { template, ...compiler.compile(template, options) };
}
function caseFile(id) {
  return "selected-template-literal-vue-3.5.35/" + id + ".json";
}
if (process.argv[2] === "--write-fixture") {
  assert(process.argv[3], "the original primary-only packet is required");
  const primary = JSON.parse(fs.readFileSync(process.argv[3], "utf8"));
  assert.equal(primary.version, "3.5.35");
  assert.equal(primary.referenceOnly, true);
  assert.deepEqual(
    primary.cases.map((item) => item.id),
    sources.map(([id]) => id),
  );
  fs.mkdirSync(new URL("selected-template-literal-vue-3.5.35/", fixturePath), { recursive: true });
  for (const [index, [id, source, nodes]] of sources.entries()) {
    const fresh = reference(source);
    const module = await load(fresh.code);
    const observed = shape(render(module, forbidden));
    assert.deepEqual(
      primary.cases[index],
      {
        id,
        source,
        template: fresh.template,
        code: fresh.code,
        map: fresh.map,
        runtime: observed,
      },
      id,
    );
    const fixture = {
      id,
      source,
      template: fresh.template,
      nodes,
      sourceSha256: hash(source),
      templateSha256: hash(fresh.template),
      code: fresh.code,
      codeSha256: hash(fresh.code),
      referenceMap: fresh.map,
      referenceMapSha256: hash(JSON.stringify(fresh.map)),
      runtime: observed,
    };
    fs.writeFileSync(new URL(caseFile(id), fixturePath), JSON.stringify(fixture, null, 2) + "\n");
  }
  fs.writeFileSync(
    fixturePath,
    JSON.stringify(
      {
        schema: "vize.selected-template-literal-reference",
        version: 1,
        compiler: { name: "@vue/compiler-dom", version: "3.5.35" },
        options,
        nativeCapturePinned: false,
        fixtures: sources.map(([id]) => ({ id, file: caseFile(id) })),
      },
      null,
      2,
    ) + "\n",
  );
  console.log("selected literals: twelve complete primary references frozen; native output absent");
  process.exit(0);
}

const mode = process.argv[2];
assert(mode, "the actual Rust capture path or --check-reference is required");
const pack = JSON.parse(fs.readFileSync(fixturePath, "utf8"));
assert.equal(pack.schema, "vize.selected-template-literal-reference");
assert.equal(pack.version, 1);
assert.deepEqual(pack.compiler, { name: "@vue/compiler-dom", version: "3.5.35" });
assert.deepEqual(pack.options, options);
assert.equal(typeof pack.nativeCapturePinned, "boolean");
assert.deepEqual(
  pack.fixtures,
  sources.map(([id]) => ({ id, file: caseFile(id) })),
);
const captured = mode === "--check-reference" ? null : JSON.parse(fs.readFileSync(mode, "utf8"));
if (mode !== "--check-reference") {
  assert(Array.isArray(captured) && captured.length === 12, "all twelve native outputs required");
  assert.deepEqual(
    captured.map((item) => item.id),
    sources.map(([id]) => id),
  );
}
for (const [index, [id, source, nodes]] of sources.entries()) {
  const fixture = JSON.parse(
    fs.readFileSync(new URL(pack.fixtures[index].file, fixturePath), "utf8"),
  );
  assert.equal(fixture.id, id);
  assert.equal(fixture.source, source);
  assert.equal(fixture.nodes, nodes);
  for (const [field, digest] of [
    ["source", "sourceSha256"],
    ["template", "templateSha256"],
    ["code", "codeSha256"],
  ]) {
    assert.equal(hash(fixture[field]), fixture[digest], id);
  }
  assert.equal(hash(JSON.stringify(fixture.referenceMap)), fixture.referenceMapSha256, id);
  const fresh = reference(source);
  assert.equal(fresh.template, fixture.template, id);
  assert.equal(fresh.code, fixture.code, id);
  assert.deepEqual(fresh.map, fixture.referenceMap, id);
  const expectedModule = await load(fresh.code);
  const nativePinned =
    fixture.nativeMapSha256 !== undefined || fixture.nativeLinksSha256 !== undefined;
  if (pack.nativeCapturePinned || nativePinned) {
    for (const field of ["nativeMapSha256", "nativeLinksSha256"]) {
      assert.equal(typeof fixture[field], "string", id + ": actual capture freeze required");
      assert.match(fixture[field], /^[0-9a-f]{64}$/u);
    }
  }
  const native = captured?.[index];
  if (native) {
    assert.equal(native.source, source, id);
    assert.equal(hash(native.source), fixture.sourceSha256, id);
    assert.equal(native.nodes, nodes, id);
    assert.equal(native.code, fresh.code, id + ": actual native bytes");
    checkNativeMap(native, fixture, options.filename);
    if (nativePinned) {
      assert.equal(hash(JSON.stringify(native.map)), fixture.nativeMapSha256, id);
      assert.equal(hash(JSON.stringify(native.links)), fixture.nativeLinksSha256, id);
    }
  }
  const actualModule = native ? await load(native.code) : null;
  for (const context of [{}, forbidden]) {
    const expected = render(expectedModule, context);
    assert.deepEqual(shape(expected), fixture.runtime, id + ": pinned runtime");
    if (actualModule) {
      const actual = render(actualModule, context);
      assert.deepEqual(shape(actual), fixture.runtime, id + ": captured native runtime");
      assert.deepEqual(shape(actual), shape(expected), id);
      if (runtime.isVNode(actual)) {
        assert.equal(actual.key, expected.key, id);
        assert.equal(actual.dynamicChildren?.length, expected.dynamicChildren?.length, id);
      }
    }
  }
}
console.log(
  captured === null
    ? "selected literals: twelve primary byte/raw-map/runtime references verified; native output absent"
    : "selected literals: twelve actual captured byte/whole-source-map/link/runtime laws passed; upstream raw-map parity unfinished",
);
