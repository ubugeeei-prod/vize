// Closed source/runtime boundary; no DOM host or native product is loaded here.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

export const hash = (source: string) => createHash("sha256").update(source).digest("hex");
export type ObjectValue = Record<string, unknown>;
export function object(value: unknown): asserts value is ObjectValue {
  assert(value && typeof value === "object" && !Array.isArray(value));
}
export type ModelSsrMode = "development" | "production";
export type Primitive =
  | { kind: "null" }
  | { kind: "undefined" }
  | { kind: "string"; value: string }
  | { kind: "boolean"; value: boolean }
  | { kind: "number"; value: number }
  | { kind: "bigint"; decimal: string };
export function primitive(value: unknown): Primitive {
  if (value === null) return { kind: "null" };
  if (value === undefined) return { kind: "undefined" };
  if (typeof value === "string") return { kind: "string", value };
  if (typeof value === "boolean") return { kind: "boolean", value };
  if (typeof value === "number") {
    assert(Number.isFinite(value) && !Object.is(value, -0));
    return { kind: "number", value };
  }
  assert(typeof value === "bigint", "closed primitive observation, no coercion");
  return { kind: "bigint", decimal: value.toString() };
}
export type ModelSsrFixture = {
  id: string;
  source: string;
  script: string;
  template: string;
  bindings: string[];
  binding: string;
  initial: Primitive;
  programmatic: string;
};
export function assertFixture(value: unknown): asserts value is ModelSsrFixture {
  object(value);
  for (const key of ["id", "source", "script", "template", "binding", "programmatic"])
    assert(typeof value[key] === "string" && value[key].length > 0);
  assert.deepEqual(value.bindings, [value.binding]);
  object(value.initial);
  const initial = value.initial;
  if (initial.kind === "null") assert.deepEqual(Reflect.ownKeys(initial), ["kind"]);
  else if (initial.kind === "bigint") {
    assert.deepEqual(Reflect.ownKeys(initial), ["kind", "decimal"]);
    assert(typeof initial.decimal === "string" && /^-?\d+$/.test(initial.decimal));
  } else {
    assert(initial.kind === "string" || initial.kind === "number" || initial.kind === "boolean");
    assert.deepEqual(Reflect.ownKeys(initial), ["kind", "value"]);
    assert.equal(typeof initial.value, initial.kind);
    if (initial.kind === "number") assert(Number.isFinite(initial.value));
  }
}
export type ModelSsrContext = { id: string; props: ObjectValue };
export function assertContext(value: unknown): asserts value is ModelSsrContext {
  object(value);
  assert.deepEqual(Object.keys(value), ["id", "props"]);
  assert(typeof value.id === "string" && value.id.length > 0);
  object(value.props);
}
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
export const fromSsrVue = createRequire(fromUi.resolve("vue/package.json"));
export type SsrModelComponent = {
  setup: (this: unknown, props: unknown, context: unknown) => unknown;
  ssrRender: (this: unknown, ...args: unknown[]) => unknown;
};
export type SsrModelApp = {
  config: {
    warnHandler?: (warning: string) => void;
    errorHandler?: (error: unknown) => void;
  };
};
export type SsrModelInstance = {
  type: SsrModelComponent;
  appContext: { app: SsrModelApp };
  proxy: unknown;
  props: unknown;
  setupState: ObjectValue;
  data: unknown;
  ctx: unknown;
};
export type SsrModelRuntime = {
  version: string;
  createSSRApp(component: SsrModelComponent, props: ObjectValue): SsrModelApp;
  getCurrentInstance(): SsrModelInstance | null;
};
type SsrModelRenderer = { renderToString(app: SsrModelApp, context: ObjectValue): Promise<string> };
function assertRuntime(value: unknown): asserts value is SsrModelRuntime {
  object(value);
  assert.equal(value.version, "3.5.35");
  assert.equal(typeof value.createSSRApp, "function");
  assert.equal(typeof value.getCurrentInstance, "function");
}
function assertRenderer(value: unknown): asserts value is SsrModelRenderer {
  object(value);
  assert.equal(typeof value.renderToString, "function");
}
export function openModelSsr() {
  const runtime: unknown = fromSsrVue("vue");
  const renderer: unknown = fromSsrVue("@vue/server-renderer");
  const metadata: unknown = fromSsrVue("@vue/server-renderer/package.json");
  assertRuntime(runtime);
  assertRenderer(renderer);
  object(metadata);
  assert.equal(metadata.version, "3.5.35");
  return { runtime, renderer };
}
type ImportNode = {
  type: string;
  source?: { type: string; value: string; start: number; end: number };
};
const parser = createRequire(fromSsrVue.resolve("@vue/compiler-sfc/package.json"))(
  "@babel/parser",
) as {
  parse(source: string, options: { sourceType: "module" }): { program: { body: ImportNode[] } };
};
const dataUrl = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
function runtimeUrl(name: "vue" | "@vue/server-renderer", exports: string[]) {
  return dataUrl(
    `import runtime from ${JSON.stringify(pathToFileURL(fromSsrVue.resolve(name)).href)};\n` +
      exports.map((entry) => `export const ${entry} = runtime.${entry};`).join("\n"),
  );
}
const vueUrl = runtimeUrl("vue", ["mergeProps", "defineComponent"]);
const serverUrl = runtimeUrl("@vue/server-renderer", [
  "ssrRenderAttrs",
  "ssrRenderAttr",
  "ssrGetDynamicModelProps",
]);
export function modelSsrModuleSource(code: string) {
  const imports = parser
    .parse(code, { sourceType: "module" })
    .program.body.filter((node) => node.type === "ImportDeclaration");
  assert(imports.length > 0);
  let rewritten = code;
  for (const node of imports.toReversed()) {
    assert(node.source?.type === "StringLiteral");
    const { start, end, value } = node.source;
    assert(
      Number.isInteger(start) &&
        Number.isInteger(end) &&
        start >= 0 &&
        start < end &&
        end <= code.length,
    );
    assert(value === "vue" || value === "@vue/server-renderer" || value === "vue/server-renderer");
    rewritten =
      rewritten.slice(0, start) +
      JSON.stringify(value === "vue" ? vueUrl : serverUrl) +
      rewritten.slice(end);
  }
  if (code.includes("/* from 'vue' 雪🌸 */")) assert(rewritten.includes("/* from 'vue' 雪🌸 */"));
  return rewritten;
}
function assertComponent(value: unknown): asserts value is SsrModelComponent {
  object(value);
  assert.equal(typeof value.setup, "function");
  assert.equal(typeof value.ssrRender, "function");
}
export async function loadModelSsrComponent(code: string) {
  const module: unknown = await import(dataUrl(modelSsrModuleSource(code)));
  object(module);
  const component = module.default;
  assertComponent(component);
  return component;
}
