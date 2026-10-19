//! Independent pinned Babel JSX transform and actual Vue renderer reference.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

export interface Fixture {
  source: string;
  modules?: Readonly<Record<string, string>>;
  arguments: readonly (readonly unknown[])[];
  registered?: boolean;
  referenceCode?: string;
  referenceMap?: unknown;
  runtime?: unknown;
}
export interface Pack {
  babelVersion: string;
  pluginVersion: string;
  vueVersion: string;
  pluginOptions: unknown;
  fixtures: Fixture[];
  refusals: Fixture[];
}
interface BabelCore {
  transformSync(source: string, options: object): { code?: string | null; map?: unknown } | null;
}
interface HostNode {
  kind: "element" | "text" | "comment";
  tag?: string;
  text?: string;
  props: Record<string, unknown>;
  children: HostNode[];
  parent: HostNode | null;
}
interface VNode {
  readonly type: unknown;
  readonly props: unknown;
  readonly key: unknown;
  readonly ref: unknown;
  readonly shapeFlag: unknown;
  readonly patchFlag: unknown;
  readonly dynamicProps: unknown;
  readonly dynamicChildren: unknown;
  readonly children: unknown;
}
interface App {
  component(name: string, definition: { render: () => VNode }): App;
  mount(root: HostNode): unknown;
  unmount(): void;
}
interface Renderer {
  render(vnode: unknown, root: HostNode): void;
  createApp(component: { render: unknown }): App;
}
interface RendererOptions {
  createElement(tag: string): HostNode;
  createText(text: string): HostNode;
  createComment(text: string): HostNode;
  setText(child: HostNode, text: string): void;
  setElementText(parent: HostNode, text: string): void;
  parentNode(child: HostNode): HostNode | null;
  nextSibling(child: HostNode): HostNode | null;
  patchProp(child: HostNode, key: string, previous: unknown, next: unknown): void;
  insert(child: HostNode, parent: HostNode, anchor?: HostNode | null): void;
  remove(child: HostNode): void;
}
interface VueRuntime {
  readonly Text: unknown;
  createRenderer(options: RendererOptions): Renderer;
  isVNode(value: unknown): value is VNode;
  createVNode(type: string, props: null, children: string): VNode;
}

export const fixtureUrl = new URL(
  "../../../../davinci/vize_l4/tests/fixtures/jsx-js-module-vue-3.5.35.json",
  import.meta.url,
);
const fromRoot = createRequire(new URL("../../../../package.json", import.meta.url));
const fromUi = createRequire(new URL("../../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const babel = fromRoot("@babel/core") as BabelCore;
const loadedPlugin: unknown = fromRoot("@vue/babel-plugin-jsx");
const plugin =
  typeof loadedPlugin === "object" && loadedPlugin !== null && "default" in loadedPlugin
    ? loadedPlugin.default
    : loadedPlugin;
const vue = fromVue("vue") as VueRuntime;
assert.equal(typeof babel.transformSync, "function");
assert.equal(typeof vue.createRenderer, "function");
const version = (require: NodeJS.Require, name: string): unknown =>
  (require(`${name}/package.json`) as { version?: unknown }).version;
const url = (code: string): string =>
  `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
const runtimeUrl = url(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    ["createVNode", "createTextVNode", "resolveComponent"]
      .map((name) => `export const ${name}=runtime.${name};`)
      .join("\n"),
);

export function readPack(url: URL = fixtureUrl): Pack {
  return JSON.parse(fs.readFileSync(url, "utf8")) as Pack;
}

export function checkVersions(pack: Pack): void {
  assert.equal(version(fromRoot, "@babel/core"), pack.babelVersion);
  assert.equal(version(fromRoot, "@vue/babel-plugin-jsx"), pack.pluginVersion);
  assert.equal(version(fromVue, "vue"), pack.vueVersion);
}

export function reference(fixture: Fixture, pack: Pack): { code: string; map: unknown } {
  const result = babel.transformSync(fixture.source, {
    babelrc: false,
    configFile: false,
    filename: "Native雪🌸.jsx",
    sourceFileName: "Native雪🌸.jsx",
    sourceMaps: true,
    plugins: [[plugin, pack.pluginOptions]],
  });
  assert(result && typeof result.code === "string", "Babel produced no module");
  // Compare the whole serialized wire map; undefined object properties have
  // no JSON representation, matching the immutable fixture and map consumers.
  return { code: result.code, map: JSON.parse(JSON.stringify(result.map)) as unknown };
}

function hostRenderer(): Renderer {
  const node = (kind: HostNode["kind"], fields: Partial<HostNode>): HostNode => ({
    kind,
    props: {},
    children: [],
    ...fields,
    parent: null,
  });
  const detach = (child: HostNode): void => {
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

type Shape =
  | { kind: HostNode["kind"]; tag?: string; props: Record<string, unknown>; children: Shape[] }
  | { kind: HostNode["kind"]; text?: string };

function shape(node: HostNode): Shape {
  return node.kind === "element"
    ? { kind: node.kind, tag: node.tag, props: node.props, children: node.children.map(shape) }
    : { kind: node.kind, text: node.text };
}

function vnodeShape(node: unknown): unknown {
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

type RenderModule = Readonly<Record<string, unknown>>;

async function loadModule(code: string, fixture: Fixture): Promise<RenderModule> {
  // Only the import address is adapted; the complete generated module executes.
  let addressed = code.replace('from "vue"', `from ${JSON.stringify(runtimeUrl)}`);
  for (const [name, external] of Object.entries(fixture.modules ?? {})) {
    const moduleUrl = url(external.replace('from "vue"', `from ${JSON.stringify(runtimeUrl)}`));
    addressed = addressed.replace(
      `from ${JSON.stringify(name)}`,
      `from ${JSON.stringify(moduleUrl)}`,
    );
  }
  return (await import(url(addressed))) as RenderModule;
}

const emptyRoot = (): HostNode => ({
  kind: "element",
  tag: "root",
  props: {},
  children: [],
  parent: null,
});

async function executeMeasured(
  code: string,
  fixture: Fixture,
  snapshot: boolean,
): Promise<{ exports: string[]; results: unknown[] }> {
  const retain = <T>(value: T): T => (snapshot ? structuredClone(value) : value);
  const module = await loadModule(code, fixture);
  const render = module.render ?? module.default;
  assert(typeof render === "function");
  const renderer = hostRenderer();
  const root = emptyRoot();
  const results: unknown[] = [];
  // Re-render into the same actual host, exercising changed real parameter reads.
  for (const args of fixture.arguments) {
    const vnode: unknown = render(...args);
    assert(vue.isVNode(vnode));
    const originalVNode = retain(vnodeShape(vnode));
    renderer.render(vnode, root);
    results.push(retain({ arguments: args, vnode: originalVNode, tree: root.children.map(shape) }));
  }
  if (typeof module.first === "function") {
    const first: unknown = module.first();
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

export async function execute(code: string, fixture: Fixture): Promise<unknown> {
  return executeMeasured(code, fixture, false);
}

// New changing-prop laws retain each actual pre-mount VNode and host state.
// Existing frozen callers keep their original observation semantics.
export async function executeSnapshots(code: string, fixture: Fixture): Promise<unknown> {
  return executeMeasured(code, fixture, true);
}

export async function executeRegistered(code: string, fixture: Fixture): Promise<Shape[]> {
  const module = await loadModule(code, fixture);
  const renderer = hostRenderer();
  const root = emptyRoot();
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
