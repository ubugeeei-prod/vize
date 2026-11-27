import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { test } from "node:test";
import vm from "node:vm";
import { TraceMap, originalPositionFor } from "@jridgewell/trace-mapping";

// A clean packed/published consumer can run the exact same runtime assertions.
const consumer = process.env.VIZE_SWC_CONSUMER ?? fileURLToPath(new URL("../", import.meta.url));
const require = createRequire(resolve(consumer, "package.json"));
const swc = require("@swc/core") as typeof import("@swc/core");
const vue = require("vue") as typeof import("vue");
const { renderToString } = require("vue/server-renderer") as typeof import("vue/server-renderer");
const wasm = require.resolve("@vizejs/swc-plugin");
const sfc = require("@vizejs/swc-plugin/sfc") as typeof import("../src/sfc.ts");

function compile(source: string, options = {}, module: "commonjs" | "es6" = "commonjs") {
  return swc.transformSync(source, {
    filename: "Component.tsx",
    swcrc: false,
    configFile: false,
    sourceMaps: true,
    jsc: {
      parser: { syntax: "typescript", tsx: true },
      target: "es2022",
      experimental: { runPluginFirst: true, plugins: [[wasm, options]] },
    },
    module: { type: module },
  });
}

function evaluate(code: string, extra: Record<string, unknown> = {}) {
  const exports: Record<string, any> = {};
  const runtimeRequire = (id: string) =>
    id === "@vue/server-renderer" ? require("vue/server-renderer") : require(id);
  vm.runInNewContext(code, { exports, require: runtimeRequire, ...extra });
  return exports;
}

function origin(code: string, map: string, needle: string) {
  const offset = code.lastIndexOf(needle);
  assert.notEqual(offset, -1, needle);
  const prefix = code.slice(0, offset).split("\n");
  return originalPositionFor(new TraceMap(map), {
    line: prefix.length,
    column: prefix.at(-1)!.length,
  });
}

await test("Wasm plugin compiles typed JSX through SWC CommonJS and renders Vue", async () => {
  const result = compile(
    'export const App = (props: {label: string}) => <div class="card">{props.label}</div>',
  );
  assert.equal(
    await renderToString(vue.h(evaluate(result.code).App, { label: "Hello & Vue" })),
    '<div class="card">Hello &amp; Vue</div>',
  );
  assert.equal(swc.parseSync(result.code, { syntax: "ecmascript" }).type, "Module");
});

await test("fragments, scoped names, imports and nested functions survive the AST bridge", async () => {
  const source = `import { h } from 'vue';
const value = 'outer';
function inner(value: string) { return <span>{value}</span>; }
export const App = () => h('div', [inner('inner'), <><b>{value}</b><i>tail</i></>]);`;
  assert.equal(
    await renderToString(vue.h(evaluate(compile(source).code).App)),
    "<div><span>inner</span><!--[--><b>outer</b><i>tail</i><!--]--></div>",
  );
});

await test("SWC receives unchanged plain TypeScript when no JSX is present", () => {
  const code = "export const twice = (n: number): number => n * 2;";
  assert.equal(evaluate(compile(code).code).twice(21), 42);
  assert.equal(compile(code).code.includes('require("vue")'), false);
});

await test("the plugin compiles edits to the current AST and preserves authored comments", async () => {
  const ast = swc.parseSync("/*! license */\nexport const App = () => <div>before</div>; // tail", {
    syntax: "ecmascript",
    jsx: true,
  });
  const walk = (value: any) => {
    if (!value || typeof value !== "object") return;
    if (value.type === "JSXText") {
      value.value = "after";
      value.raw = "after";
    }
    for (const child of Object.values(value)) walk(child);
  };
  walk(ast);
  const result = swc.transformSync(ast, {
    filename: "Edited.jsx",
    swcrc: false,
    configFile: false,
    jsc: { target: "es2022", experimental: { runPluginFirst: true, plugins: [[wasm, {}]] } },
    module: { type: "commonjs" },
  });
  assert.equal(await renderToString(vue.h(evaluate(result.code).App)), "<div>after</div>");
  const comments = compile(
    "/*! license */\nexport const App = () => <div/>; // tail\nexport const end = 1;",
  ).code;
  assert.equal(comments.split("/*! license */").length, 2);
  assert.equal(comments.split("// tail").length, 2);
});

await test("Wasm source maps return to authored Unicode source after SWC output", () => {
  const source =
    'const 日本 = "🌸";\nexport const App = () => <div>{日本}</div>;\nexport const sentinel = 42;';
  const result = compile(source, {}, "es6");
  assert.deepEqual(JSON.parse(result.map!).sourcesContent, [source]);
  assert.deepEqual(origin(result.code, result.map!, "sentinel"), {
    source: "Component.tsx",
    line: 3,
    column: 13,
    name: "sentinel",
  });
  assert.equal(origin(result.code, result.map!, "日本").line, 2);
});

await test("SSR and Babel compatibility use the shared compiler", async () => {
  const server = compile(
    "export const App = (props: {label: string}) => { return <div>{props.label}</div>; }",
    { ssr: true },
  );
  assert.equal(
    await renderToString(vue.h(evaluate(server.code).App, { label: "server" })),
    "<div>server</div>",
  );
  const compat = compile('export const App = () => <div id="compat">ok</div>;', {
    jsxCompat: "babel",
  });
  assert.equal(await renderToString(vue.h(evaluate(compat.code).App)), '<div id="compat">ok</div>');
});

await test("Vapor routes through native codegen and leaves valid JavaScript", () => {
  const result = compile(
    "export const App = () => { return <div>Vapor</div>; }",
    { jsxMode: "vapor" },
    "es6",
  );
  const ast = swc.parseSync(result.code, { syntax: "ecmascript" });
  assert.equal(ast.type, "Module");
  assert.equal(result.code.includes("template"), true);
});

await test("invalid options, unsupported scoped JSX CSS and native diagnostics fail the host", () => {
  assert.throws(
    () => compile("const x = <div/>", { jsxMode: "react" }),
    /Invalid Vize SWC options/,
  );
  assert.throws(() => compile("const x = <div/>", { unknown: true }), /Invalid Vize SWC options/);
  assert.throws(
    () =>
      compile(
        "export const App = () => { return <div><style scoped>{`div { color: red }`}</style></div>; }",
      ),
    /Scoped JSX CSS/,
  );
  assert.throws(
    () => compile("const x = <div/>", { jsxMode: "vapor", jsxCompat: "babel" }),
    /babel|Vapor|compat/,
  );
});

await test("SFC ESM and CommonJS APIs both compile a scoped TypeScript component", async () => {
  const esm = (await import(pathToFileURL(resolve(dirname(wasm), "sfc.mjs")).href)) as typeof sfc;
  const source =
    '<script setup lang="ts">defineProps<{ label: string }>(); const className = "card";</script>\n<template><p :class="className">{{ label }}</p></template>\n<style scoped>.card { color: red }</style>';
  const options = {
    filename: "Card.vue",
    sourceMaps: true,
    module: { type: "commonjs" as const },
    vize: { styles: "metadata" as const },
  };
  const sync = sfc.transformVueSync(source, options);
  const async = await esm.transformVue(source, options);
  assert.deepEqual(async, sync);
  const App = evaluate(sync.code).default;
  assert.equal(App.__scopeId, sync.vize.scopeId);
  assert.equal(
    await renderToString(vue.h(App, { label: "SFC & SWC" })),
    `<p class="card" ${sync.vize.scopeId}>SFC &amp; SWC</p>`,
  );
  assert.equal(sync.vize.css.includes(sync.vize.scopeId), true);
  assert.deepEqual(JSON.parse(sync.map!).sourcesContent, [source]);
});

await test("template-only and Options API SFCs expose real default components", async () => {
  for (const source of [
    "<template><div>template only</div></template>",
    '<script>export default { data() { return { msg: "options" } } }</script><template><div>{{msg}}</div></template>',
  ]) {
    const result = sfc.transformVueSync(source, {
      filename: "App.vue",
      module: { type: "commonjs" },
    });
    assert.equal(
      await renderToString(vue.h(evaluate(result.code).default)),
      source.includes("template only") ? "<div>template only</div>" : "<div>options</div>",
    );
  }
});

await test("client CSS is inserted once, SSR returns CSS without touching document", async () => {
  const source = "<template><div>styles</div></template><style scoped>div { color: red }</style>";
  const options = { filename: "Styles.vue", module: { type: "commonjs" as const } };
  const client = sfc.transformVueSync(source, options);
  const elements: Record<string, any> = {};
  let appended = 0;
  const document = {
    getElementById: (id: string) => elements[id],
    createElement: () => ({}),
    head: {
      appendChild: (element: any) => {
        elements[element.id] = element;
        appended++;
      },
    },
  };
  evaluate(client.code, { document });
  evaluate(client.code, { document });
  assert.equal(appended, 1);
  assert.equal(Object.values(elements)[0].textContent, client.vize.css);
  const server = sfc.transformVueSync(source, { ...options, vize: { ssr: true } });
  const failDocument = new Proxy(
    {},
    {
      get() {
        throw new Error("SSR touched document");
      },
    },
  );
  assert.equal(
    await renderToString(vue.h(evaluate(server.code, { document: failDocument }).default)),
    `<div ${server.vize.scopeId}>styles</div>`,
  );
  assert.equal(server.vize.css, client.vize.css);
});

await test("SFC source map maps script identifiers to the .vue source", () => {
  const source =
    '<script setup lang="ts">\nconst greeting: string = "日本🌸";\n</script>\n<template>{{ greeting }}</template>';
  const result = sfc.transformVueSync(source, { filename: "Unicode.vue", sourceMaps: true });
  assert.equal(origin(result.code, result.map!, '"日本🌸"').source, "Unicode.vue");
  assert.equal(origin(result.code, result.map!, '"日本🌸"').line, 2);
});

await test("static SFC source maps retain the source without fabricated runtime origins", () => {
  const source = "<template><div/></template>";
  const result = sfc.transformVueSync(source, { filename: "Static.vue", sourceMaps: true });
  assert.deepEqual(JSON.parse(result.map!), {
    version: 3,
    sources: ["Static.vue"],
    sourcesContent: [source],
    names: [],
    mappings: "",
  });
});

await test("SFC unsupported asset pipelines and invalid configuration fail explicitly", () => {
  for (const source of [
    '<template><div/></template><style lang="scss">$a: red;</style>',
    "<template><div/></template><style module>.a{}</style>",
    "<template><div/></template><i18n>{}</i18n>",
    '<template src="./template.html"/>',
    '<script src="./script.ts"/>',
  ]) {
    assert.throws(() => sfc.transformVueSync(source, { filename: "App.vue" }));
  }
  assert.throws(
    () => sfc.transformVueSync("<template><div/></template>", { filename: "App.jsx" }),
    /\.vue/,
  );
  assert.throws(
    () => sfc.transformVueSync("<template><div/></template>", { filename: "App.vue", swcrc: true }),
    /configuration directly/,
  );
});
