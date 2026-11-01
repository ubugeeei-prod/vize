// Real three-argument checkbox helper; captured lexical access is DIAGNOSTIC only.
import assert from "node:assert/strict";
import { checkboxVaporModule } from "./native-input-model-checkbox-modules.ts";
import {
  checkboxVaporGlobals,
  openCheckboxVaporDom,
  loadCheckboxVaporRuntime,
  type Component,
  type App,
  type Runtime,
} from "./native-input-model-checkbox-vapor-environment.ts";
import fs from "node:fs";
import { isDeepStrictEqual } from "node:util";
import type { Window } from "happy-dom";
import {
  hash,
  object,
  primitive,
  primitiveValue,
  type ObjectValue,
} from "./native-original-input-model-vapor-boundary.ts";
import {
  assertCheckboxFixture,
  type CheckboxFixture,
  type CheckboxMode,
} from "./native-input-model-checkbox-boundary.ts";
import {
  modelVaporError,
  vaporDiagnostic,
} from "./native-original-input-model-vapor-observation.ts";
export { modelVaporError } from "./native-original-input-model-vapor-observation.ts";

type Helper = {
  args: unknown[];
  instance: unknown;
  completed: boolean;
  returned?: unknown;
  error?: unknown;
};
let sequence = 0;

export async function executeCheckboxModelVapor(
  code: string,
  fixture: CheckboxFixture,
  mode: CheckboxMode,
  attempt: ObjectValue,
): Promise<unknown> {
  const observations: unknown[] = [];
  attempt.observations = observations;
  const warnings: unknown[] = [],
    errors: unknown[] = [],
    events: unknown[] = [];
  attempt.warnings = warnings;
  attempt.errors = errors;
  attempt.events = events;
  const record = (phase: string, value: unknown) => observations.push({ phase, value });
  const globals = checkboxVaporGlobals;
  const saved = globals.map(
    (key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)] as const,
  );
  const hookKey = `vize.original-model-checkbox-vapor.${mode}.${fixture.id}.${sequence++}`;
  const hookSymbol = Symbol.for(hookKey);
  const savedHook = Object.getOwnPropertyDescriptor(globalThis, hookSymbol);
  let hookInstalled = false;
  let window: Window | undefined;
  let app: App | undefined;
  let container: ReturnType<Window["document"]["createElement"]> | undefined;
  let runtime: Runtime | undefined;
  let failed = false;
  let firstFailure: unknown;
  let unmountAttempted = false;
  let windowClosed = false;
  let setupInvocations = 0;
  let outcome: unknown;
  const retain = (error: unknown, phase: string) => {
    if (!failed) {
      failed = true;
      firstFailure = error;
    }
    try {
      record(phase, modelVaporError(error));
    } catch {
      /* Preserve the first failure. */
    }
  };
  async function unmount() {
    if (!app || unmountAttempted) return;
    unmountAttempted = true;
    try {
      app.unmount();
    } catch (error) {
      retain(error, "unmount-failure");
    }
    try {
      await runtime?.nextTick();
    } catch (error) {
      retain(error, "unmount-tick-failure");
    }
    try {
      record("unmounted-before-comparisons", { html: container?.innerHTML });
    } catch (error) {
      retain(error, "unmount-observation-failure");
    }
  }
  try {
    attempt.originalCode = code;
    record("actor-source-before-comparisons", { code, source: fixture.source });
    assertCheckboxFixture(fixture);
    assert.equal(process.env.NODE_ENV ?? "development", mode);
    assert.equal(savedHook, undefined);
    const environment = await openCheckboxVaporDom(globals, record, (opened) => {
      window = opened;
    });
    window = environment.window;
    const { runtimePath, runtimeUrl, runtimeSha256, actual, vue } = await loadCheckboxVaporRuntime(
      mode,
      hookKey,
      attempt,
      environment,
      globals,
    );
    runtime = vue;
    const helpers: Helper[] = [];
    Object.defineProperty(globalThis, hookSymbol, {
      configurable: true,
      value: {
        before(args: unknown[], instance: unknown, publicInstance: unknown) {
          helpers.push({ args, instance, completed: false });
          record("actual-checkbox-helper-before-forward", {
            argumentCount: args.length,
            elementTag: args[0] instanceof window!.HTMLInputElement ? args[0].tagName : null,
            originalGetter: vaporDiagnostic(args[1]),
            originalSetter: vaporDiagnostic(args[2]),
            remainingArguments: vaporDiagnostic(args.slice(3)),
            currentInstanceObserved: instance !== null,
            publicGetCurrentInstance: vaporDiagnostic(publicInstance),
          });
          return helpers.length - 1;
        },
        after(index: number, returned: unknown) {
          const item = helpers[index];
          assert(item);
          item.completed = true;
          item.returned = returned;
          record("actual-checkbox-helper-returned", { index, returned: vaporDiagnostic(returned) });
        },
        failed(index: number, error: unknown) {
          const item = helpers[index];
          assert(item);
          item.error = error;
          retain(error, "actual-checkbox-helper-first-failure");
          record("actual-checkbox-helper-threw", { index, error: modelVaporError(error) });
        },
      },
    });
    hookInstalled = true;
    const adapted = checkboxVaporModule(code, runtimeUrl, hookKey, actual);
    attempt.moduleAddressAdapter = adapted;
    const loaded: unknown = (await import(adapted.moduleUrl)).default;
    attempt.loadedDefaultDiagnosticOnly = vaporDiagnostic(loaded);
    object(loaded);
    assert.equal(loaded.__vapor, true);
    assert.equal(typeof loaded.setup, "function");
    assert.equal(loaded.render, undefined);
    const original = loaded as Component;
    const originalSetup = original.setup;
    const descriptors = Object.getOwnPropertyDescriptors(original);
    record("original-component-own-descriptors", vaporDiagnostic(descriptors));
    assert(
      descriptors.setup &&
        "value" in descriptors.setup &&
        descriptors.setup.value === originalSetup,
    );
    let setupInstance: unknown;
    const countedSetup = function (this: unknown, ...args: unknown[]) {
      setupInvocations++;
      setupInstance = actual.currentInstance;
      record("actual-setup-before-forward", {
        args: vaporDiagnostic(args),
        privateCurrentInstance: vaporDiagnostic(setupInstance !== null),
        publicGetCurrentInstance: vaporDiagnostic(vue.getCurrentInstance()),
      });
      try {
        const result: unknown = Reflect.apply(originalSetup, this, args);
        record("actual-setup-returned", vaporDiagnostic(result));
        return result;
      } catch (error) {
        retain(error, "original-setup-first-failure");
        throw error;
      }
    };
    const component = Object.defineProperties(
      {},
      { ...descriptors, setup: { ...descriptors.setup, value: countedSetup } },
    ) as Component;
    container = window.document.createElement("div");
    window.document.body.appendChild(container);
    app = vue.createVaporApp(component);
    app.config.warnHandler = (...args) => warnings.push(vaporDiagnostic(args));
    app.config.errorHandler = (...args) => {
      errors.push(vaporDiagnostic(args));
      retain(args[0], "vue-original-error-handler");
    };
    assert.equal(app._component.setup, countedSetup);
    const proxy = app.mount(container);
    await vue.nextTick();
    record("mounted-before-comparisons", {
      publicProxy: vaporDiagnostic(proxy),
      helperCalls: helpers.length,
      setupInvocations,
      html: container.innerHTML,
    });
    assert.equal(proxy, undefined);
    assert.equal(setupInvocations, 1);
    assert.equal(helpers.length, 1);
    const helper = helpers[0];
    assert(helper);
    assert.equal(helper.args.length, 3);
    assert(helper.completed && helper.error === undefined);
    assert.equal(helper.returned, undefined);
    const elementValue = helper.args[0];
    assert(elementValue instanceof window.HTMLInputElement);
    const element = elementValue;
    assert.equal(container.querySelector("input"), element);
    assert.equal(container.childNodes.length, 1);
    const getter = helper.args[1],
      setter = helper.args[2];
    assert(typeof getter === "function" && typeof setter === "function");
    const get = getter,
      set = setter;
    const instance = helper.instance;
    object(instance);
    assert.equal(instance, setupInstance);
    object(instance.type);
    object(instance.appContext);
    object(instance.scope);
    record("actual-private-instance-before-comparisons", {
      ownKeys: Reflect.ownKeys(instance).map(String),
      type: vaporDiagnostic(Object.getOwnPropertyDescriptors(instance.type)),
      setupState: vaporDiagnostic(instance.setupState),
      exposed: vaporDiagnostic(instance.exposed),
      update: vaporDiagnostic(instance.update),
      scopeActive: instance.scope.active,
      sameSetup: instance.type.setup === countedSetup,
      sameApp: instance.appContext.app === app,
      appPrivateInstanceSame: app._instance === instance,
    });
    assert.equal(instance.type.setup, countedSetup);
    assert.equal(instance.appContext.app, app);
    assert.equal(instance.scope.active, true);
    if (mode === "development") {
      assert.equal(app._instance, instance);
      assert.deepEqual(Object.keys(instance.setupState ?? {}), []);
      assert.deepEqual(Object.keys(instance.exposed ?? {}), []);
      assert.equal(instance.update, undefined);
    } else assert.equal(app._instance, null);
    let beforeClick: unknown;
    for (const type of ["click", "input", "change"])
      element.addEventListener(type, (event) => {
        const observed = {
          type: event.type,
          checked: element.checked,
          value: element.value,
          inputType: element.type,
          diagnosticModelBeforeClick: primitive(beforeClick),
          diagnosticModelAtEvent: primitive(get()),
          bubbles: event.bubbles,
          cancelable: event.cancelable,
          composed: event.composed,
          defaultPrevented: event.defaultPrevented,
          sameTarget: event.target === element,
          sameCurrentTarget: event.currentTarget === element,
          focused: window!.document.activeElement === element,
          isTrusted: primitive(Reflect.get(event, "isTrusted")),
        };
        events.push(observed);
        record("actual-dom-event-before-comparisons", observed);
      });
    function snapshot(phase: string, expectedModel: unknown, checked: boolean, focused: boolean) {
      const actualModel: unknown = get();
      const observed = {
        diagnosticModel: primitive(actualModel),
        checked: element.checked,
        value: element.value,
        type: element.type,
        defaultValue: element.defaultValue,
        defaultChecked: element.defaultChecked,
        indeterminate: element.indeterminate,
        attributes: [...element.attributes].map((attribute) => [attribute.name, attribute.value]),
        html: container!.innerHTML,
        connected: element.isConnected,
        childCount: element.childNodes.length,
        focused: window!.document.activeElement === element,
      };
      record(phase + "-before-comparisons", observed);
      assert(
        Object.is(actualModel, expectedModel),
        "captured actual lexical getter remains DIAGNOSTIC",
      );
      assert.equal(observed.checked, checked);
      assert.equal(observed.value, "on");
      assert.equal(observed.type, "checkbox");
      assert.equal(observed.defaultValue, "");
      assert.equal(observed.defaultChecked, false);
      assert.equal(observed.indeterminate, false);
      assert.equal(observed.focused, focused);
      assert.deepEqual(observed.attributes, [["type", "checkbox"]]);
      assert.equal(observed.connected, true);
      assert.equal(observed.childCount, 0);
      assert.equal(container!.querySelector("input"), element);
      return { phase, ...observed };
    }
    const expected = fixture.checkboxRuntime;
    const initialValue = primitiveValue(fixture.initial);
    const initial = snapshot("initial", initialValue, expected.initialChecked, false);
    element.focus();
    record("actual-input-focused", { focused: window.document.activeElement === element });
    async function click(phase: string, checked: boolean) {
      beforeClick = get();
      record(phase + "-before-real-click", {
        checked: element.checked,
        diagnosticModel: primitive(beforeClick),
      });
      const returned: unknown = element.click();
      record(phase + "-after-real-click", {
        returned: primitive(returned),
        checked: element.checked,
      });
      await vue.nextTick();
      return snapshot(phase, checked, checked, true);
    }
    const firstClick = await click("first-click", expected.firstClick);
    const setterReturn: unknown = Reflect.apply(set, undefined, [expected.programmatic]);
    record("DIAGNOSTIC-original-lexical-setter-returned", {
      args: [expected.programmatic],
      returned: vaporDiagnostic(setterReturn),
    });
    const beforeProgrammaticTick = snapshot(
      "programmatic-before-tick",
      expected.programmatic,
      expected.firstClick,
      true,
    );
    await vue.nextTick();
    const programmatic = snapshot(
      "programmatic-after-tick",
      expected.programmatic,
      expected.vapor.afterProgrammatic,
      true,
    );
    const secondClick = await click("second-click", expected.vapor.secondClick);
    const thirdClick = await click("third-click", expected.vapor.thirdClick);
    record("whole-event-list-before-comparisons", events);
    const expectedEvents = [
      [initialValue, expected.firstClick],
      [expected.programmatic, expected.vapor.secondClick],
      [expected.vapor.secondClick, expected.vapor.thirdClick],
    ].flatMap(([before, checked]) =>
      ["click", "input", "change"].map((type) => ({
        type,
        checked,
        value: "on",
        inputType: "checkbox",
        diagnosticModelBeforeClick: primitive(before),
        diagnosticModelAtEvent: primitive(type === "change" ? checked : before),
        bubbles: true,
        cancelable: true,
        composed: type === "click",
        defaultPrevented: false,
        sameTarget: true,
        sameCurrentTarget: true,
        focused: true,
        isTrusted: { kind: "undefined" },
      })),
    );
    assert.deepEqual(events, expectedEvents);
    assert.equal(setupInvocations, 1);
    assert.equal(helpers.length, 1);
    assert.deepEqual(warnings, []);
    assert.deepEqual(errors, []);
    await unmount();
    record("cleanup-state-before-comparisons", {
      html: container.innerHTML,
      childCount: container.childNodes.length,
      connected: element.isConnected,
      scopeActive: instance.scope.active,
    });
    assert.equal(container.childNodes.length, 0);
    assert.equal(element.isConnected, false);
    assert.equal(instance.scope.active, false);
    assert.deepEqual(warnings, []);
    assert.deepEqual(errors, []);
    assert.equal(hash(fs.readFileSync(runtimePath, "utf8")), runtimeSha256);
    outcome = {
      initial,
      firstClick,
      beforeProgrammaticTick,
      programmatic,
      secondClick,
      thirdClick,
      diagnosticSetterReturn: primitive(setterReturn),
      events,
      warnings,
      errors,
      setupInvocations,
      publicProxy: "undefined",
      render: "undefined",
      unmounted: true,
    };
    attempt.outcome = outcome;
  } catch (error) {
    retain(error, "runtime-first-failure");
  } finally {
    await unmount();
    try {
      container?.remove();
    } catch (error) {
      retain(error, "host-cleanup-failure");
    }
    try {
      if (window) {
        await window.happyDOM.close();
        windowClosed = true;
      }
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
    if (hookInstalled) {
      try {
        assert(Reflect.deleteProperty(globalThis, hookSymbol));
      } catch (error) {
        retain(error, "observer-cleanup-failure");
      }
    }
    const cleanup = {
      unmountAttempted,
      windowClosed,
      globalsRestored: saved.every(([key, descriptor]) =>
        isDeepStrictEqual(Object.getOwnPropertyDescriptor(globalThis, key), descriptor),
      ),
      observerRestored: isDeepStrictEqual(
        Object.getOwnPropertyDescriptor(globalThis, hookSymbol),
        savedHook,
      ),
    };
    attempt.cleanup = cleanup;
    try {
      record("cleanup-before-comparisons", cleanup);
    } catch (error) {
      retain(error, "cleanup-observation-failure");
    }
    try {
      assert(cleanup.globalsRestored && cleanup.observerRestored);
    } catch (error) {
      retain(error, "cleanup-verification-failure");
    }
  }
  try {
    assert.deepEqual(warnings, []);
    assert.deepEqual(errors, []);
  } catch (error) {
    retain(error, "final-runtime-diagnostics-failure");
  }
  if (failed) throw firstFailure;
  return outcome;
}
