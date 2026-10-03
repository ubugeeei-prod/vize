import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";
import { fromVue, runtime } from "./native-sfc-setup-reference.ts";

const dataUrl = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const runtimeUrl = dataUrl(
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
const hostNode = (type: string, text = "") => ({
  type,
  text,
  props: {},
  children: [] as any[],
  parent: null as any,
});
function hostShape(node: any): any {
  return {
    type: node.type,
    text: node.text,
    props: node.props,
    children: node.children.map(hostShape),
  };
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
  patchProp: (node: any, key: string, _old: unknown, value: unknown) => {
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

export async function executeOrdinary(code: string) {
  const loaded = await import(
    dataUrl(code.replace(/from (["'])vue\1/g, `from ${JSON.stringify(runtimeUrl)}`))
  );
  const component = loaded.default;
  assert.equal(typeof component, "object");
  assert.equal(component.setup, undefined);
  assert.equal(typeof component.render, "function");
  const originalRender = component.render;
  let renders = 0;
  component.render = function (this: any, ...args: any[]) {
    renders += 1;
    return originalRender.apply(this, args);
  };
  const root = hostNode("root");
  const warnings: string[] = [];
  const app = renderer.createApp(component);
  app.config.warnHandler = (warning: string) => warnings.push(warning);
  app.mount(root);
  assert.equal(renders, 1);
  assert.deepEqual(Object.keys(app._instance.setupState), []);
  const initial = shape(app._instance.subTree),
    mounted = hostShape(root);
  app._instance.proxy.$forceUpdate();
  await runtime.nextTick();
  assert.equal(renders, 2);
  assert.deepEqual(warnings, []);
  assert.equal(component.setup, undefined);
  const updated = shape(app._instance.subTree);
  assert.deepEqual(hostShape(root), mounted);
  app.unmount();
  assert.deepEqual(root.children, []);
  return {
    vnodes: [initial, updated],
    mounted,
    nativeRenderInvocations: renders,
    defaultSetupAbsent: true,
  };
}

export function checkOrdinaryMap(fixture: any) {
  const map = fixture.nativeMap;
  assert.equal(map.version, 3);
  assert.equal(map.file, "Ordinary雪🌸.vue");
  assert.deepEqual(map.sources, [map.file]);
  assert.deepEqual(map.sourcesContent, [fixture.source]);
  assert.deepEqual(map.names, []);
  const generated = fixture.code.split("\n"),
    authored = fixture.source.split("\n");
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  let sourceIndex = 0,
    sourceLine = 0,
    sourceColumn = 0,
    segments = 0;
  for (const [lineIndex, line] of map.mappings.split(";").entries()) {
    let column = 0;
    for (const segment of line.split(",").filter(Boolean)) {
      const fields: number[] = [];
      let value = 0,
        shift = 0;
      for (const character of segment) {
        const digit = alphabet.indexOf(character);
        assert(digit >= 0);
        value += (digit & 31) * 2 ** shift;
        if (digit & 32) shift += 5;
        else {
          fields.push(value & 1 ? -(value >> 1) : value >> 1);
          value = 0;
          shift = 0;
        }
      }
      assert.equal(shift, 0);
      assert.equal(fields.length, 4);
      column += fields[0];
      sourceIndex += fields[1];
      sourceLine += fields[2];
      sourceColumn += fields[3];
      assert.equal(sourceIndex, 0);
      assert(generated[lineIndex] !== undefined && authored[sourceLine] !== undefined);
      assert(column >= 0 && column <= generated[lineIndex].length);
      assert(sourceColumn >= 0 && sourceColumn <= authored[sourceLine].length);
      segments += 1;
    }
  }
  assert(segments > 0);
}
