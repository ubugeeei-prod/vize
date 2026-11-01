// Actual Vue DOM Model behavior, not a substitute renderer or setup state.
import assert from "node:assert/strict";
import {
  assertModelFixture,
  dataUrl,
  modelModuleSource,
  primitive,
  primitiveValue,
  type ModelComponent,
  type ModelFixture,
  type ModelInstance,
  type ModelMode,
  type ModelVNode,
  type openModelDom,
} from "./native-original-input-model-sfc-dom-environment.ts";
import {
  diagnosticValue,
  runtimeErrorDetails,
} from "./native-original-for-constant-sfc-dom-observation.ts";

type Dom = Awaited<ReturnType<typeof openModelDom>>;
export function modelError(error: unknown) {
  return {
    ...runtimeErrorDetails(error),
    cause: diagnosticValue(
      error !== null && typeof error === "object" ? Reflect.get(error, "cause") : undefined,
    ),
  };
}

export async function executeInputModel(
  dom: Dom,
  code: string,
  fixture: ModelFixture,
  mode: ModelMode,
  actor: "official" | "native",
  observe: (observation: unknown) => void,
) {
  assertModelFixture(fixture);
  assert.equal(process.env.NODE_ENV ?? "development", mode);
  const { window, runtime } = dom;
  const rewritten = modelModuleSource(code);
  const loaded: unknown = (await import(dataUrl(rewritten))).default;
  assert(loaded && typeof loaded === "object");
  assert.equal(typeof Reflect.get(loaded, "setup"), "function");
  const original = loaded as ModelComponent;
  const component = { ...original };
  const originalSetup = original.setup;
  const originalRender = original.render;
  const container = window.document.createElement("div");
  window.document.body.appendChild(container);
  let instance: ModelInstance | null = null;
  let setupInvocations = 0;
  let renders = 0;
  let setupResult: unknown;
  function counted(render: (this: unknown, ...args: unknown[]) => unknown) {
    return function (this: unknown, ...args: unknown[]) {
      renders++;
      const result = render.apply(this, args);
      record("render-before-comparisons", { vnode: observedVNode(result) });
      return result;
    };
  }
  component.setup = (props, context) => {
    setupInvocations++;
    instance = runtime.getCurrentInstance();
    record("setup-instance-before-comparisons");
    assert(instance && instance.type.setup === component.setup);
    setupResult = originalSetup(props, context);
    record("actual-setup-return-before-comparisons", { returned: diagnosticValue(setupResult) });
    if (typeof setupResult === "function")
      return counted(setupResult as (this: unknown, ...args: unknown[]) => unknown);
    return setupResult;
  };
  if (originalRender) component.render = counted(originalRender);
  const warnings: string[] = [];
  const errors: unknown[] = [];
  const app = runtime.createApp(component);
  app.config.warnHandler = (warning) => warnings.push(warning);
  app.config.errorHandler = (error) => errors.push(modelError(error));
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
  const vnode = (): ModelVNode => {
    const node = current().subTree;
    assert(runtime.isVNode(node));
    return node;
  };
  function observedVNode(value: unknown): unknown {
    if (!runtime.isVNode(value)) return { isVNode: false, value: diagnosticValue(value) };
    return {
      type: value.type,
      key: value.key,
      props: diagnosticValue(value.props),
      children: diagnosticValue(value.children),
      patchFlag: value.patchFlag,
      dynamicProps: value.dynamicProps,
      directives:
        value.dirs?.map((row) => ({
          actualTextModel: row.dir === runtime.vModelText,
          value: primitive(row.value),
          oldValue: primitive(row.oldValue),
          arg: primitive(row.arg),
          modifiers: diagnosticValue(row.modifiers),
        })) ?? null,
    };
  }
  function record(phase: string, details?: unknown) {
    observe(
      diagnosticValue({
        phase,
        actor,
        mode,
        setupInvocations,
        renders,
        host: container.outerHTML,
        connected: container.isConnected,
        sameApp: instance?.appContext.app === app,
        sameSetup: instance?.type.setup === component.setup,
        isUnmounted: instance?.isUnmounted,
        warnings: [...warnings],
        errors: [...errors],
        details,
      }),
    );
  }
  function snapshot(
    phase: string,
    expectedBinding: unknown,
    expectedValue: string,
    composing: boolean,
  ) {
    const node = vnode();
    const element = input();
    const props = node.props;
    const directives = node.dirs;
    record(phase, {
      vnode: observedVNode(node),
      props:
        props &&
        Object.keys(props).map((name) => ({
          name,
          descriptor: Object.getOwnPropertyDescriptor(props, name),
        })),
      directiveValues: directives?.map((row) => ({
        value: primitive(row.value),
        oldValue: primitive(row.oldValue),
      })),
      dom: {
        value: element.value,
        type: element.type,
        defaultValue: element.defaultValue,
        composing: Reflect.get(element, "composing"),
      },
    });
    assert.equal(node.type, "input");
    assert.equal(node.key, null);
    assert.equal(node.children, null);
    assert.equal(node.patchFlag, 8);
    assert.deepEqual(node.dynamicProps, ["onUpdate:modelValue"]);
    assert(props);
    assert.deepEqual(Object.keys(props), ["onUpdate:modelValue"]);
    const setter: unknown = props["onUpdate:modelValue"];
    assert.equal(typeof setter, "function");
    assert(directives && directives.length === 1);
    const [directive] = directives;
    assert(directive);
    assert.equal(directive.dir, runtime.vModelText);
    assert.equal(directive.arg, undefined);
    assert.deepEqual(directive.modifiers, {});
    assert(
      Object.is(directive.value, expectedBinding),
      "actual evaluated lexical read matches exact primitive type/value",
    );
    assert.equal(element.value, expectedValue);
    assert.equal(element.type, "text");
    assert.equal(element.defaultValue, "");
    assert.equal(element.childNodes.length, 0);
    assert.equal(element.isConnected, true);
    assert.equal(Boolean(Reflect.get(element, "composing")), composing);
    assert.deepEqual(element.getAttributeNames(), []);
    const result = {
      phase,
      binding: primitive(directive.value),
      oldBinding: primitive(directive.oldValue),
      vnode: {
        type: node.type,
        key: node.key,
        children: node.children,
        patchFlag: node.patchFlag,
        dynamicProps: node.dynamicProps,
        propKeys: Object.keys(props),
        modifiers: directive.modifiers,
      },
      host: {
        html: container.outerHTML,
        inputHtml: element.outerHTML,
        value: element.value,
        type: element.type,
        defaultValue: element.defaultValue,
        composing,
        connected: element.isConnected,
      },
      setupInvocations,
      renders,
    };
    return result;
  }
  async function forceRender() {
    await runtime.nextTick();
    const previous = renders;
    current().proxy.$forceUpdate();
    await runtime.nextTick();
    assert.equal(
      renders,
      previous + 1,
      "bare lexical state is observed by one explicit actual render",
    );
  }
  function dispatch(type: string) {
    const event = new window.Event(type, { bubbles: true });
    const dispatched = input().dispatchEvent(event);
    record("event-dispatched", {
      type,
      dispatched,
      defaultPrevented: event.defaultPrevented,
      isTrusted: primitive(Reflect.get(event, "isTrusted")),
    });
    assert(dispatched && !event.defaultPrevented);
    return {
      type,
      dispatched,
      defaultPrevented: event.defaultPrevented,
      isTrusted: primitive(Reflect.get(event, "isTrusted")),
    };
  }
  let failed = false;
  let firstFailure: unknown;
  let unmountAttempted = false;
  let outcome: unknown;
  function retainFailure(error: unknown, phase: string) {
    if (!failed) {
      failed = true;
      firstFailure = error;
    }
    try {
      record(phase, { error: modelError(error) });
    } catch {
      /* Preserve the first failure. */
    }
  }
  try {
    app.mount(container);
    record("mounted-before-comparisons");
    assert.equal(current().appContext.app, app);
    assert.equal(setupInvocations, 1);
    assert.equal(renders, 1);
    const initialValue = primitiveValue(fixture.initial);
    const observedEvents: unknown[] = [];
    const actualInput = input();
    for (const type of ["input", "compositionstart", "compositionend"]) {
      actualInput.addEventListener(type, (event) => {
        const observed = {
          type: event.type,
          value: actualInput.value,
          composing: Boolean(Reflect.get(actualInput, "composing")),
          bubbles: event.bubbles,
          isTrusted: primitive(Reflect.get(event, "isTrusted")),
          sameTarget: event.target === actualInput,
          sameCurrentTarget: event.currentTarget === actualInput,
        };
        observedEvents.push(observed);
        record("actual-dom-event-observer", observed);
      });
    }
    const initial = snapshot(
      "initial",
      initialValue,
      initialValue == null ? "" : String(initialValue),
      false,
    );
    const element = input();
    const events: unknown[] = [];
    element.value = fixture.input;
    events.push(dispatch("input"));
    await forceRender();
    const event = snapshot("input", fixture.input, fixture.input, false);
    const setter: unknown = vnode().props?.["onUpdate:modelValue"];
    assert(typeof setter === "function");
    const setterReturn: unknown = setter(fixture.programmatic);
    record("actual-vnode-setter-invoked", { returnValue: primitive(setterReturn) });
    assert.equal(setterReturn, fixture.programmatic, "actual generated assignment result");
    await forceRender();
    const programmatic = snapshot(
      "programmatic",
      fixture.programmatic,
      fixture.programmatic,
      false,
    );
    events.push(dispatch("compositionstart"));
    element.value = fixture.composing;
    events.push(dispatch("input"));
    await forceRender();
    const duringComposition = snapshot(
      "composing-input-ignored",
      fixture.programmatic,
      fixture.composing,
      true,
    );
    events.push(dispatch("compositionend"));
    await forceRender();
    const compositionEnd = snapshot(
      "compositionend-input",
      fixture.composing,
      fixture.composing,
      false,
    );
    assert.deepEqual(observedEvents, [
      {
        type: "input",
        value: fixture.input,
        composing: false,
        bubbles: true,
        isTrusted: { kind: "undefined" },
        sameTarget: true,
        sameCurrentTarget: true,
      },
      {
        type: "compositionstart",
        value: fixture.programmatic,
        composing: true,
        bubbles: true,
        isTrusted: { kind: "undefined" },
        sameTarget: true,
        sameCurrentTarget: true,
      },
      {
        type: "input",
        value: fixture.composing,
        composing: true,
        bubbles: true,
        isTrusted: { kind: "undefined" },
        sameTarget: true,
        sameCurrentTarget: true,
      },
      {
        type: "input",
        value: fixture.composing,
        composing: false,
        bubbles: false,
        isTrusted: { kind: "undefined" },
        sameTarget: true,
        sameCurrentTarget: true,
      },
      {
        type: "compositionend",
        value: fixture.composing,
        composing: false,
        bubbles: true,
        isTrusted: { kind: "undefined" },
        sameTarget: true,
        sameCurrentTarget: true,
      },
    ]);
    assert.equal(input(), element, "the actual DOM input survives every update");
    assert.equal(setupInvocations, 1);
    assert.equal(renders, 5);
    assert.deepEqual(warnings, []);
    assert.deepEqual(errors, []);
    unmountAttempted = true;
    app.unmount();
    await runtime.nextTick();
    record("unmounted-before-comparisons", { inputConnected: element.isConnected });
    assert.equal(current().isUnmounted, true);
    assert.equal(element.isConnected, false);
    assert.equal(container.childNodes.length, 0);
    outcome = {
      initial,
      event,
      programmatic,
      duringComposition,
      compositionEnd,
      events,
      observedEvents,
      setterReturn: primitive(setterReturn),
      setupInvocations,
      renders,
      unmounted: true,
      warnings,
      errors,
    };
  } catch (error) {
    retainFailure(error, "runtime-failure");
  } finally {
    if (!unmountAttempted) {
      try {
        app.unmount();
        await runtime.nextTick();
        record("failure-cleanup-after-unmount");
      } catch (error) {
        retainFailure(error, "cleanup-failure");
      }
    }
    container.remove();
  }
  if (failed) throw firstFailure;
  return outcome;
}
