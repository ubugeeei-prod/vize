import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const fixtureRoot = new URL("../../davinci/vize_l4/tests/fixtures/vue-setup-let/", import.meta.url);
const metadata = JSON.parse(fs.readFileSync(new URL("capture.json", fixtureRoot), "utf8"));
const fixtures = metadata.fixtureFiles.map((file: string) =>
  JSON.parse(fs.readFileSync(new URL(file, fixtureRoot), "utf8")),
);
const hash = (value: string) => createHash("sha256").update(value).digest("hex");
const fromUi = createRequire(
  process.env.VIZE_TEST_VUE_PACKAGE ?? new URL("../../npm/ui/package.json", import.meta.url),
);
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const sfc = fromVue("@vue/compiler-sfc");
const dom = fromVue("@vue/compiler-dom");
const vue = fromVue("vue");
const url = (source: string) => `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const runtimeUrl = url(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    ["toDisplayString", "openBlock", "createElementBlock", "normalizeClass", "normalizeStyle"]
      .map((name) => `export const ${name}=runtime.${name};`).join("\n"),
);

test("six original setup-let references preserve complete pinned modules, options and maps", () => {
  assert.equal(metadata.schema, "vize.native-vue-setup-let-reference");
  assert.equal(metadata.compiler.version, "3.5.35");
  assert.equal(fixtures.length, 6);
  assert.equal(new Set(fixtures.map((fixture: any) => fixture.id)).size, 6);
  for (const name of ["vue", "@vue/compiler-dom", "@vue/compiler-sfc"])
    assert.equal(fromVue(`${name}/package.json`).version, "3.5.35");
  for (const fixture of fixtures) {
    assert.equal(hash(fixture.source), fixture.sourceSha256);
    assert.equal(hash(fixture.template), fixture.templateSha256);
    assert.equal(hash(fixture.code), fixture.codeSha256);
    assert.equal(hash(JSON.stringify(fixture.options)), fixture.optionsSha256);
    assert.equal(hash(JSON.stringify(fixture.referenceMap)), fixture.referenceMapSha256);
    const parsed = sfc.parse(fixture.source, { filename: metadata.filename });
    assert.deepEqual(parsed.errors, []);
    for (const block of [parsed.descriptor.scriptSetup, parsed.descriptor.template]) {
      const key = block === parsed.descriptor.scriptSetup ? "script" : "template";
      assert.deepEqual({
        utf16Start: block.loc.start.offset, utf16End: block.loc.end.offset,
        authoredByteStart: Buffer.byteLength(fixture.source.slice(0, block.loc.start.offset)),
        authoredByteEnd: Buffer.byteLength(fixture.source.slice(0, block.loc.end.offset)),
      }, fixture.windows[key]);
    }
    const script = sfc.compileScript(parsed.descriptor, fixture.compiledScript.options);
    assert.deepEqual(script.bindings, fixture.compiledScript.bindings);
    assert.equal(script.content, fixture.compiledScript.code);
    // Compare the complete serialized map; compiler-sfc returns a SourceMap
    // instance, while the immutable fixture retains its full JSON wire value.
    assert.equal(JSON.stringify(script.map), JSON.stringify(fixture.compiledScript.rawMap));
    const options = { ...metadata.templateOptions, bindingMetadata: script.bindings };
    assert.deepEqual(options, fixture.options);
    for (let repeated = 0; repeated < 2; repeated++) {
      const measured = dom.compile(fixture.template, options);
      assert.equal(measured.code, fixture.code, fixture.id);
      assert.deepEqual(measured.map, fixture.referenceMap, fixture.id);
    }
    assert.deepEqual(fixture.referenceMap.names, []);
  }
});

function shape(node: any): any {
  assert(vue.isVNode(node));
  return { type: node.type, props: node.props, children: node.children, patchFlag: node.patchFlag,
    dynamicProps: node.dynamicProps, dynamicChildren: node.dynamicChildren?.map(shape) ?? null,
    key: node.key, ref: node.ref, shapeFlag: node.shapeFlag };
}

async function execute(fixture: any, code: string) {
  const component = (await import(url(fixture.compiledScript.code))).default;
  const state = vue.proxyRefs(component.setup(Object.create(null), { expose() {} }));
  const render = (await import(url(code.replace('from "vue"', `from ${JSON.stringify(runtimeUrl)}`)))).render;
  const outcomes = [];
  for (const reference of fixture.executions) {
    if (reference.context === "actual-setup-mutated")
      for (const [name, value] of Object.entries(fixture.update)) state[name] = value;
    const attemptedForeignReads: Array<{ slot: string; key: string }> = [];
    const forbidden = (slot: string) => new Proxy(Object.create(null), { get(_, key) {
      attemptedForeignReads.push({ slot, key: String(key) });
      throw Error(`unexpected ${slot}.${String(key)}`);
    } });
    const node = render(forbidden("context"), [], forbidden("props"), state, forbidden("data"), forbidden("options"));
    assert.deepEqual(shape(node), reference.vnode);
    assert.deepEqual(JSON.parse(JSON.stringify(node)), reference.rawVNode);
    assert.deepEqual(attemptedForeignReads, []);
    outcomes.push({ context: reference.context, vnode: shape(node), attemptedForeignReads });
  }
  return outcomes;
}

for (const fixture of fixtures) {
  test(`${fixture.id} complete module reads actual setup state before and after mutation`, async () => {
    await execute(fixture, fixture.code);
  });
}

test("actual native modules and typed refusals retain the six-source denominator", async () => {
  const capturePath = process.env.VIZE_L4_VUE_READ_NATIVE_CAPTURE;
  if (!capturePath) return;
  const captures = JSON.parse(fs.readFileSync(capturePath, "utf8"));
  assert.equal(captures.length, 6);
  assert.deepEqual(captures.map((row: any) => row.id), fixtures.map((row: any) => row.id));
  const runtime = [];
  let complete = 0;
  let refused = 0;
  for (const [index, fixture] of fixtures.entries()) {
    const capture = captures[index];
    assert.equal(capture.source, fixture.source);
    if (["let-var-prop-binary", "unicode-let-var"].includes(fixture.id)) {
      assert.equal(capture.outcome, "typed_refusal");
      assert.equal(capture.kind, "UncertifiedExpressionSpelling");
      assert.equal(Object.hasOwn(capture, "code"), false);
      refused++;
      continue;
    }
    assert.equal(capture.outcome, "complete_module");
    assert.equal(capture.code, fixture.code);
    assert.deepEqual(capture.map.sourcesContent, [fixture.source]);
    assert.notDeepEqual(capture.map.names, fixture.referenceMap.names);
    runtime.push({ id: fixture.id, codeSha256: hash(capture.code),
      mapSha256: hash(JSON.stringify(capture.map)), executions: await execute(fixture, capture.code) });
    complete++;
  }
  assert.deepEqual({ complete, refused }, { complete: 4, refused: 2 });
  if (process.env.VIZE_L4_VUE_READ_RUNTIME_CAPTURE)
    fs.writeFileSync(process.env.VIZE_L4_VUE_READ_RUNTIME_CAPTURE, JSON.stringify({
      sourceCapture: { path: capturePath, sha256: hash(fs.readFileSync(capturePath, "utf8")) },
      completeModules: complete, typedRefusals: refused, nativeRuntimeContexts: 8,
      completeUpstreamMapParity: false, runtime,
    }, null, 2) + "\n");
});
