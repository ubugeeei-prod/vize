// Pinned runtime setup and module linkage shared by native and reference traces.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { compileFunction } from "node:vm";
import { pathToFileURL } from "node:url";
import {
  transformSync,
  type NodePath,
  type PluginObj,
  type types as BabelTypes,
} from "@babel/core";
import { Window } from "happy-dom";

const window = new Window();
for (const key of [
  "window",
  "document",
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
] as const) {
  (globalThis as unknown as Record<string, unknown>)[key] = key === "window" ? window : window[key];
}

const require = createRequire(import.meta.url);
// The development browser build prints a banner to stdout during module
// evaluation, which would corrupt the machine-readable receipt consumed by
// the Rust oracle. Production and reference output must share this exact
// runtime entry so stdout remains a single JSON document.
const vueVaporEntry = require.resolve("vue/dist/vue.runtime-with-vapor.esm-browser.prod.js");
const Vue = (await import(pathToFileURL(vueVaporEntry).href)) as typeof import("vue") & {
  setDynamicProps: (...args: unknown[]) => unknown;
};
const { transformVueJsxVapor } = await import("vue-jsx-vapor/api");

// Execute the published vue-jsx-vapor runtime source against the same Vue
// module as Vize output. Node's `vue` condition omits Vapor helpers, while the
// package is normally linked by a bundler to Vue's with-vapor entry.
const fromJsxVapor = createRequire(require.resolve("vue-jsx-vapor"));
const runtimeEntry = fromJsxVapor.resolve("@vue-jsx-vapor/runtime");
const runtimeSource = readFileSync(join(dirname(runtimeEntry), "vapor.js"), "utf8");
const linkedRuntimeResult = transformSync(runtimeSource, {
  configFile: false,
  babelrc: false,
  plugins: [
    ({ types: t }: { types: typeof import("@babel/core").types }): PluginObj => ({
      visitor: {
        ImportDeclaration(path: NodePath<BabelTypes.ImportDeclaration>) {
          assert.equal(path.node.source.value, "vue");
          const declarations = path.node.specifiers.map((specifier) => {
            if (specifier.type === "ImportNamespaceSpecifier") {
              return t.variableDeclaration("const", [
                t.variableDeclarator(specifier.local, t.identifier("__Vue")),
              ]);
            }
            assert.equal(specifier.type, "ImportSpecifier");
            return t.variableDeclaration("const", [
              t.variableDeclarator(
                specifier.local,
                t.memberExpression(t.identifier("__Vue"), specifier.imported),
              ),
            ]);
          });
          path.replaceWithMultiple(declarations);
        },
        ExportNamedDeclaration(path: NodePath<BabelTypes.ExportNamedDeclaration>) {
          if (path.node.declaration) path.replaceWith(path.node.declaration);
          else path.remove();
        },
      },
    }),
  ],
});
assert.ok(linkedRuntimeResult && typeof linkedRuntimeResult.code === "string");
const linkedRuntime = linkedRuntimeResult.code;
// Component and slot cohorts import helpers beyond `setNodes`, so expose the
// pinned runtime's complete public surface to the linked generated modules.
const JsxVapor = compileFunction(
  `${linkedRuntime}\nreturn {
    VaporFor,
    createComponent,
    createNodes,
    createProxyComponent,
    defineVaporComponent,
    defineVaporHmrComponent,
    defineVaporSSRComponent,
    isBlock,
    normalizeNode,
    normalizeVaporSlots,
    setNodes,
  };`,
  ["__Vue"],
  { filename: "vue-jsx-vapor.runtime.mjs" },
)(Vue);
const installed = {
  packageVersion: require("vue-jsx-vapor/package.json").version,
  vueVersion: require("vue/package.json").version,
};

const componentFixtures = {
  FixtureCard: Vue.defineComponent({
    props: {
      title: { type: String, required: true },
    },
    setup(props, { slots }) {
      return () => {
        const header = slots.header
          ? [Vue.h("header", { class: "header" }, slots.header({ suffix: "header" })), "|"]
          : [];
        return Vue.h("article", { "data-fixture": "card" }, [
          ...header,
          Vue.h("h2", null, ["Title:", props.title]),
          "|",
          Vue.h("div", { class: "slot" }, slots.default?.({ suffix: "slot" })),
        ]);
      };
    },
  }),
};

function linkedFunction(
  code: string,
  functionName: string,
  state: Record<string, unknown>,
  reads: string[],
  observations: { propPublications: number },
): unknown {
  const linkedResult = transformSync(code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }: { types: typeof import("@babel/core").types }): PluginObj => ({
        visitor: {
          ImportDeclaration(path: NodePath<BabelTypes.ImportDeclaration>) {
            const source = path.node.source.value;
            const runtime =
              source === "vue" ? "Vue" : source.includes("vue-jsx-vapor") ? "JsxVapor" : null;
            assert.notEqual(runtime, null, `unexpected runtime import: ${source}`);
            const properties = path.node.specifiers.map((specifier) => {
              assert.equal(specifier.type, "ImportSpecifier", source);
              return t.objectProperty(specifier.imported, specifier.local);
            });
            path.replaceWith(
              t.variableDeclaration("const", [
                t.variableDeclarator(t.objectPattern(properties), t.identifier(runtime!)),
              ]),
            );
          },
          ExportNamedDeclaration(path: NodePath<BabelTypes.ExportNamedDeclaration>) {
            assert.ok(path.node.declaration, "only declaration exports are executable fixtures");
            path.replaceWith(path.node.declaration);
          },
        },
      }),
    ],
  });
  assert.ok(linkedResult && typeof linkedResult.code === "string");
  const linked = linkedResult.code;
  return compileFunction(
    `${linked}\nreturn ${functionName};`,
    ["Vue", "JsxVapor", "state", "FixtureCard", "read"],
    { filename: `${functionName}.generated.mjs` },
  )(
    {
      ...Vue,
      setDynamicProps(...args: unknown[]) {
        observations.propPublications += 1;
        return Vue.setDynamicProps(...args);
      },
    },
    JsxVapor,
    state,
    componentFixtures.FixtureCard,
    (name: string) => {
      reads.push(name);
      return state[name];
    },
  );
}

export { Vue, window, installed, componentFixtures, linkedFunction, transformVueJsxVapor };
