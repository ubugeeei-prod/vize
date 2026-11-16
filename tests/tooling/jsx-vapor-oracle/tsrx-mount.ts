// Mounted parity for the TSRX Vue profile (#6858): the pinned reference
// pipeline and Vize's DOM output run against one Vue runtime in happy-dom, and
// their normalized DOM observations must be equal after the mount and after
// every scripted step. The decision record is
// docs/davinci/decisions/2026-11-09-6858-tsrx-vue-profile.md ("What parity means").
//
// Reference: `@tsrx/vue` → the `vue-jsx-vapor` Vite transforms with the options
// the reference Vite plugin passes (`macros: true`, runtime module
// `vue-jsx-vapor`) → Vapor. Vize: `vize_l4::lang::tsrx::compile` (DOM) → VDOM.
//
// It lives beside the vue-jsx-vapor oracle, whose packages it shares; the
// pinned `@tsrx/vue` comes through `tests/tooling/tsrx-oracle`.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire, stripTypeScriptTypes } from "node:module";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import { compileFunction } from "node:vm";
import { transformSync, type NodePath, type PluginObj, type types as T } from "@babel/core";
import { Window } from "happy-dom";
import vueJsxVapor from "vue-jsx-vapor/vite";

import { compileReference, repoRoot } from "../tsrx-oracle/reference.ts";

const window = new Window();
const scope = globalThis as unknown as Record<string, unknown>;
for (const key of [
  "document",
  "Node",
  "Text",
  "Comment",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Event",
]) {
  scope[key] = (window as unknown as Record<string, unknown>)[key];
}
scope.window = window;

const require = createRequire(import.meta.url);
// Node's `vue` condition omits Vapor; both pipelines share the with-Vapor build.
const Vue = (await import(
  pathToFileURL(require.resolve("vue/dist/vue.runtime-with-vapor.esm-browser.prod.js")).href
)) as Record<string, any>;

/** Rewrite imports to runtime objects and collect named exports. */
function link(code: string, filename: string, runtimes: Record<string, string>): string {
  const exported: string[] = [];
  const plugin = ({ types: t }: { types: typeof T }): PluginObj => ({
    visitor: {
      ImportDeclaration(path: NodePath<T.ImportDeclaration>) {
        const runtime = runtimes[path.node.source.value];
        assert.ok(runtime, `${filename}: unexpected import ${path.node.source.value}`);
        const declarations = path.node.specifiers.map((specifier) => {
          const value =
            specifier.type === "ImportNamespaceSpecifier"
              ? t.identifier(runtime)
              : t.memberExpression(
                  t.identifier(runtime),
                  specifier.type === "ImportDefaultSpecifier"
                    ? t.identifier("default")
                    : specifier.imported,
                );
          return t.variableDeclaration("const", [t.variableDeclarator(specifier.local, value)]);
        });
        path.replaceWithMultiple(declarations);
      },
      ExportNamedDeclaration(path: NodePath<T.ExportNamedDeclaration>) {
        const declaration = path.node.declaration;
        if (!declaration) {
          assert.equal(path.node.source, null, `${filename}: no re-exports`);
          for (const specifier of path.node.specifiers) {
            assert.equal(specifier.type, "ExportSpecifier", filename);
            const name = t.isIdentifier(specifier.exported)
              ? specifier.exported.name
              : specifier.exported.value;
            exported.push(`${name}: ${specifier.local.name}`);
          }
          path.remove();
          return;
        }
        for (const name of Object.keys(t.getBindingIdentifiers(declaration))) exported.push(name);
        path.replaceWith(declaration);
      },
    },
  });
  const result = transformSync(code, {
    configFile: false,
    babelrc: false,
    filename,
    plugins: [plugin],
  });
  assert.ok(result?.code, filename);
  return `${result.code}\nreturn { ${exported.join(", ")} };`;
}

function evaluate(code: string, filename: string, runtimes: Record<string, unknown>) {
  const names = Object.keys(runtimes);
  const body = link(code, filename, Object.fromEntries(names.map((name, i) => [name, `__r${i}`])));
  return compileFunction(
    body,
    names.map((_, i) => `__r${i}`),
    { filename },
  )(...Object.values(runtimes)) as Record<string, unknown>;
}

// The published vue-jsx-vapor runtime, linked to the same Vue module.
const runtimeDir = dirname(
  createRequire(require.resolve("vue-jsx-vapor")).resolve("@vue-jsx-vapor/runtime"),
);
const vaporRuntime = evaluate(
  `${readFileSync(join(runtimeDir, "vapor.js"), "utf8")}`,
  "vue-jsx-vapor/vapor.js",
  { vue: Vue },
);
const propsRuntime = evaluate(
  readFileSync(join(runtimeDir, "props.js"), "utf8"),
  "vue-jsx-vapor/props.js",
  {
    vue: Vue,
  },
);

/** The reference pipeline's JavaScript for one module. */
export async function referenceCode(source: string, file: string): Promise<string> {
  const compiled = compileReference(file, source);
  assert.ok(compiled.code !== null, `${file}: the reference compiles it`);
  let code = compiled.code;
  const id = `/tsrx-oracle/${file}.tsx`;
  const plugins = [
    vueJsxVapor({ macros: true, compiler: { runtimeModuleName: "vue-jsx-vapor" } }),
  ].flat(Infinity) as Array<{ transform?: unknown }>;
  for (const plugin of plugins) {
    const hook = plugin.transform as
      | ((code: string, id: string) => unknown)
      | { handler: (code: string, id: string) => unknown }
      | undefined;
    const run = typeof hook === "function" ? hook : hook?.handler;
    if (!run) continue;
    const result = (await run.call(
      { warn() {}, error: (e: unknown) => assert.fail(String(e)) },
      code,
      id,
    )) as string | { code: string } | null | undefined;
    if (result) code = typeof result === "string" ? result : result.code;
  }
  return code;
}

type Step = { props: Record<string, unknown> } | { click: string };
export interface MountRow {
  id: string;
  file: string;
  component: string;
  props: Record<string, unknown>;
  steps: Step[];
  /** The issue that tracks a known difference; the row must still differ. */
  expected?: string;
}

function observe(parent: any): unknown[] {
  const children: unknown[] = [];
  for (const node of parent.childNodes) {
    if (node.nodeType === 3) {
      if (!node.data) continue;
      if (typeof children.at(-1) === "string") children[children.length - 1] += node.data;
      else children.push(node.data);
    } else if (node.nodeType === 1) {
      const attributes = Object.fromEntries(
        [...node.attributes]
          .map(({ name, value }: any) => [name, value])
          .sort(([a], [b]) => a.localeCompare(b)),
      );
      children.push({ tag: node.localName, attributes, children: observe(node) });
    }
  }
  return children;
}

/** Mount one exported component and record every phase. */
export async function trace(
  module: Record<string, unknown>,
  row: MountRow,
  runtime: "vapor" | "vdom",
) {
  const component = module[row.component];
  assert.ok(component, `${row.id}: export ${row.component}`);
  const state = Vue.reactive(structuredClone(row.props));
  const keys = Object.keys(row.props);
  const app =
    runtime === "vapor"
      ? Vue.createVaporApp(
          Vue.defineVaporComponent({
            setup: () =>
              Vue.createComponent(
                component,
                Object.fromEntries(keys.map((key) => [key, () => state[key]])),
              ),
          }),
        )
      : Vue.createApp({ render: () => Vue.h(component, { ...state }) });
  const warnings: string[] = [];
  app.config.warnHandler = (message: string) => warnings.push(message);
  app.config.errorHandler = (error: unknown) => warnings.push(String(error));
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const identities = new WeakMap<object, number>();
  let next = 0;
  const snapshot = () => {
    const elements = [...host.querySelectorAll("*")];
    for (const element of elements) if (!identities.has(element)) identities.set(element, next++);
    return {
      dom: observe(host),
      identities: elements.map((element) => identities.get(element)),
      warnings: warnings.splice(0),
    };
  };
  try {
    app.mount(host);
    await Vue.nextTick();
    const phases = [snapshot()];
    for (const step of row.steps) {
      if ("props" in step) Object.assign(state, structuredClone(step.props));
      else {
        const target = host.querySelector(step.click);
        assert.ok(target, `${row.id}: click target ${step.click}`);
        target.dispatchEvent(new window.Event("click", { bubbles: true, cancelable: true }));
      }
      await Vue.nextTick();
      phases.push(snapshot());
    }
    app.unmount();
    await Vue.nextTick();
    phases.push({ dom: observe(host), identities: [], warnings: warnings.splice(0) });
    return phases;
  } finally {
    host.remove();
  }
}

const runtimes = { vue: Vue, "vue-jsx-vapor": vaporRuntime, "/vue-jsx-vapor/props": propsRuntime };

export async function mountRow(row: MountRow, vizeCode: string) {
  const source = readFileSync(join(repoRoot, row.file), "utf8");
  // Both pipelines keep host TypeScript (the reference leaves it in functions it
  // does not wrap); erase it the way a bundler's TS stage would.
  const reference = evaluate(
    stripTypeScriptTypes(await referenceCode(source, row.file)),
    `${row.id}.reference.mjs`,
    runtimes,
  );
  const vize = evaluate(stripTypeScriptTypes(vizeCode), `${row.id}.vize.mjs`, { vue: Vue });
  return {
    id: row.id,
    reference: await trace(reference, row, "vapor"),
    vize: await trace(vize, row, "vdom"),
  };
}
