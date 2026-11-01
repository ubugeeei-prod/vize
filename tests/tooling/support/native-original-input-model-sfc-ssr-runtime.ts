// Actual server-renderer executions; original module bytes stay independently retained.
import assert from "node:assert/strict";
import {
  loadModelSsrComponent,
  object,
  openModelSsr,
  primitive,
  type ModelSsrContext,
  type ModelSsrFixture,
  type ObjectValue,
  type SsrModelApp,
  type SsrModelComponent,
  type SsrModelInstance,
} from "./native-original-input-model-sfc-ssr-boundary.ts";
import { diagnosticValue } from "./native-original-for-constant-sfc-dom-observation.ts";

export function modelSsrError(error: unknown) {
  const value = error && (typeof error === "object" || typeof error === "function") ? error : null;
  const field = (name: string): unknown => (value ? Reflect.get(value, name) : undefined);
  return {
    name: diagnosticValue(field("name")),
    message: diagnosticValue(field("message")),
    stack: diagnosticValue(field("stack")),
    code: diagnosticValue(field("code")),
    operator: diagnosticValue(field("operator")),
    actualPresent: value ? Object.hasOwn(value, "actual") : false,
    actual: diagnosticValue(field("actual")),
    expectedPresent: value ? Object.hasOwn(value, "expected") : false,
    expected: diagnosticValue(field("expected")),
    cause: diagnosticValue(field("cause")),
    original: diagnosticValue(error),
  };
}
function descriptors(state: ObjectValue, fixture: ModelSsrFixture) {
  return fixture.bindings.map((name) => {
    const descriptor = Object.getOwnPropertyDescriptor(state, name);
    assert(descriptor);
    return {
      name,
      enumerable: descriptor.enumerable,
      configurable: descriptor.configurable,
      getter: typeof descriptor.get,
      setter: typeof descriptor.set,
    };
  });
}
const forbiddenContext = new Proxy(Object.create(null) as ObjectValue, {
  get(_target, name) {
    throw Error(`SSR Model attempted _ctx fallback ${String(name)}`);
  },
});

export async function executeInputModelSsr(
  code: string,
  fixture: ModelSsrFixture,
  contexts: ModelSsrContext[],
  attempt: ObjectValue,
) {
  const { runtime, renderer } = openModelSsr();
  const component = await loadModelSsrComponent(code);
  const actualCases: ObjectValue[] = [];
  attempt.actualCases = actualCases;
  const outcomes: ObjectValue[] = [];
  let failed = false;
  let firstFailure: unknown;
  for (const context of contexts) {
    for (const phase of ["initial", "updated"] as const) {
      const observed: ObjectValue[] = [];
      const row: ObjectValue = {
        context: context.id,
        phase,
        state: "attempting",
        observations: observed,
      };
      actualCases.push(row);
      try {
        const result = await scenario(
          component,
          fixture,
          context,
          phase,
          runtime,
          renderer,
          observed,
        );
        row.outcome = result;
        row.state = "observed-unqualified";
        outcomes.push(result);
      } catch (error) {
        row.state = "failed";
        row.error = modelSsrError(error);
        if (!failed) {
          failed = true;
          firstFailure = error;
        }
      }
    }
  }
  if (failed) throw firstFailure;
  return { contexts: outcomes };
}

async function scenario(
  component: SsrModelComponent,
  fixture: ModelSsrFixture,
  context: ModelSsrContext,
  phase: "initial" | "updated",
  runtime: ReturnType<typeof openModelSsr>["runtime"],
  renderer: ReturnType<typeof openModelSsr>["renderer"],
  observations: ObjectValue[],
) {
  const originalSetup = component.setup;
  const originalRender = component.ssrRender;
  let setupInvocations = 0;
  let renderInvocations = 0;
  let sourceRenderInvocations = 0;
  let explicitSetterInvocations = 0;
  let instance: SsrModelInstance | null = null;
  let state: ObjectValue | null = null;
  let app: SsrModelApp | undefined;
  let receipt: ObjectValue | undefined;
  let outcome: ObjectValue | undefined;
  let failed = false;
  let firstFailure: unknown;
  let runtimeFailed = false;
  let runtimeFailure: unknown;
  const warnings: string[] = [];
  const errors: unknown[] = [];
  const observe = (stage: string, value: unknown) =>
    observations.push({ stage, value: diagnosticValue(value) });
  component.setup = function (this: unknown, props: unknown, setupContext: unknown) {
    setupInvocations += 1;
    instance = runtime.getCurrentInstance();
    const result: unknown = originalSetup.call(this, props, setupContext);
    observe("setup-returned", {
      instancePresent: instance !== null,
      actualApp: instance?.appContext.app === app,
      actualSetup: instance?.type.setup === component.setup,
      actualSsrRender: instance?.type.ssrRender === component.ssrRender,
      props,
      state: result,
      descriptors:
        result && typeof result === "object" ? Object.getOwnPropertyDescriptors(result) : null,
    });
    assert(instance);
    assert.equal(instance.appContext.app, app);
    assert.equal(instance.type.setup, component.setup);
    assert.equal(instance.type.ssrRender, component.ssrRender);
    object(result);
    state = result;
    assert.deepEqual(Object.keys(state), fixture.bindings);
    assert.deepEqual(Object.getOwnPropertyDescriptor(state, "__isScriptSetup"), {
      value: true,
      writable: false,
      enumerable: false,
      configurable: false,
    });
    for (const name of fixture.bindings) {
      const descriptor = Object.getOwnPropertyDescriptor(state, name);
      assert.equal(typeof descriptor?.get, "function");
      assert.equal(typeof descriptor?.set, "function");
    }
    assert.deepEqual(primitive(state[fixture.binding]), fixture.initial);
    return state;
  };
  component.ssrRender = function (this: unknown, ...args: unknown[]) {
    renderInvocations += 1;
    observe("server-entry", {
      argumentCount: args.length,
      setupInvocations,
      renderInvocations,
      sourceRenderInvocations,
      actualProxy: args[0] === instance?.proxy,
      actualParent: args[2] === instance,
      actualProps: args[4] === instance?.props,
      actualSixthSetup: args[5] === instance?.setupState,
      actualData: args[6] === instance?.data,
      actualOptions: args[7] === instance?.ctx,
      attrs: args[3],
      state: state && primitive(state[fixture.binding]),
    });
    assert(instance && state);
    assert.equal(args.length, 8);
    assert.equal(args[0], instance.proxy);
    assert.equal(args[2], instance);
    assert.equal(args[4], instance.props);
    assert.equal(args[5], instance.setupState);
    assert.equal(args[6], instance.data);
    assert.equal(args[7], instance.ctx);
    const actualSetup = args[5];
    object(actualSetup);
    const initialValue = primitive(state[fixture.binding]);
    if (phase === "updated") {
      explicitSetterInvocations += 1;
      const set = Reflect.set(actualSetup, fixture.binding, fixture.programmatic);
      observe("actual-setup-setter", {
        set,
        value: state[fixture.binding],
        explicitSetterInvocations,
      });
      assert.equal(set, true);
      assert.equal(state[fixture.binding], fixture.programmatic);
    }
    const before = primitive(state[fixture.binding]);
    const chunks: string[] = [];
    const directArgs = [...args];
    directArgs[0] = forbiddenContext;
    directArgs[1] = (chunk: unknown) => {
      observe("direct-push", chunk);
      assert(typeof chunk === "string");
      chunks.push(chunk);
    };
    sourceRenderInvocations += 1;
    const directResult: unknown = originalRender.call(this, ...directArgs);
    observe("direct-render-returned", {
      result: directResult,
      before,
      after: primitive(state[fixture.binding]),
      chunks,
    });
    assert.equal(directResult, undefined);
    assert.deepEqual(primitive(state[fixture.binding]), before);
    const actualArgs = [...args];
    actualArgs[0] = forbiddenContext;
    sourceRenderInvocations += 1;
    const result: unknown = originalRender.call(this, ...actualArgs);
    const after = primitive(state[fixture.binding]);
    receipt = {
      context: context.id,
      phase,
      inputProps: context.props,
      initialValue,
      before,
      after,
      descriptors: descriptors(state, fixture),
      marker: Object.getOwnPropertyDescriptor(state, "__isScriptSetup"),
      argumentCount: args.length,
      actualSixthSetupState: true,
      attrs: diagnosticValue(args[3]),
      direct: chunks.join(""),
      setupInvocations,
      renderInvocations,
      sourceRenderInvocations,
      explicitSetterInvocations,
    };
    observe("actual-render-returned", { result, ...receipt });
    assert.equal(result, undefined);
    assert.deepEqual(after, before);
    return result;
  };
  try {
    app = runtime.createSSRApp(component, structuredClone(context.props));
    app.config.warnHandler = (warning) => {
      warnings.push(warning);
      observe("warning", warning);
    };
    app.config.errorHandler = (error) => {
      errors.push(modelSsrError(error));
      observe("error", modelSsrError(error));
      if (!runtimeFailed) {
        runtimeFailed = true;
        runtimeFailure = error;
      }
    };
    const serverContext: ObjectValue = {};
    const html = await renderer.renderToString(app, serverContext);
    observe("renderToString-returned", { html, serverContext, receipt, warnings, errors });
    if (runtimeFailed) throw runtimeFailure;
    assert.deepEqual(warnings, []);
    assert.deepEqual(errors, []);
    assert(receipt);
    assert.equal(setupInvocations, 1);
    assert.equal(renderInvocations, 1);
    assert.equal(
      sourceRenderInvocations,
      2,
      "one direct proof plus one actual server push invocation",
    );
    assert.equal(explicitSetterInvocations, phase === "updated" ? 1 : 0);
    assert.equal(receipt.direct, html);
    outcome = { ...receipt, html, serverContext, warnings, errors };
  } catch (error) {
    failed = true;
    firstFailure = error;
    observe("first-failure", modelSsrError(error));
  } finally {
    try {
      component.setup = originalSetup;
      component.ssrRender = originalRender;
      observe("wrapper-cleanup", {
        setupRestored: component.setup === originalSetup,
        renderRestored: component.ssrRender === originalRender,
      });
      assert.equal(component.setup, originalSetup);
      assert.equal(component.ssrRender, originalRender);
    } catch (error) {
      observe("cleanup-error", modelSsrError(error));
      if (!failed) {
        failed = true;
        firstFailure = error;
      }
    }
  }
  if (failed) throw firstFailure;
  assert(outcome);
  return { ...outcome, wrapperCleanup: { setupRestored: true, renderRestored: true } };
}
