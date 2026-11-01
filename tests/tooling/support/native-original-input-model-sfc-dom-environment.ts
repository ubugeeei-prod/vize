// Genuine DOM globals must exist before the actual Vue runtime-dom is loaded.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { Window } from "happy-dom";
import type { HTMLElement } from "happy-dom";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromTests = createRequire(new URL("../../package.json", import.meta.url));
const happyDomPackage = fromTests("happy-dom/package.json") as { version: unknown };
assert.equal(happyDomPackage.version, "21.0.4");
export const fromModelVue = createRequire(fromUi.resolve("vue/package.json"));
export const hash = (text: string) => createHash("sha256").update(text).digest("hex");
export const dataUrl = (text: string) =>
  `data:text/javascript;base64,${Buffer.from(text).toString("base64")}`;
export type ModelMode = "development" | "production";
export type Primitive =
  | { kind: "null" | "undefined" }
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
  assert.equal(typeof value, "bigint", "closed primitive observation, never coercion");
  return { kind: "bigint", decimal: String(value) };
}
export function primitiveValue(value: Primitive): unknown {
  if (value.kind === "null") return null;
  if (value.kind === "undefined") return undefined;
  if (value.kind === "bigint") return BigInt(value.decimal);
  return value.value;
}
export type ModelFixture = {
  id: string;
  source: string;
  script: string;
  template: string;
  bindings: string[];
  binding: string;
  initial: Primitive;
  input: string;
  programmatic: string;
  composing: string;
};
export function assertModelFixture(value: unknown): asserts value is ModelFixture {
  assert(value && typeof value === "object");
  for (const key of [
    "id",
    "source",
    "script",
    "template",
    "binding",
    "input",
    "programmatic",
    "composing",
  ]) {
    assert(typeof Reflect.get(value, key) === "string" && Reflect.get(value, key).length > 0);
  }
  const binding: unknown = Reflect.get(value, "binding");
  const bindings: unknown = Reflect.get(value, "bindings");
  assert.deepEqual(bindings, [binding]);
  const initial: unknown = Reflect.get(value, "initial");
  assert(initial && typeof initial === "object");
  const kind: unknown = Reflect.get(initial, "kind");
  if (kind === "null") assert.deepEqual(Reflect.ownKeys(initial), ["kind"]);
  else if (kind === "bigint") {
    assert.deepEqual(Reflect.ownKeys(initial), ["kind", "decimal"]);
    assert(typeof Reflect.get(initial, "decimal") === "string");
    assert(/^-?\d+$/.test(Reflect.get(initial, "decimal")));
  } else {
    assert(kind === "string" || kind === "number" || kind === "boolean");
    assert.deepEqual(Reflect.ownKeys(initial), ["kind", "value"]);
    assert.equal(typeof Reflect.get(initial, "value"), kind);
    if (kind === "number") assert(Number.isFinite(Reflect.get(initial, "value")));
  }
}
export type ModelComponent = {
  setup: (props: unknown, context: unknown) => unknown;
  render?: (this: unknown, ...args: unknown[]) => unknown;
};
export type ModelVNode = {
  type: unknown;
  key: unknown;
  props: Record<string, unknown> | null;
  children: unknown;
  patchFlag: number;
  dynamicProps: string[] | null;
  dirs:
    | { dir: unknown; value: unknown; oldValue: unknown; modifiers: unknown; arg: unknown }[]
    | null;
};
export type ModelApp = {
  config: { warnHandler?: (warning: string) => void; errorHandler?: (error: unknown) => void };
  mount(element: HTMLElement): unknown;
  unmount(): void;
};
export type ModelInstance = {
  type: ModelComponent;
  appContext: { app: ModelApp };
  subTree: ModelVNode;
  proxy: { $forceUpdate(): void };
  isUnmounted: boolean;
};
type ModelRuntime = {
  version: string;
  vModelText: unknown;
  createApp(component: ModelComponent): ModelApp;
  getCurrentInstance(): ModelInstance | null;
  nextTick(): Promise<void>;
  isVNode(value: unknown): value is ModelVNode;
};

export function openModelDom() {
  // No shared helper importing Vue is used by this family.
  const runtimePath = fromModelVue.resolve("@vue/runtime-dom");
  assert.equal(
    fromModelVue.cache[runtimePath],
    undefined,
    "runtime-dom must see the real document on first load",
  );
  const window = new Window({ url: "https://native-model.test/" });
  const globals = [
    "window",
    "document",
    "navigator",
    "Node",
    "Element",
    "HTMLElement",
    "HTMLInputElement",
    "SVGElement",
    "Document",
    "ShadowRoot",
    "Event",
  ];
  const saved = globals.map(
    (name) => [name, Object.getOwnPropertyDescriptor(globalThis, name)] as const,
  );
  function close() {
    let failed = false;
    let firstFailure: unknown;
    try {
      window.close();
    } catch (error) {
      failed = true;
      firstFailure = error;
    } finally {
      for (const [name, descriptor] of saved) {
        try {
          if (descriptor) Object.defineProperty(globalThis, name, descriptor);
          else Reflect.deleteProperty(globalThis, name);
        } catch (error) {
          if (!failed) {
            failed = true;
            firstFailure = error;
          }
        }
      }
    }
    if (failed) throw firstFailure;
  }
  try {
    for (const name of globals)
      Object.defineProperty(globalThis, name, {
        configurable: true,
        writable: true,
        value: name === "window" ? window : Reflect.get(window, name),
      });
    const runtime = fromModelVue("vue") as ModelRuntime;
    assert.equal(runtime.version, "3.5.35");
    return { window, runtime, close };
  } catch (error) {
    try {
      close();
    } catch {
      /* Preserve the bootstrap's first failure. */
    }
    throw error;
  }
}
type ImportNode = {
  type: string;
  source?: { type: string; value: string; start: number; end: number };
};
const parser = createRequire(fromModelVue.resolve("@vue/compiler-sfc/package.json"))(
  "@babel/parser",
) as {
  parse(code: string, options: { sourceType: "module" }): { program: { body: ImportNode[] } };
};
const runtimeUrl = dataUrl(
  `import runtime from ${JSON.stringify(pathToFileURL(fromModelVue.resolve("vue")).href)};\n` +
    [
      "openBlock",
      "createElementBlock",
      "createElementVNode",
      "defineComponent",
      "withDirectives",
      "vModelText",
    ]
      .map((name) => `export const ${name} = runtime.${name};`)
      .join("\n"),
);
export function modelModuleSource(code: string) {
  const imports = parser
    .parse(code, { sourceType: "module" })
    .program.body.filter(
      (node) => node.type === "ImportDeclaration" && node.source?.value === "vue",
    );
  assert(imports.length > 0);
  let rewritten = code;
  for (const node of imports.toReversed()) {
    assert(node.source?.type === "StringLiteral");
    const { start, end } = node.source;
    assert(
      Number.isInteger(start) &&
        Number.isInteger(end) &&
        start >= 0 &&
        start < end &&
        end <= code.length,
    );
    rewritten = rewritten.slice(0, start) + JSON.stringify(runtimeUrl) + rewritten.slice(end);
  }
  if (code.includes("/* from 'vue' 雪🌸 */")) assert(rewritten.includes("/* from 'vue' 雪🌸 */"));
  return rewritten;
}
