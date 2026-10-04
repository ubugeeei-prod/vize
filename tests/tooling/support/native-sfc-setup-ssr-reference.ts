import assert from "node:assert/strict";
import { checkMap, contexts, core, renderer } from "./native-sfc-ssr-reference.ts";
import { loadSetupModule, type SetupGraph } from "./native-sfc-setup-ssr-loader.ts";
export {
  checkOfficialSetupMap,
  primarySetupBindings,
  removeSetupMarker,
  insertSetupRead,
  insertConstSetter,
  stockSetupSfc,
} from "./native-sfc-setup-ssr-loader.ts";

const values = (state: any, bindings: any[]) =>
  Object.fromEntries(
    bindings.map(({ name }) => {
      const value = state[name];
      assert(value === null || ["string", "number", "boolean", "bigint"].includes(typeof value));
      return [name, typeof value === "bigint" ? { bigint: String(value) } : value];
    }),
  );
const forbidden = new Proxy(Object.create(null), {
  get(_, name) {
    throw Error(`SSR setup _ctx fallback ${String(name)}`);
  },
});

export async function executeExternalSetup(
  graph: SetupGraph,
  bindings: any[],
  updates: any = {},
  native = false,
  ignoredBindings: string[] = [],
) {
  const component = await loadSetupModule(graph);
  const originalSetup = component.setup,
    originalRender = component.ssrRender;
  assert.equal(typeof originalRender, "function");
  const executions: any[] = [];
  const restorations: (() => void)[] = [];
  try {
    for (const props of contexts) {
      const phases: any = {};
      for (const phase of ["initial", ...(Object.keys(updates).length ? ["updated"] : [])]) {
        let state: any,
          setupInvocations = 0,
          renderInvocations = 0,
          executingSsr = false,
          receipt: any;
        component.setup = (actualProps: any, context: any) => {
          setupInvocations += 1;
          state = originalSetup(actualProps, context);
          assert(state && typeof state === "object");
          assert.deepEqual(
            Object.keys(state),
            bindings.map((row) => row.name),
          );
          assert.deepEqual(Object.getOwnPropertyDescriptor(state, "__isScriptSetup"), {
            value: true,
            writable: false,
            enumerable: false,
            configurable: false,
          });
          for (const { name, kind } of bindings) {
            const descriptor = Object.getOwnPropertyDescriptor(state, name);
            assert(["Const", "Let", "Var"].includes(kind));
            if (kind !== "Const") {
              assert.equal(typeof descriptor?.get, "function");
              assert.equal(typeof descriptor?.set, "function");
            } else if (native) {
              assert.equal(typeof descriptor?.get, "function");
              assert.equal(typeof descriptor?.set, "undefined");
              const before = state[name];
              assert.equal(Reflect.set(state, name, Symbol("forbidden const mutation")), false);
              assert.equal(state[name], before);
            }
          }
          for (const name of ignoredBindings) {
            const descriptor = Object.getOwnPropertyDescriptor(state, name);
            assert.equal(typeof descriptor?.get, "function");
            assert.equal(descriptor?.configurable, true);
            restorations.push(() => Object.defineProperty(state, name, descriptor!));
            Object.defineProperty(state, name, {
              ...descriptor,
              get() {
                assert(!executingSsr, `SSR evaluated ignored handler binding ${name}`);
                return descriptor!.get!.call(this);
              },
            });
          }
          return state;
        };
        component.ssrRender = (...args: any[]) => {
          renderInvocations += 1;
          assert.equal(
            args.length,
            8,
            "actual server renderer calls the full eight-argument entry",
          );
          assert.equal(args[0], args[2].proxy);
          assert.equal(args[4], args[2].props);
          assert.equal(
            args[5],
            args[2].setupState,
            "sixth argument is the actual instance setup state",
          );
          assert.equal(args[6], args[2].data);
          assert.equal(args[7], args[2].ctx);
          assert.deepEqual(values(args[5], bindings), values(state, bindings));
          const initialValues = values(state, bindings);
          if (phase === "updated")
            for (const [name, value] of Object.entries(updates)) {
              assert(bindings.some((row) => row.name === name && row.kind !== "Const"));
              assert.equal(Reflect.set(args[5], name, value), true);
              assert.equal(
                state[name],
                value,
                "real setup-state setter updates the retained lexical binding",
              );
            }
          const stateValues = values(state, bindings),
            chunks: string[] = [];
          const directArgs = [...args];
          directArgs[0] = forbidden;
          directArgs[1] = (chunk: string) => {
            assert.equal(typeof chunk, "string");
            chunks.push(chunk);
          };
          const invoke = (parameters: any[]) => {
            executingSsr = true;
            try {
              return originalRender(...parameters);
            } finally {
              executingSsr = false;
            }
          };
          assert.equal(invoke(directArgs), undefined);
          assert.deepEqual(
            values(state, bindings),
            stateValues,
            "ignored handlers and SSR reads preserve state",
          );
          receipt = {
            setupInvocations,
            renderInvocations,
            argumentCount: args.length,
            actualSixthSetupState: true,
            initialValues,
            stateValues,
            direct: chunks.join(""),
          };
          const actualArgs = [...args];
          actualArgs[0] = forbidden;
          const result = invoke(actualArgs);
          assert.deepEqual(values(state, bindings), stateValues);
          return result;
        };
        const app = core.createSSRApp(component, props),
          warnings: string[] = [],
          context: any = {};
        app.config.warnHandler = (warning: string) => warnings.push(warning);
        const html = await renderer.renderToString(app, context);
        assert.equal(setupInvocations, 1);
        assert.equal(renderInvocations, 1);
        assert.deepEqual(warnings, []);
        assert.equal(receipt.direct, html);
        phases[phase] = { html, modules: [...(context.modules ?? [])], warnings, ...receipt };
      }
      executions.push(phases);
    }
  } finally {
    for (const restore of restorations) restore();
    component.setup = originalSetup;
    component.ssrRender = originalRender;
  }
  return executions;
}

export async function executeInlineSetup(graph: SetupGraph) {
  const component = await loadSetupModule(graph),
    originalSetup = component.setup;
  assert.equal(component.__ssrInlineRender, true);
  const executions = [];
  try {
    for (const props of contexts) {
      let setupInvocations = 0;
      component.setup = (actualProps: any, context: any) => {
        setupInvocations += 1;
        const result = originalSetup(actualProps, context);
        assert.equal(typeof result, "function");
        return result;
      };
      const app = core.createSSRApp(component, props),
        warnings: string[] = [],
        context: any = {};
      app.config.warnHandler = (message: string) => warnings.push(message);
      const html = await renderer.renderToString(app, context);
      assert.equal(setupInvocations, 1);
      assert.deepEqual(warnings, []);
      executions.push({ html, modules: [...(context.modules ?? [])], warnings, setupInvocations });
    }
  } finally {
    component.setup = originalSetup;
  }
  return executions;
}

function originalBytes(source: string, span: any) {
  assert(Number.isSafeInteger(span.start) && Number.isSafeInteger(span.end));
  const bytes = Buffer.from(source);
  assert(span.start >= 0 && span.start <= span.end && span.end <= bytes.length);
  for (const offset of [span.start, span.end])
    assert(
      Buffer.from(bytes.subarray(0, offset).toString("utf8")).equals(bytes.subarray(0, offset)),
      "custody endpoints retain original UTF-8 boundaries",
    );
  return bytes.subarray(span.start, span.end).toString("utf8");
}

export function checkSetupCustody(row: any, expected: any) {
  assert.equal(row.name, expected.name);
  assert.equal(row.filename, expected.filename);
  assert.equal(row.source, expected.source);
  assert.equal(
    row.code,
    row.noLinksCode,
    "Recorded and NoLinks emit the same complete module bytes",
  );
  assert.deepEqual(JSON.parse(row.mapText), row.mapValue);
  assert.equal(row.setup.rawProgram, originalBytes(row.source, row.setup.sourceSpan));
  assert(Number.isSafeInteger(row.setup.unit) && Number.isSafeInteger(row.setup.scope));
  assert.deepEqual(
    row.setup.bindings.map(({ name, kind }: any) => ({ name, kind })),
    expected.bindings,
  );
  const ids = new Set(row.setup.bindings.map((binding: any) => binding.id));
  assert.equal(ids.size, row.setup.bindings.length);
  for (const binding of row.setup.bindings) {
    assert(Number.isSafeInteger(binding.id));
    originalBytes(row.source, binding.span);
    assert(
      binding.span.start >= row.setup.sourceSpan.start &&
        binding.span.end <= row.setup.sourceSpan.end,
    );
  }
  let previous = row.setup.sourceSpan.start;
  const runtimeSpans = [];
  for (const annotation of row.setup.annotations) {
    assert(annotation.span.start >= previous && annotation.span.end <= row.setup.sourceSpan.end);
    assert(originalBytes(row.source, annotation.span).startsWith(":"));
    if (previous < annotation.span.start)
      runtimeSpans.push({ start: previous, end: annotation.span.start });
    previous = annotation.span.end;
  }
  if (previous < row.setup.sourceSpan.end)
    runtimeSpans.push({ start: previous, end: row.setup.sourceSpan.end });
  for (const span of runtimeSpans) {
    const link = row.links.find(
      (link: any) =>
        link.segment && link.authored.start === span.start && link.authored.end === span.end,
    );
    assert(link, "every original runtime Program segment retains its complete source link");
    assert.equal(originalBytes(row.code, link.generated), originalBytes(row.source, span));
  }
  for (const interpolation of row.setup.interpolations) {
    assert(
      Number.isSafeInteger(interpolation.node) && Number.isSafeInteger(interpolation.regionScope),
    );
    assert.equal(originalBytes(row.source, interpolation.span), interpolation.raw);
    assert(interpolation.preparedSpan, "original interpolation retains its prepared source span");
    assert(
      interpolation.preparedSpan.start >= interpolation.span.start &&
        interpolation.preparedSpan.end <= interpolation.span.end,
    );
    assert.equal(originalBytes(row.source, interpolation.preparedSpan), interpolation.decoded);
    assert.equal(
      interpolation.regionScope,
      row.setup.scope,
      "root template resolution retains the original setup unit scope",
    );
    for (const read of interpolation.reads) {
      assert.equal(originalBytes(interpolation.decoded, read.decodedSpan), read.name);
      assert.equal(originalBytes(row.source, read.span), read.name);
      assert.equal(read.span.start, interpolation.preparedSpan.start + read.decodedSpan.start);
      assert.equal(read.span.end, interpolation.preparedSpan.start + read.decodedSpan.end);
      const binding = row.setup.bindings.find((binding: any) => binding.id === read.binding);
      assert(binding && binding.name === read.name);
      assert.equal(read.declarationScope, row.setup.scope);
      assert.equal(read.kind, binding.kind === "Const" ? "SetupConst" : "SetupLet");
    }
  }
  for (const handler of row.setup.handlers) {
    assert.equal(originalBytes(row.source, handler.span), handler.raw);
    assert.equal(handler.raw, handler.decoded);
    assert.equal(handler.raw, "value");
    assert(!row.code.includes("$event"));
  }
  assert.equal(row.setup.handlers.length, expected.handler ? 1 : 0);
  return checkMap({ ...row, map: row.mapValue });
}
