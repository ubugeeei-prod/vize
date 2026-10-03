//! Independent pinned Babel JSX transform and actual Vue renderer reference.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

export const fixtureUrl = new URL(
  "../../../../davinci/vize_l4/tests/fixtures/jsx-js-module-vue-3.5.35.json",
  import.meta.url,
);
const fromRoot = createRequire(new URL("../../../../package.json", import.meta.url));
const fromUi = createRequire(new URL("../../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const babel = fromRoot("@babel/core");
const loadedPlugin = fromRoot("@vue/babel-plugin-jsx");
const plugin = loadedPlugin.default ?? loadedPlugin;
const vue = fromVue("vue");
const url = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
const runtimeUrl = url(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    ["createVNode", "createTextVNode", "resolveComponent"]
      .map((name) => `export const ${name}=runtime.${name};`)
      .join("\n"),
);

export function readPack(url = fixtureUrl) {
  return JSON.parse(fs.readFileSync(url, "utf8"));
}

export function checkVersions(pack) {
  assert.equal(fromRoot("@babel/core/package.json").version, pack.babelVersion);
  assert.equal(fromRoot("@vue/babel-plugin-jsx/package.json").version, pack.pluginVersion);
  assert.equal(fromVue("vue/package.json").version, pack.vueVersion);
}

export function reference(fixture, pack) {
  const result = babel.transformSync(fixture.source, {
    babelrc: false,
    configFile: false,
    filename: "Native雪🌸.jsx",
    sourceFileName: "Native雪🌸.jsx",
    sourceMaps: true,
    plugins: [[plugin, pack.pluginOptions]],
  });
  // Compare the whole serialized wire map; undefined object properties have
  // no JSON representation, matching the immutable fixture and map consumers.
  return { code: result.code, map: JSON.parse(JSON.stringify(result.map)) };
}

function hostRenderer() {
  const node = (kind, fields) => ({ kind, ...fields, parent: null });
  const detach = (child) => {
    if (child.parent) {
      const siblings = child.parent.children;
      siblings.splice(siblings.indexOf(child), 1);
      child.parent = null;
    }
  };
  return vue.createRenderer({
    createElement: (tag) => node("element", { tag, props: {}, children: [] }),
    createText: (text) => node("text", { text }),
    createComment: (text) => node("comment", { text }),
    setText: (child, text) => {
      child.text = text;
    },
    setElementText: (parent, text) => {
      parent.children = text ? [node("text", { text })] : [];
      for (const child of parent.children) child.parent = parent;
    },
    parentNode: (child) => child.parent,
    nextSibling: (child) =>
      child.parent?.children[child.parent.children.indexOf(child) + 1] ?? null,
    patchProp: (child, key, _previous, next) => {
      if (next == null) delete child.props[key];
      else child.props[key] = next;
    },
    insert: (child, parent, anchor = null) => {
      detach(child);
      const index = anchor === null ? parent.children.length : parent.children.indexOf(anchor);
      assert(index >= 0);
      parent.children.splice(index, 0, child);
      child.parent = parent;
    },
    remove: detach,
  });
}

function shape(node) {
  return node.kind === "element"
    ? { kind: node.kind, tag: node.tag, props: node.props, children: node.children.map(shape) }
    : { kind: node.kind, text: node.text };
}

function vnodeShape(node) {
  if (!vue.isVNode(node)) return node;
  const type =
    node.type === vue.Text
      ? "vue-text"
      : typeof node.type === "function"
        ? `function:${node.type.name}`
        : node.type;
  return {
    type,
    props: node.props,
    key: node.key,
    ref: node.ref,
    shapeFlag: node.shapeFlag,
    patchFlag: node.patchFlag,
    dynamicProps: node.dynamicProps,
    dynamicChildren: node.dynamicChildren,
    children: Array.isArray(node.children) ? node.children.map(vnodeShape) : node.children,
  };
}

async function loadModule(code, fixture) {
  // Only the import address is adapted; the complete generated module executes.
  let addressed = code.replace('from "vue"', `from ${JSON.stringify(runtimeUrl)}`);
  for (const [name, external] of Object.entries(fixture.modules ?? {})) {
    const moduleUrl = url(external.replace('from "vue"', `from ${JSON.stringify(runtimeUrl)}`));
    addressed = addressed.replace(
      `from ${JSON.stringify(name)}`,
      `from ${JSON.stringify(moduleUrl)}`,
    );
  }
  return import(url(addressed));
}

async function executeMeasured(code, fixture, snapshot) {
  const retain = (value) => (snapshot ? structuredClone(value) : value);
  const module = await loadModule(code, fixture);
  const render = module.render ?? module.default;
  assert.equal(typeof render, "function");
  const renderer = hostRenderer();
  const root = { kind: "element", tag: "root", props: {}, children: [], parent: null };
  const results = [];
  // Re-render into the same actual host, exercising changed real parameter reads.
  for (const args of fixture.arguments) {
    const vnode = render(...args);
    assert(vue.isVNode(vnode));
    const originalVNode = retain(vnodeShape(vnode));
    renderer.render(vnode, root);
    results.push(retain({ arguments: args, vnode: originalVNode, tree: root.children.map(shape) }));
  }
  if (typeof module.first === "function") {
    const first = module.first();
    const originalVNode = retain(vnodeShape(first));
    renderer.render(first, root);
    results.push(
      retain({
        export: "first",
        arguments: [],
        vnode: originalVNode,
        tree: root.children.map(shape),
      }),
    );
  }
  renderer.render(null, root);
  assert.deepEqual(root.children, []);
  return { exports: Object.keys(module), results };
}

export async function execute(code, fixture) {
  return executeMeasured(code, fixture, false);
}

// New changing-prop laws retain each actual pre-mount VNode and host state.
// Existing frozen callers keep their original observation semantics.
export async function executeSnapshots(code, fixture) {
  return executeMeasured(code, fixture, true);
}

export async function executeRegistered(code, fixture) {
  const module = await loadModule(code, fixture);
  const renderer = hostRenderer();
  const root = { kind: "element", tag: "root", props: {}, children: [], parent: null };
  const app = renderer.createApp({ render: module.render });
  app.component("search", { render: () => vue.createVNode("b", null, "registered") });
  app.mount(root);
  const tree = root.children.map(shape);
  app.unmount();
  assert.deepEqual(root.children, []);
  return tree;
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href &&
  process.argv.includes("--write")
) {
  const pack = readPack();
  checkVersions(pack);
  for (const fixture of pack.fixtures) {
    const measured = reference(fixture, pack);
    fixture.referenceCode = measured.code;
    fixture.referenceMap = measured.map;
    fixture.runtime = await execute(measured.code, fixture);
  }
  for (const fixture of pack.refusals) {
    const measured = reference(fixture, pack);
    fixture.referenceCode = measured.code;
    fixture.referenceMap = measured.map;
    fixture.runtime = fixture.registered
      ? await executeRegistered(measured.code, fixture)
      : await execute(measured.code, fixture);
  }
  fs.writeFileSync(fixtureUrl, JSON.stringify(pack, null, 2) + "\n");
}
