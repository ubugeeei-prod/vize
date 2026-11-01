// Actual DOM/provider setup; caller owns the window before any later guard can fail.
import assert from "node:assert/strict";
import fs from "node:fs";
import { pathToFileURL } from "node:url";
import type { Window } from "happy-dom";
import {
  fromUi,
  object,
  hash,
  type ObjectValue,
} from "./native-original-input-model-vapor-boundary.ts";
export type Component = {
  __vapor: boolean;
  setup: (...args: unknown[]) => unknown;
  render?: unknown;
};
type Instance = {
  type: Component;
  appContext: unknown;
  setupState?: object;
  exposed?: object;
  update?: unknown;
  scope: { active: boolean };
};
export type App = {
  _component: Component;
  _instance: Instance | null;
  config: {
    warnHandler?: (...args: unknown[]) => void;
    errorHandler?: (...args: unknown[]) => void;
  };
  mount(host: unknown): unknown;
  unmount(): void;
};
export type Runtime = {
  version: string;
  createVaporApp(component: Component): App;
  getCurrentInstance(): Instance | null;
  nextTick(): Promise<void>;
};
export const checkboxVaporGlobals = [
  "window",
  "document",
  "navigator",
  "Document",
  "Node",
  "Text",
  "Comment",
  "Element",
  "HTMLElement",
  "HTMLInputElement",
  "SVGElement",
  "Event",
  "ShadowRoot",
];
export async function openCheckboxVaporDom(
  globals: readonly string[],
  record: (phase: string, value: unknown) => void,
  opened: (window: Window) => void,
) {
  const happyPackagePath = fromUi.resolve("happy-dom/package.json");
  const happyPackage: unknown = JSON.parse(fs.readFileSync(happyPackagePath, "utf8"));
  object(happyPackage);
  assert.equal(happyPackage.version, "21.0.4");
  const happyPath = fromUi.resolve("happy-dom");
  const happyNamespace: unknown = await import(happyPath);
  object(happyNamespace);
  assert.equal(typeof happyNamespace.Window, "function");
  assert.equal(typeof happyNamespace.Document, "function");
  const happy = happyNamespace as typeof import("happy-dom");
  const window = new happy.Window({ url: "https://native-vapor-checkbox.test/" });
  opened(window);
  const documentIdentity = {
    constructor: window.document.constructor.name,
    instanceOfWindowDocument: window.document instanceof window.Document,
    instanceOfImplementationDocument: window.document instanceof happy.Document,
  };
  record("actual-document-identity", documentIdentity);
  assert(documentIdentity.instanceOfImplementationDocument);
  for (const key of globals)
    Object.defineProperty(globalThis, key, {
      configurable: true,
      writable: true,
      value:
        key === "window" ? window : key === "Document" ? happy.Document : Reflect.get(window, key),
    });

  return { window, happyPath, happyPackage, documentIdentity };
}

// DOM identity is checked before loading the unique actual browser-runtime module.
export async function loadCheckboxVaporRuntime(
  mode: "development" | "production",
  hookKey: string,
  attempt: ObjectValue,
  environment: Awaited<ReturnType<typeof openCheckboxVaporDom>>,
  globals: readonly string[],
) {
  assert.equal(Reflect.get(globalThis, "window"), environment.window);
  assert.equal(Reflect.get(globalThis, "document"), environment.window.document);
  const { happyPath, happyPackage, documentIdentity } = environment;
  const packagePath = fromUi.resolve("vue-vapor-runtime/package.json");
  const metadata: unknown = JSON.parse(fs.readFileSync(packagePath, "utf8"));
  attempt.runtimePackage = { path: packagePath, value: metadata };
  object(metadata);
  assert.equal(metadata.version, "3.6.0-rc.9");
  const runtimePath = fromUi.resolve(
    `vue-vapor-runtime/dist/vue.runtime-with-vapor.esm-browser${mode === "production" ? ".prod" : ""}.js`,
  );
  const address = pathToFileURL(runtimePath);
  address.searchParams.set("original-checkbox-observer", hookKey);
  const runtimeUrl = address.href;
  const runtimeSha256 = hash(fs.readFileSync(runtimePath, "utf8"));
  const actual: unknown = await import(runtimeUrl);
  object(actual);
  attempt.provider = {
    runtimePath,
    runtimeUrl,
    runtimeSha256,
    happyPath,
    happySha256: hash(fs.readFileSync(happyPath, "utf8")),
    happyPackage,
    globalsBeforeRuntimeLoad: globals,
    runtimeOwnKeys: Reflect.ownKeys(actual).map(String),
    documentIdentity,
  };
  for (const name of ["createVaporApp", "nextTick", "getCurrentInstance", "applyCheckboxModel"])
    assert.equal(typeof actual[name], "function", name);
  assert(Object.hasOwn(actual, "currentInstance"));
  assert.equal(actual.version, "3.6.0-rc.9");
  const apply = actual.applyCheckboxModel;
  assert(typeof apply === "function");
  assert.equal(apply.length, 3);
  const vue = actual as Runtime;

  return { runtimePath, runtimeUrl, runtimeSha256, actual, vue };
}
