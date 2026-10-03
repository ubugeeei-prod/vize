import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const compiler = fromVue("@vue/compiler-sfc");
const runtime = fromVue("vue");
const hash = (source: string) => createHash("sha256").update(source).digest("hex");
const dataUrl = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const runtimeUrl = dataUrl(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    [
      "defineComponent",
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
  // Each fixture reads every binding once. Getters retain two named links;
  // only mutable bindings add the two setter links.
  assert.equal(named, fixture.bindings.length * 5 - fixture.immutableBindings.length * 2);
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
async function execute(code: string, fixture: any, native = false) {
  const loaded = await import(
    dataUrl(code.replace(/from (["'])vue\1/g, `from ${JSON.stringify(runtimeUrl)}`))
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
    if (fixture.immutableBindings.includes(name)) {
      if (native) {
        assert.equal(typeof descriptor?.get, "function");
        assert.equal(typeof descriptor?.set, "undefined");
        const original = state[name];
        assert.equal(Reflect.set(state, name, Symbol("forbidden update")), false);
        assert.equal(state[name], original);
      } else {
        assert.equal(typeof descriptor?.get, "undefined");
        assert.equal(typeof descriptor?.set, "undefined");
      }
    } else {
      assert.equal(typeof descriptor?.get, "function");
      assert.equal(typeof descriptor?.set, "function");
    }
  }
  const initial = shape(app._instance.subTree);
  for (const [name, value] of Object.entries(fixture.updates)) {
    assert(fixture.bindings.includes(name));
    assert(!fixture.immutableBindings.includes(name));
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

export { compiler, runtime, fromVue, hash, checkMap, execute };
