// Real checkbox clicks and measured Vue renders; callback requests are not lexical reads.
import assert from "node:assert/strict";
import { checkboxModule } from "./native-input-model-checkbox-modules.ts";
import {
  primitive,
  primitiveValue,
  type ModelApp,
  type ModelComponent,
  type ModelInstance,
  type openModelDom,
} from "./native-original-input-model-sfc-dom-environment.ts";
import {
  assertCheckboxFixture,
  type CheckboxFixture,
  type CheckboxMode,
} from "./native-input-model-checkbox-boundary.ts";
import { runtimeErrorDetails } from "./native-original-for-constant-sfc-dom-observation.ts";
import { vaporDiagnostic } from "./native-original-input-model-vapor-observation.ts";

type Dom = Awaited<ReturnType<typeof openModelDom>>;
let sequence = 0;
export function modelError(error: unknown) {
  return {
    ...runtimeErrorDetails(error),
    cause: vaporDiagnostic(
      error && typeof error === "object" ? Reflect.get(error, "cause") : undefined,
    ),
    completeOwnDescriptors: vaporDiagnostic(error),
  };
}

export async function executeCheckboxModelDom(
  dom: Dom,
  code: string,
  fixture: CheckboxFixture,
  mode: CheckboxMode,
  actor: "official" | "native",
  observe: (observation: unknown) => void,
) {
  const { window, runtime } = dom;
  const container = window.document.createElement("div");
  let app: ModelApp | undefined;
  let component: ModelComponent | undefined;
  let instance: ModelInstance | null = null;
  let setupInvocations = 0;
  let renders = 0;
  const warnings: unknown[] = [],
    errors: unknown[] = [],
    events: unknown[] = [],
    writes: unknown[] = [];
  const originals = new WeakMap<object, unknown>();
  let failed = false;
  let firstFailure: unknown;
  let unmountAttempted = false;
  let outcome: unknown;
  const record = (phase: string, details?: unknown) =>
    observe(
      vaporDiagnostic({
        phase,
        actor,
        mode,
        setupInvocations,
        renders,
        host: container.outerHTML,
        connected: container.isConnected,
        sameApp: instance?.appContext.app === app,
        sameSetup: instance?.type.setup === component?.setup,
        isUnmounted: instance?.isUnmounted,
        warnings: [...warnings],
        errors: [...errors],
        details,
      }),
    );
  const retain = (error: unknown, phase: string) => {
    if (!failed) {
      failed = true;
      firstFailure = error;
    }
    try {
      record(phase, modelError(error));
    } catch {
      /* Preserve the first failure. */
    }
  };
  const current = () => {
    assert(instance);
    return instance;
  };
  const input = () => {
    const element = container.querySelector("input");
    assert(element instanceof window.HTMLInputElement);
    assert.equal(container.childNodes.length, 1);
    return element;
  };
  const vnode = () => {
    const node = current().subTree;
    assert(runtime.isVNode(node));
    return node;
  };
  const binding = () => {
    const row = vnode().dirs?.[0];
    assert(row);
    return row.value;
  };
  function observedVNode(value: unknown) {
    if (!runtime.isVNode(value)) return { isVNode: false, value: vaporDiagnostic(value) };
    return {
      type: value.type,
      key: value.key,
      props: vaporDiagnostic(value.props),
      children: vaporDiagnostic(value.children),
      patchFlag: value.patchFlag,
      dynamicProps: value.dynamicProps,
      directives:
        value.dirs?.map((row) => ({
          actualCheckboxModel: row.dir === Reflect.get(runtime, "vModelCheckbox"),
          value: primitive(row.value),
          oldValue: primitive(row.oldValue),
          arg: primitive(row.arg),
          modifiers: vaporDiagnostic(row.modifiers),
        })) ?? null,
    };
  }
  function forward(original: unknown, receiver: unknown, args: unknown[], origin: string) {
    assert(typeof original === "function");
    const row: Record<string, unknown> = {
      origin,
      argumentCount: args.length,
      args: vaporDiagnostic(args),
      original: vaporDiagnostic(original),
      renderedBindingBefore: primitive(binding()),
    };
    writes.push(row);
    record("original-callback-before-forward", row);
    try {
      const returned: unknown = Reflect.apply(original, receiver, args);
      row.returned = vaporDiagnostic(returned);
      row.renderedBindingAfter = primitive(binding());
      record("original-callback-after-forward", row);
      return returned;
    } catch (error) {
      retain(error, "original-callback-first-failure");
      try {
        row.error = modelError(error);
        record("original-callback-threw", row);
      } catch {
        /* Preserve the original callback error. */
      }
      throw error;
    }
  }
  function counted(render: (this: unknown, ...args: unknown[]) => unknown) {
    return function (this: unknown, ...args: unknown[]) {
      renders++;
      const result: unknown = Reflect.apply(render, this, args);
      record("render-before-comparisons", { vnode: observedVNode(result) });
      if (runtime.isVNode(result) && result.props) {
        const descriptor = Object.getOwnPropertyDescriptor(result.props, "onUpdate:modelValue");
        const original: unknown = descriptor?.value;
        assert(typeof original === "function" && descriptor);
        const wrapped = function (this: unknown, ...values: unknown[]) {
          return forward(original, this, values, "actual-change-listener");
        };
        originals.set(wrapped, original);
        Object.defineProperty(result.props, "onUpdate:modelValue", {
          ...descriptor,
          value: wrapped,
        });
        record("actual-vnode-callback-ownership", { descriptor: vaporDiagnostic(descriptor) });
      }
      return result;
    };
  }
  async function forceRender() {
    const before = renders;
    await runtime.nextTick();
    record("before-explicit-force-render", { before, actual: renders });
    assert.equal(renders, before, "plain lexical assignment does not invent an automatic render");
    current().proxy.$forceUpdate();
    await runtime.nextTick();
    record("after-explicit-force-render", { before, actual: renders });
    assert.equal(renders, before + 1);
  }
  async function unmount() {
    if (!app || unmountAttempted) return;
    unmountAttempted = true;
    try {
      app.unmount();
    } catch (error) {
      retain(error, "unmount-failure");
    }
    try {
      await runtime.nextTick();
    } catch (error) {
      retain(error, "unmount-tick-failure");
    }
    try {
      record("unmounted-before-comparisons");
    } catch (error) {
      retain(error, "unmount-observation-failure");
    }
  }
  try {
    record("actor-source-before-comparisons", { code, source: fixture.source });
    assertCheckboxFixture(fixture);
    assert.equal(process.env.NODE_ENV ?? "development", mode);
    assert.equal(runtime.version, "3.5.35");
    for (const name of ["createApp", "getCurrentInstance", "nextTick", "isVNode"])
      assert.equal(typeof Reflect.get(runtime, name), "function", name);
    const checkbox: unknown = Reflect.get(runtime, "vModelCheckbox");
    assert(checkbox && typeof checkbox === "object");
    for (const name of ["created", "mounted", "beforeUpdate"])
      assert.equal(typeof Reflect.get(checkbox, name), "function", name);
    const adapted = checkboxModule(code, runtime, `${mode}.${actor}.${fixture.id}.${sequence++}`);
    record("module-address-and-provider-before-load", adapted);
    const loaded: unknown = (await import(adapted.moduleUrl)).default;
    record("actual-component-before-comparisons", {
      actual: vaporDiagnostic(loaded),
      descriptors:
        loaded && typeof loaded === "object"
          ? vaporDiagnostic(Object.getOwnPropertyDescriptors(loaded))
          : null,
    });
    assert(loaded && typeof loaded === "object");
    assert.equal(typeof Reflect.get(loaded, "setup"), "function");
    const original = loaded as ModelComponent;
    assert(original.render === undefined || typeof original.render === "function");
    const originalSetup = original.setup;
    const setupDescriptor = Object.getOwnPropertyDescriptor(original, "setup");
    assert(
      setupDescriptor && "value" in setupDescriptor && setupDescriptor.value === originalSetup,
    );
    component = { ...original };
    component.setup = function (this: unknown, props, context) {
      setupInvocations++;
      instance = runtime.getCurrentInstance();
      record("setup-instance-before-comparisons");
      assert(instance && instance.type.setup === component?.setup);
      const result: unknown = Reflect.apply(originalSetup, this, [props, context]);
      record("actual-setup-return-before-comparisons", vaporDiagnostic(result));
      if (typeof result === "function")
        return counted(result as (this: unknown, ...args: unknown[]) => unknown);
      return result;
    };
    if (original.render) component.render = counted(original.render);
    app = runtime.createApp(component);
    app.config.warnHandler = (...args) => warnings.push(vaporDiagnostic(args));
    app.config.errorHandler = (...args) => {
      errors.push({ args: vaporDiagnostic(args), error: modelError(args[0]) });
      retain(args[0], "vue-original-error-handler");
    };
    window.document.body.appendChild(container);
    app.mount(container);
    record("mounted-before-comparisons", dom.documentIdentity);
    assert.equal(current().appContext.app, app);
    assert.equal(setupInvocations, 1);
    assert.equal(renders, 1);
    const element = input();
    function snapshot(phase: string, expectedBinding: unknown, checked: boolean, focused: boolean) {
      record(phase + "-start");
      const node = vnode(),
        actual = input(),
        directives = node.dirs;
      const observed = {
        renderedBinding: primitive(binding()),
        oldBinding: primitive(directives?.[0]?.oldValue),
        checked: actual.checked,
        value: actual.value,
        type: actual.type,
        defaultValue: actual.defaultValue,
        defaultChecked: actual.defaultChecked,
        indeterminate: actual.indeterminate,
        focused: window.document.activeElement === actual,
        attributes: [...actual.attributes].map((attribute) => [attribute.name, attribute.value]),
        html: container.outerHTML,
        inputHtml: actual.outerHTML,
        connected: actual.isConnected,
        childCount: actual.childNodes.length,
        setupInvocations,
        renders,
        vnode: {
          type: node.type,
          key: node.key,
          children: node.children,
          patchFlag: node.patchFlag,
          dynamicProps: node.dynamicProps,
          propKeys: Object.keys(node.props ?? {}),
          modifiers: directives?.[0]?.modifiers,
        },
      };
      record(phase + "-before-comparisons", {
        ...observed,
        props: vaporDiagnostic(Object.getOwnPropertyDescriptors(node.props ?? {})),
      });
      assert.equal(actual, element);
      assert.equal(node.type, "input");
      assert.equal(node.key, null);
      assert.equal(node.children, null);
      assert(Number.isSafeInteger(node.patchFlag));
      assert(
        node.dynamicProps === null || node.dynamicProps.every((name) => typeof name === "string"),
      );
      assert(node.props && typeof node.props["onUpdate:modelValue"] === "function");
      assert.deepEqual(new Set(Object.keys(node.props)), new Set(["type", "onUpdate:modelValue"]));
      assert.equal(node.props.type, "checkbox");
      assert(directives && directives.length === 1);
      assert.equal(directives[0]?.dir, checkbox);
      assert.equal(directives[0]?.arg, undefined);
      assert.deepEqual(directives[0]?.modifiers, {});
      assert(Object.is(binding(), expectedBinding));
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
      return { phase, ...observed };
    }
    for (const type of ["click", "input", "change"])
      element.addEventListener(type, (event) => {
        const observed = {
          type: event.type,
          checked: element.checked,
          value: element.value,
          inputType: element.type,
          renderedBinding: primitive(binding()),
          bubbles: event.bubbles,
          cancelable: event.cancelable,
          composed: event.composed,
          defaultPrevented: event.defaultPrevented,
          sameTarget: event.target === element,
          sameCurrentTarget: event.currentTarget === element,
          focused: window.document.activeElement === element,
          isTrusted: primitive(Reflect.get(event, "isTrusted")),
        };
        events.push(observed);
        record("actual-dom-event-before-comparisons", observed);
      });
    const initialValue = primitiveValue(fixture.initial);
    const expected = fixture.checkboxRuntime;
    const initial = snapshot("initial", initialValue, expected.initialChecked, false);
    element.focus();
    record("actual-input-focused", { focused: window.document.activeElement === element });
    const click = async (phase: string, checked: boolean) => {
      record(phase + "-before-real-click", {
        checked: element.checked,
        renderedBinding: primitive(binding()),
      });
      const returned: unknown = element.click();
      record(phase + "-after-real-click-before-render", {
        returned: primitive(returned),
        checked: element.checked,
      });
      await forceRender();
      return snapshot(phase, checked, checked, true);
    };
    const firstClick = await click("first-click", expected.firstClick);
    const setter: unknown = vnode().props?.["onUpdate:modelValue"];
    assert(typeof setter === "function");
    const originalSetter = originals.get(setter);
    assert(
      typeof originalSetter === "function",
      "programmatic write uses the original current-vnode callback",
    );
    const setterReturn = forward(
      originalSetter,
      undefined,
      [expected.programmatic],
      "programmatic-original-callback",
    );
    record("programmatic-original-callback-before-render", { returned: primitive(setterReturn) });
    assert.equal(setterReturn, expected.programmatic);
    const beforeProgrammaticRender = snapshot(
      "programmatic-before-render",
      expected.firstClick,
      expected.firstClick,
      true,
    );
    await forceRender();
    const programmatic = snapshot(
      "programmatic-after-render",
      expected.programmatic,
      expected.dom.afterProgrammatic,
      true,
    );
    const secondClick = await click("second-click", expected.dom.secondClick);
    const thirdClick = await click("third-click", expected.dom.thirdClick);
    record("whole-event-list-before-comparisons", events);
    const expectedEvents = [
      [initialValue, expected.firstClick],
      [expected.programmatic, expected.dom.secondClick],
      [expected.dom.secondClick, expected.dom.thirdClick],
    ].flatMap(([renderedBinding, checked]) =>
      ["click", "input", "change"].map((type) => ({
        type,
        checked,
        value: "on",
        inputType: "checkbox",
        renderedBinding: primitive(renderedBinding),
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
    assert.equal(writes.length, 4);
    assert.equal(setupInvocations, 1);
    assert.equal(renders, 5);
    await unmount();
    record("cleanup-state-before-comparisons", {
      inputConnected: element.isConnected,
      childCount: container.childNodes.length,
    });
    assert.equal(current().isUnmounted, true);
    assert.equal(element.isConnected, false);
    assert.equal(container.childNodes.length, 0);
    assert.deepEqual(warnings, []);
    assert.deepEqual(errors, []);
    outcome = {
      initial,
      firstClick,
      beforeProgrammaticRender,
      programmatic,
      secondClick,
      thirdClick,
      events,
      setterReturn: primitive(setterReturn),
      setupInvocations,
      renders,
      unmounted: true,
      warnings,
      errors,
    };
  } catch (error) {
    retain(error, "runtime-first-failure");
  } finally {
    await unmount();
    try {
      container.remove();
      record("host-removed");
    } catch (error) {
      retain(error, "host-cleanup-failure");
    }
  }
  if (failed) throw firstFailure;
  return outcome;
}
