// Actual browser `.trim` helper; the getter/setter observer is diagnostic only.
import assert from "node:assert/strict";
import fs from "node:fs";
import { pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import type { Window } from "happy-dom";
import {
  dataUrl,
  fromUi,
  hash,
  object,
  primitive,
  primitiveValue,
  type ModelVaporMode,
  type ObjectValue,
} from "./native-original-input-model-vapor-boundary.ts";
import { assertTrimFixture, type TrimFixture } from "./native-input-model-trim-boundary.ts";
import { trimVaporModule } from "./native-input-model-trim-vapor-module.ts";
import {
  modelVaporError,
  vaporDiagnostic,
} from "./native-original-input-model-vapor-observation.ts";
export { modelVaporError } from "./native-original-input-model-vapor-observation.ts";

type Component = { __vapor: boolean; setup: (...args: unknown[]) => unknown; render?: unknown };
type Instance = {
  type: Component;
  appContext: unknown;
  setupState?: object;
  exposed?: object;
  update?: unknown;
  scope: { active: boolean };
};
type App = {
  _component: Component;
  _instance: Instance | null;
  config: {
    warnHandler?: (...args: unknown[]) => void;
    errorHandler?: (...args: unknown[]) => void;
  };
  mount(host: unknown): unknown;
  unmount(): void;
};
type Runtime = {
  version: string;
  createVaporApp(component: Component): App;
  getCurrentInstance(): Instance | null;
  nextTick(): Promise<void>;
};
let sequence = 0;

export async function executeTrimModelVapor(
  code: string,
  fixture: TrimFixture,
  mode: ModelVaporMode,
  attempt: ObjectValue,
): Promise<unknown> {
  assertTrimFixture(fixture);
  assert.equal(process.env.NODE_ENV ?? "development", mode);
  const observations: unknown[] = [];
  attempt.observations = observations;
  const record = (phase: string, value: unknown) => observations.push({ phase, value });
  const globals = [
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
  const saved = globals.map(
    (key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)] as const,
  );
  const hookKey = `vize.original-model-trim-vapor.${mode}.${fixture.id}.${sequence++}`;
  const hookSymbol = Symbol.for(hookKey);
  assert.equal(Object.getOwnPropertyDescriptor(globalThis, hookSymbol), undefined);
  let window: Window | undefined;
  let app: App | undefined;
  let container: ReturnType<Window["document"]["createElement"]> | undefined;
  let runtime: Runtime | undefined;
  let failed = false;
  let firstFailure: unknown;
  let outcome: unknown;
  let unmountAttempted = false;
  const retain = (error: unknown, phase: string) => {
    record(phase, modelVaporError(error));
    if (!failed) {
      failed = true;
      firstFailure = error;
    }
  };
  try {
    const happyPackagePath = fromUi.resolve("happy-dom/package.json");
    const happyPackage: unknown = JSON.parse(fs.readFileSync(happyPackagePath, "utf8"));
    object(happyPackage);
    assert.equal(happyPackage.version, "21.0.4");
    const happyPath = fromUi.resolve("happy-dom");
    const happy = (await import(happyPath)) as typeof import("happy-dom");
    window = new happy.Window({ url: "https://native-vapor-model.test/" });
    for (const key of globals)
      Object.defineProperty(globalThis, key, {
        configurable: true,
        writable: true,
        value: key === "window" ? window : Reflect.get(window, key),
      });
    // This exact browser runtime is loaded only after the real DOM globals exist.
    const runtimePackagePath = fromUi.resolve("vue-vapor-runtime/package.json");
    const runtimePackage: unknown = JSON.parse(fs.readFileSync(runtimePackagePath, "utf8"));
    attempt.runtimePackage = { path: runtimePackagePath, value: runtimePackage };
    object(runtimePackage);
    assert.equal(runtimePackage.version, "3.6.0-rc.9");
    const runtimePath = fromUi.resolve(
      `vue-vapor-runtime/dist/vue.runtime-with-vapor.esm-browser${mode === "production" ? ".prod" : ""}.js`,
    );
    const runtimeAddress = pathToFileURL(runtimePath);
    runtimeAddress.searchParams.set("original-model-observer", hookKey);
    const runtimeUrl = runtimeAddress.href;
    const actualRuntime: unknown = await import(runtimeUrl);
    object(actualRuntime);
    for (const name of ["createVaporApp", "nextTick", "getCurrentInstance", "applyTextModel"])
      assert.equal(typeof actualRuntime[name], "function", name);
    assert(Object.hasOwn(actualRuntime, "currentInstance"));
    const vue = actualRuntime as Runtime;
    runtime = vue;
    attempt.provider = {
      runtimePath,
      runtimeUrl,
      runtimeSha256: hash(fs.readFileSync(runtimePath, "utf8")),
      version: vue.version,
      happyPath,
      happySha256: hash(fs.readFileSync(happyPath, "utf8")),
      happyPackage,
      globalsBeforeRuntimeLoad: globals,
      runtimeOwnKeys: Reflect.ownKeys(actualRuntime).map(String),
    };
    assert.equal(vue.version, "3.6.0-rc.9");
    const helpers: {
      args: unknown[];
      instance: Instance | null;
      returned?: unknown;
      error?: unknown;
    }[] = [];
    Object.defineProperty(globalThis, hookSymbol, {
      configurable: true,
      value: {
        before(args: unknown[], instance: Instance | null, publicInstance: unknown) {
          const item = { args, instance };
          helpers.push(item);
          record("actual-applyTextModel-before-forward", {
            argumentCount: args.length,
            elementTag: args[0] instanceof window!.HTMLInputElement ? args[0].tagName : null,
            getter: vaporDiagnostic(args[1]),
            setter: vaporDiagnostic(args[2]),
            optionsAndRemainingArguments: vaporDiagnostic(args.slice(3)),
            currentInstanceObserved: instance !== null,
            publicGetCurrentInstance: vaporDiagnostic(publicInstance),
          });
          return helpers.length - 1;
        },
        after(index: number, returned: unknown) {
          helpers[index]!.returned = returned;
          record("actual-applyTextModel-after-forward", {
            index,
            returned: vaporDiagnostic(returned),
          });
        },
        failed(index: number, error: unknown) {
          helpers[index]!.error = error;
          record("actual-applyTextModel-forward-threw", { index, error: modelVaporError(error) });
        },
      },
    });
    const adapted = trimVaporModule(code, runtimeUrl, hookKey, attempt);
    const loaded: unknown = (await import(dataUrl(adapted))).default;
    attempt.loadedDefaultDiagnosticOnly = vaporDiagnostic(loaded);
    object(loaded);
    assert.equal(loaded.__vapor, true);
    assert.equal(typeof loaded.setup, "function");
    assert.equal(loaded.render, undefined);
    const component = loaded as Component;
    container = window.document.createElement("div");
    window.document.body.appendChild(container);
    app = vue.createVaporApp(component);
    const warnings: unknown[] = [],
      errors: unknown[] = [],
      events: unknown[] = [];
    attempt.warnings = warnings;
    attempt.errors = errors;
    app.config.warnHandler = (...args) => warnings.push(vaporDiagnostic(args));
    app.config.errorHandler = (...args) => errors.push(vaporDiagnostic(args));
    assert.equal(app._component.setup, component.setup);
    const proxy = app.mount(container);
    await vue.nextTick();
    record("mounted-before-verdict", {
      publicProxy: vaporDiagnostic(proxy),
      helperCalls: helpers.length,
      html: container.innerHTML,
    });
    assert.equal(proxy, undefined, "original primitive inline setup exposes no public proxy");
    assert.equal(helpers.length, 1, "the actual original input owns exactly one helper call");
    const helper = helpers[0]!;
    assert.equal(helper.args.length, 4);
    assert.deepEqual(helper.args[3], { trim: true });
    const element = helper.args[0];
    assert(element instanceof window.HTMLInputElement);
    assert.equal(container.querySelector("input"), element);
    assert.equal(container.childNodes.length, 1);
    const get = helper.args[1],
      set = helper.args[2];
    assert(typeof get === "function" && typeof set === "function");
    assert.equal(helper.error, undefined);
    const instance = helper.instance;
    const appContext: unknown = instance?.appContext;
    record("actual-private-instance-before-verdict", {
      ownKeys: instance ? Reflect.ownKeys(instance).map(String) : null,
      type: instance ? vaporDiagnostic(Object.getOwnPropertyDescriptors(instance.type)) : null,
      setupState: vaporDiagnostic(instance?.setupState),
      exposed: vaporDiagnostic(instance?.exposed),
      update: vaporDiagnostic(instance?.update),
      appPrivateInstanceSame: app._instance === instance,
      sameSetup: instance?.type.setup === component.setup,
      sameApp:
        appContext !== null &&
        typeof appContext === "object" &&
        Reflect.get(appContext, "app") === app,
    });
    assert(instance);
    assert.equal(instance.type.setup, component.setup);
    object(appContext);
    assert.equal(appContext.app, app);
    if (mode === "development") {
      assert(instance && app._instance === instance);
      assert.deepEqual(Object.keys(instance.setupState ?? {}), []);
      assert.deepEqual(Object.keys(instance.exposed ?? {}), []);
      assert.equal(instance.update, undefined);
      assert.equal(instance.scope.active, true);
    } else assert.equal(app._instance, null, "production public app has no private root instance");
    for (const type of ["input", "compositionstart", "compositionend", "change"])
      element.addEventListener(type, (event) =>
        events.push({
          type: event.type,
          value: element.value,
          composing: Boolean(Reflect.get(element, "composing")),
          bubbles: event.bubbles,
          defaultPrevented: event.defaultPrevented,
          sameTarget: event.target === element,
          sameCurrentTarget: event.currentTarget === element,
          isTrusted: vaporDiagnostic(Reflect.get(event, "isTrusted")),
        }),
      );
    const snapshot = (
      phase: string,
      expectedModel: unknown,
      expectedDom: string,
      composing: boolean,
    ) => {
      const actualModel: unknown = get();
      const observed = {
        diagnosticGetter: vaporDiagnostic(actualModel),
        domValue: element.value,
        attributes: [...element.attributes].map((attribute) => [attribute.name, attribute.value]),
        html: container!.innerHTML,
        composing: Boolean(Reflect.get(element, "composing")),
        valueDescriptor: vaporDiagnostic(
          Object.getOwnPropertyDescriptor(window!.HTMLInputElement.prototype, "value"),
        ),
        composingDescriptor: vaporDiagnostic(Object.getOwnPropertyDescriptor(element, "composing")),
      };
      record(phase + "-before-verdict", observed);
      assert.deepEqual(primitive(actualModel), primitive(expectedModel));
      assert.equal(observed.domValue, expectedDom);
      assert.equal(observed.composing, composing);
      assert.equal(container!.querySelector("input"), element);
      return {
        model: primitive(actualModel),
        value: observed.domValue,
        attributes: observed.attributes,
        html: observed.html,
        composing,
      };
    };
    const dispatch = (type: string) => {
      const event = new window!.Event(type, { bubbles: true });
      const dispatched = element.dispatchEvent(event);
      record("actual-event-dispatched", {
        type,
        dispatched,
        defaultPrevented: event.defaultPrevented,
      });
      assert(dispatched && !event.defaultPrevented);
    };
    const initialValue = primitiveValue(fixture.initial);
    const initial = snapshot(
      "initial",
      initialValue,
      initialValue == null ? "" : String(initialValue),
      false,
    );
    element.value = fixture.input;
    dispatch("input");
    await vue.nextTick();
    const input = snapshot("input", fixture.input.trim(), fixture.input, false);
    const setterReturn: unknown = set(fixture.programmatic);
    record("DIAGNOSTIC-captured-lexical-setter-invoked", {
      returned: vaporDiagnostic(setterReturn),
    });
    await vue.nextTick();
    // Plain let/var is not reactive. This diagnostic setter does not invent a render/update API.
    const programmatic = snapshot(
      "diagnostic-programmatic",
      fixture.programmatic,
      fixture.input,
      false,
    );
    dispatch("compositionstart");
    element.value = fixture.composing;
    dispatch("input");
    await vue.nextTick();
    const composing = snapshot(
      "composition-input-ignored",
      fixture.programmatic,
      fixture.composing,
      true,
    );
    dispatch("compositionend");
    await vue.nextTick();
    const compositionEnd = snapshot(
      "compositionend-input",
      fixture.composing.trim(),
      fixture.composing,
      false,
    );
    element.value = fixture.change;
    dispatch("change");
    await vue.nextTick();
    const change = snapshot(
      "change-dom-only",
      fixture.composing.trim(),
      fixture.change.trim(),
      false,
    );
    record("actual-event-list-before-verdict", events);
    assert.deepEqual(
      events.map((event) => {
        object(event);
        return event.type;
      }),
      ["input", "compositionstart", "input", "input", "compositionend", "change"],
    );
    assert.deepEqual(warnings, []);
    assert.deepEqual(errors, []);
    assert.equal(helpers.length, 1, "no reconstructed helper or setup rerun");
    unmountAttempted = true;
    app.unmount();
    await vue.nextTick();
    record("unmounted-before-verdict", {
      html: container.innerHTML,
      childCount: container.childNodes.length,
      connected: element.isConnected,
      scopeActive: instance?.scope.active ?? null,
    });
    assert.equal(container.childNodes.length, 0);
    assert.equal(element.isConnected, false);
    if (instance) assert.equal(instance.scope.active, false);
    outcome = {
      initial,
      input,
      programmatic,
      composing,
      compositionEnd,
      change,
      setterReturn: primitive(setterReturn),
      events,
      warnings,
      errors,
      publicProxy: "undefined",
      render: "undefined",
      unmounted: [],
    };
    attempt.outcome = outcome;
  } catch (error) {
    retain(error, "runtime-first-failure");
  } finally {
    if (app && !unmountAttempted) {
      try {
        app.unmount();
        await runtime?.nextTick();
        record("failure-cleanup-unmounted", { html: container?.innerHTML });
      } catch (error) {
        retain(error, "unmount-cleanup-failure");
      }
    }
    try {
      container?.remove();
    } catch (error) {
      retain(error, "host-cleanup-failure");
    }
    try {
      if (window) await window.happyDOM.close();
    } catch (error) {
      retain(error, "dom-cleanup-failure");
    }
    for (const [key, descriptor] of saved) {
      try {
        if (descriptor) Object.defineProperty(globalThis, key, descriptor);
        else assert(Reflect.deleteProperty(globalThis, key));
      } catch (error) {
        retain(error, "globals-cleanup-failure");
      }
    }
    Reflect.deleteProperty(globalThis, hookSymbol);
    const cleanup = {
      unmountAttempted,
      globalsRestored: saved.every(([key, descriptor]) => {
        const current = Object.getOwnPropertyDescriptor(globalThis, key);
        return isDeepStrictEqual(current, descriptor);
      }),
      observerRemoved: Object.getOwnPropertyDescriptor(globalThis, hookSymbol) === undefined,
    };
    attempt.cleanup = cleanup;
    try {
      assert(cleanup.globalsRestored && cleanup.observerRemoved);
    } catch (error) {
      retain(error, "cleanup-verification-failure");
    }
  }
  if (failed) throw firstFailure;
  return outcome;
}
