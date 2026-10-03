import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_sfc_js_setup_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const compiler = fromVue("@vue/compiler-sfc");
const runtime = fromVue("vue");
const hash = (source: string) => createHash("sha256").update(source).digest("hex");
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
const capturePath = process.env.VIZE_NATIVE_SFC_SETUP_CAPTURE;
const captured = capturePath ? JSON.parse(fs.readFileSync(capturePath, "utf8")) : null;
const executions: unknown[] = [];

test("JS setup whole-module fixtures retain pinned source and checked binding references", () => {
  assert.equal(pack.schema, "vize.native-sfc.js-setup-dom-reference");
  assert.equal(compiler.version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.equal(pack.fixtures.length, 8);
  assert.equal(new Set(pack.fixtures.map((fixture: any) => fixture.id)).size, 8);
  for (const fixture of pack.fixtures) {
    const parsed = compiler.parse(fixture.source, { filename: "Setup雪🌸.vue" });
    assert.deepEqual(parsed.errors, []);
    assert.equal(parsed.descriptor.script, null);
    assert.equal(parsed.descriptor.scriptSetup.content, fixture.script);
    assert.equal(parsed.descriptor.template.content, fixture.template);
    const script = compiler.compileScript(parsed.descriptor, {
      id: fixture.id,
      genDefaultAs: "_sfc_main",
    });
    assert.equal(script.content, fixture.referenceScript, fixture.id);
    assert.deepEqual(Object.keys(script.bindings), fixture.bindings);
    for (const name of fixture.bindings) assert.equal(script.bindings[name], "setup-let");
    const render = compiler.compileTemplate({
      source: fixture.template,
      filename: "Setup雪🌸.vue",
      id: fixture.id,
      sourceMap: true,
      compilerOptions: {
        mode: "module",
        hoistStatic: false,
        prefixIdentifiers: true,
        comments: true,
        bindingMetadata: script.bindings,
        cacheHandlers: false,
      },
    });
    assert.deepEqual(render.errors, []);
    assert.equal(render.code, fixture.referenceRender, fixture.id);
    checkMap(fixture);
  }
});

test(
  "native setup capture binds every complete Rust result to the same original input",
  { skip: !captured },
  () => {
    assert.equal(captured.schema, "vize.native-sfc.js-setup-capture");
    assert.equal(captured.adapter, "vize_atelier_sfc::compile_native_sfc");
    assert.deepEqual(
      captured.fixtures,
      pack.fixtures.map(({ id, source, code, nativeMap, bindings }: any) => ({
        id,
        source,
        code,
        nativeMap,
        bindings,
      })),
    );
  },
);

function vlq(segment: string): number[] {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const values: number[] = [];
  let value = 0,
    shift = 0;
  for (const character of segment) {
    const digit = alphabet.indexOf(character);
    assert(digit >= 0);
    value += (digit & 31) * 2 ** shift;
    if (digit & 32) shift += 5;
    else {
      values.push(value & 1 ? -(value >> 1) : value >> 1);
      value = 0;
      shift = 0;
    }
  }
  assert.equal(shift, 0);
  return values;
}
function checkMap(fixture: any) {
  const map = fixture.nativeMap;
  assert.equal(map.version, 3);
  assert.equal(map.file, "Setup雪🌸.vue");
  assert.deepEqual(map.sources, [map.file]);
  assert.deepEqual(map.sourcesContent, [fixture.source]);
  assert.deepEqual(map.names, fixture.bindings);
  const generated = fixture.code.split("\n"),
    authored = fixture.source.split("\n");
  let sourceIndex = 0,
    sourceLine = 0,
    sourceColumn = 0,
    nameIndex = 0,
    segments = 0,
    named = 0;
  map.mappings.split(";").forEach((line: string, lineIndex: number) => {
    let column = 0;
    for (const segment of line.split(",").filter(Boolean)) {
      const fields = vlq(segment);
      assert(fields.length === 4 || fields.length === 5);
      column += fields[0];
      sourceIndex += fields[1];
      sourceLine += fields[2];
      sourceColumn += fields[3];
      assert.equal(sourceIndex, 0);
      assert(generated[lineIndex] !== undefined && authored[sourceLine] !== undefined);
      assert(column >= 0 && column <= generated[lineIndex].length);
      assert(sourceColumn >= 0 && sourceColumn <= authored[sourceLine].length);
      if (fields.length === 5) {
        nameIndex += fields[4];
        assert(map.names[nameIndex] !== undefined);
        named += 1;
      }
      segments += 1;
    }
  });
  assert(segments > 0);
  assert(named >= fixture.bindings.length * 4);
}
function shape(node: any): any {
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
function hostNode(type: string, text = "") {
  return { type, text, children: [] as any[], parent: null as any, props: {} };
}
const renderer = runtime.createRenderer({
  createElement: (type: string) => hostNode(type),
  createText: (text: string) => hostNode("text", text),
  createComment: (text: string) => hostNode("comment", text),
  setText: (node: any, text: string) => {
    node.text = text;
  },
  setElementText: (node: any, text: string) => {
    node.text = text;
    node.children = [];
  },
  patchProp: (node: any, key: string, _previous: any, value: any) => {
    node.props[key] = value;
  },
  parentNode: (node: any) => node.parent,
  nextSibling: (node: any) => node.parent?.children[node.parent.children.indexOf(node) + 1] ?? null,
  insert: (node: any, parent: any, anchor: any) => {
    if (node.parent) {
      const index = node.parent.children.indexOf(node);
      if (index >= 0) node.parent.children.splice(index, 1);
    }
    const index = anchor ? parent.children.indexOf(anchor) : -1;
    parent.children.splice(index < 0 ? parent.children.length : index, 0, node);
    node.parent = parent;
  },
  remove: (node: any) => {
    const index = node.parent?.children.indexOf(node) ?? -1;
    if (index >= 0) node.parent.children.splice(index, 1);
    node.parent = null;
  },
});
async function execute(code: string, fixture: any) {
  const loaded = await import(
    dataUrl(code.replaceAll('from "vue"', `from ${JSON.stringify(runtimeUrl)}`))
  );
  const component = loaded.default,
    originalSetup = component.setup;
  let state: any,
    calls = 0;
  // Two real arguments preserve Vue's setup-context arity test. The recorder
  // returns only the actual generated setup result; it never supplies state.
  component.setup = (props: unknown, context: unknown) => {
    calls += 1;
    state = originalSetup(props, context);
    return state;
  };
  const warnings: string[] = [];
  const app = renderer.createApp(component);
  app.config.warnHandler = (warning: string) => warnings.push(warning);
  app.mount(hostNode("root"));
  assert.equal(calls, 1);
  assert.deepEqual(Object.keys(state), fixture.bindings);
  assert.deepEqual(Object.getOwnPropertyDescriptor(state, "__isScriptSetup"), {
    value: true,
    writable: false,
    enumerable: false,
    configurable: false,
  });
  for (const name of fixture.bindings) {
    const descriptor = Object.getOwnPropertyDescriptor(state, name);
    assert.equal(typeof descriptor?.get, "function");
    assert.equal(typeof descriptor?.set, "function");
  }
  const initial = shape(app._instance.subTree);
  for (const [name, value] of Object.entries(fixture.updates)) {
    assert(fixture.bindings.includes(name));
    state[name] = value;
    assert.equal(state[name], value);
  }
  app._instance.proxy.$forceUpdate();
  await runtime.nextTick();
  const updated = shape(app._instance.subTree);
  assert.equal(calls, 1);
  assert.deepEqual(warnings, []);
  app.unmount();
  return [initial, updated];
}

for (const fixture of pack.fixtures) {
  test(`${fixture.id} pinned complete setup and two Vue renders`, async () => {
    const module =
      fixture.referenceScript +
      "\n;\n" +
      fixture.referenceRender.replace("export function render", "function render") +
      "\n_sfc_main.render = render\nexport default _sfc_main\n";
    assert.deepEqual(await execute(module, fixture), fixture.runtime);
  });
  test(
    `${fixture.id} captured native setup owns lexical state and two Vue renders`,
    { skip: !captured },
    async () => {
      const row = captured.fixtures.find((row: any) => row.id === fixture.id);
      assert(row);
      assert.equal(row.code, fixture.code);
      assert.equal(row.source, fixture.source);
      assert.deepEqual(await execute(row.code, fixture), fixture.runtime);
      executions.push({
        id: fixture.id,
        sourceSha256: hash(row.source),
        codeSha256: hash(row.code),
        nativeSetupInvocations: 1,
        nativeRenders: 2,
      });
      if (process.env.VIZE_NATIVE_SFC_SETUP_RUNTIME_CAPTURE) {
        fs.writeFileSync(
          process.env.VIZE_NATIVE_SFC_SETUP_RUNTIME_CAPTURE,
          JSON.stringify(
            { schema: "vize.native-sfc.js-setup-runtime", runtime: "vue@3.5.35", executions },
            null,
            2,
          ) + "\n",
        );
      }
    },
  );
}
