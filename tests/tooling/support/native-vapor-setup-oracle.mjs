// Independent original-SFC primary graph. No native assembly or hoist equality.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const compiler = fromVue("vue/compiler-sfc");
const pluginFactory = (await import(pathToFileURL(fromUi.resolve("@vitejs/plugin-vue")).href))
  .default;
const fromPlugin = createRequire(fromUi.resolve("@vitejs/plugin-vue/package.json"));
const { transformWithOxc } = await import(pathToFileURL(fromPlugin.resolve("vite")).href);
const { transformSync } = fromUi("@babel/core");
const vue = fromVue("vue");
const { renderToString } = fromVue("vue/server-renderer");

export const pluginVersion = fromUi("@vitejs/plugin-vue/package.json").version;
export const compilerVersion = compiler.version;
assert.equal(pluginVersion, "6.0.7");
assert.equal(compilerVersion, "3.6.0-rc.9");
assert.equal(vue.version, compilerVersion);
assert.equal(fromVue("@vue/server-renderer/package.json").version, compilerVersion);

const helperId = "\0plugin-vue:export-helper";
const dataUrl = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
const serialized = (value) => JSON.parse(JSON.stringify(value));
const context = {
  error(value) {
    throw new Error(typeof value === "string" ? value : value.message);
  },
  warn(value) {
    throw new Error(typeof value === "string" ? value : value.message);
  },
  addWatchFile() {},
};
let sequence = 0;

function declarations(code) {
  return transformSync(code, { configFile: false, babelrc: false, ast: true, code: false }).ast
    .program.body;
}

// Replace only parser-certified module-source literal ranges. Other original
// bytes, including comments and module-looking authored string literals, survive.
function resolveOwners(code, owners) {
  const edits = declarations(code)
    .filter((node) => node.source)
    .map((node) => {
      assert.ok(
        ["ImportDeclaration", "ExportNamedDeclaration", "ExportAllDeclaration"].includes(node.type),
      );
      assert.ok(owners.has(node.source.value), `unsupported stock owner ${node.source.value}`);
      return {
        start: node.source.start,
        end: node.source.end,
        text: JSON.stringify(owners.get(node.source.value)),
      };
    })
    .sort((left, right) => right.start - left.start);
  for (const edit of edits) code = code.slice(0, edit.start) + edit.text + code.slice(edit.end);
  return code;
}

async function compile(source, filename, ssr) {
  const graph = [];
  const selectedCompiler = {
    ...compiler,
    parse(...args) {
      const result = compiler.parse(...args);
      assert.equal(result.descriptor.source, source, "unchanged whole original SFC");
      result.descriptor.vapor = true;
      graph.push({
        api: "parse",
        originalSource: args[0],
        selectedVapor: true,
        scriptSetup: result.descriptor.scriptSetup?.loc ?? null,
        template: result.descriptor.template?.loc ?? null,
        rootElement:
          result.descriptor.template?.ast?.children.length === 1 &&
          result.descriptor.template.ast.children[0].type === 1 &&
          result.descriptor.template.ast.children[0].tagType === 0 &&
          result.descriptor.template.ast.children[0].ns === 0,
        errors: result.errors,
      });
      return result;
    },
    compileScript(descriptor, options) {
      assert.equal(options.inlineTemplate, true, "real stock production inline setup branch");
      const result = compiler.compileScript(descriptor, options);
      graph.push({
        api: "compileScript",
        inlineTemplate: options.inlineTemplate,
        isProd: options.isProd,
        ssr: options.templateOptions?.ssr ?? false,
        bindings: result.bindings,
        code: result.content,
        map: serialized(result.map),
        imports: result.imports,
      });
      return result;
    },
    compileTemplate(options) {
      const result = compiler.compileTemplate(options);
      graph.push({
        api: "compileTemplate",
        source: options.source,
        ssr: options.ssr,
        code: result.code,
        map: result.map,
        errors: result.errors,
      });
      return result;
    },
  };
  const plugin = pluginFactory({
    compiler: selectedCompiler,
    template: { compilerOptions: { comments: true } },
  });
  plugin.configResolved({
    root: "/native-vapor-setup",
    command: "build",
    isProduction: true,
    build: { sourcemap: true },
    define: {},
    logger: { warn: (value) => context.warn(value) },
  });
  plugin.buildStart.call(context);
  const result = await plugin.transform.handler.call(context, source, filename, { ssr });
  assert.equal(typeof result?.code, "string");
  const helperCode = plugin.load.handler.call(context, helperId, { ssr });
  assert.equal(typeof helperCode, "string", "actual stock export-helper");
  const virtualOwners = [
    ...new Set(
      declarations(result.code)
        .filter((node) => node.source?.value.includes("?vue&"))
        .map((node) => node.source.value),
    ),
  ];
  const virtualModules = [];
  for (const id of virtualOwners) {
    assert.equal(plugin.resolveId.handler.call(context, id), id);
    const loaded = plugin.load.handler.call(context, id, { ssr });
    assert.equal(typeof loaded?.code, "string", "actual virtual TS script module");
    const transformed = await transformWithOxc(
      loaded.code,
      id,
      { lang: "ts", sourcemap: true },
      loaded.map,
    );
    virtualModules.push({
      id,
      loaded: { code: loaded.code, map: serialized(loaded.map) },
      transformed: { code: transformed.code, map: serialized(transformed.map) },
      loader: "stock plugin-vue resolveId/load",
      transform: "actual Vite transformWithOxc(lang:ts,sourcemap:true)",
    });
  }
  return { code: result.code, map: serialized(result.map), helperCode, virtualModules, graph };
}

/** Render the unchanged primary SSR graph with genuine app configuration. */
export async function renderVaporSetupSfcReference(reference, configuration = {}) {
  assert.equal(reference.compilerVersion, compilerVersion);
  assert.equal(reference.pluginVersion, pluginVersion);
  assert.equal(reference.mode, "inline-production");
  const props = structuredClone(configuration.props ?? {});
  const inheritAttrs = configuration.inheritAttrs ?? true;
  assert.equal(typeof inheritAttrs, "boolean");
  const owners = new Map([
    ["vue", pathToFileURL(fromVue.resolve("vue")).href],
    ["vue/server-renderer", pathToFileURL(fromVue.resolve("vue/server-renderer")).href],
    [helperId, dataUrl(reference.ssr.helperCode)],
  ]);
  for (const virtual of reference.ssr.virtualModules)
    owners.set(
      virtual.id,
      dataUrl(
        resolveOwners(virtual.transformed.code, owners) + `\n// stock virtual SSR ${sequence++}`,
      ),
    );
  const component = (
    await import(
      dataUrl(resolveOwners(reference.ssr.code, owners) + `\n// stock whole SSR ${sequence++}`)
    )
  ).default;
  assert.equal(component.__vapor, true);
  assert.equal(component.__ssrInlineRender, true, "actual inline SSR setup branch");
  assert.equal(typeof component.setup, "function");
  assert.equal(component.render, undefined);
  assert.equal(component.ssrRender, undefined);
  component.inheritAttrs = inheritAttrs;
  const app = vue.createSSRApp(component, props);
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push({ kind: "warn", message });
  app.config.errorHandler = (error) =>
    diagnostics.push({ kind: "error", name: error.name, message: error.message });
  try {
    const serverHtml = await renderToString(app);
    return {
      serverHtml,
      serverDiagnostics: diagnostics,
      serverAuthority: "actual-stock-inline-SSR-default/createSSRApp/renderToString",
      propsBeforeNormalization: structuredClone(configuration.props ?? {}),
      propsAfterNormalization: props,
      inheritAttrs,
      defaultAuthority: {
        vapor: component.__vapor,
        inlineSSR: component.__ssrInlineRender,
        setup: typeof component.setup,
        render: typeof component.render,
        ssrRender: typeof component.ssrRender,
      },
    };
  } catch (error) {
    return {
      serverHtml: null,
      serverUnsupported: { name: error.name, message: error.message, stack: error.stack },
      serverAuthority: "actual-stock-inline-SSR-failure",
      inheritAttrs,
    };
  }
}

/** Complete stock CSR/SSR graphs and serialized maps; no native code equality. */
export async function compileVaporSetupSfcReference(
  source,
  id = "NativeVapor",
  configuration = {},
) {
  assert.equal(typeof source, "string");
  assert.match(id, /^[A-Za-z0-9_-]+$/);
  const filename = `/native-vapor-setup/${id}.vue`;
  const reference = {
    source,
    id,
    filename,
    mode: "inline-production",
    pluginVersion,
    compilerVersion,
    targetSelection: "explicit Vapor target on genuine parse result; original source unchanged",
    client: await compile(source, filename, false),
    ssr: await compile(source, filename, true),
  };
  reference.rootElement = reference.client.graph.find((step) => step.api === "parse").rootElement;
  const rendered = await renderVaporSetupSfcReference(reference, configuration);
  Object.assign(reference.ssr, rendered);
  return reference;
}
