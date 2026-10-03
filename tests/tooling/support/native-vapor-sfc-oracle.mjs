// Dev-only independent whole-SFC control: stock plugin, export-helper and SSR runtime.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const requireUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const requireVue = createRequire(requireUi.resolve("vue-vapor-runtime/package.json"));
const officialCompiler = requireVue("vue/compiler-sfc");
const pluginFactory = (await import(pathToFileURL(requireUi.resolve("@vitejs/plugin-vue")).href))
  .default;
const { transformSync } = requireUi("@babel/core");
const { createSSRApp } = requireVue("vue");
const { renderToString } = requireVue("vue/server-renderer");

export const pluginVersion = requireUi("@vitejs/plugin-vue/package.json").version;
export const compilerVersion = officialCompiler.version;
assert.equal(pluginVersion, "6.0.7", "audited stock SFC plugin release");
assert.equal(compilerVersion, "3.6.0-rc.9", "audited Vapor compiler release");
assert.equal(requireVue("./package.json").version, compilerVersion);
assert.equal(requireVue("@vue/server-renderer/package.json").version, compilerVersion);

const helperId = "\0plugin-vue:export-helper";
const dataUrl = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
let sequence = 0;

// The native API explicitly chooses Vapor even for an unchanged ordinary <template>.
// Select that same target in the actual parse result; preserve all original source,
// locations, errors, and blocks. Compilation and SFC assembly remain stock code.
const compiler = {
  ...officialCompiler,
  parse(...args) {
    const parsed = officialCompiler.parse(...args);
    assert.equal(parsed.descriptor.source, args[0], "original SFC source remains exact");
    parsed.descriptor.vapor = true;
    return parsed;
  },
};

const context = {
  error(error) {
    throw new Error(typeof error === "string" ? error : error.message);
  },
  warn(warning) {
    throw new Error(typeof warning === "string" ? warning : warning.message);
  },
  addWatchFile() {},
};

async function transform(source, filename, ssr) {
  const plugin = pluginFactory({ compiler, template: { compilerOptions: { comments: true } } });
  plugin.configResolved({
    root: "/native-vapor",
    command: "build",
    isProduction: true,
    build: { sourcemap: true },
    define: {},
    logger: { warn: (warning) => context.warn(warning) },
  });
  plugin.buildStart.call(context);
  const result = await plugin.transform.handler.call(context, source, filename, { ssr });
  assert.equal(typeof result?.code, "string", "whole original SFC stock transform");
  const helperCode = plugin.load.handler.call(context, helperId, { ssr });
  assert.equal(typeof helperCode, "string", "actual stock export-helper source");
  return { code: result.code, map: result.map, helperCode };
}

async function serverComponent(code, helperCode, filename) {
  const imports = new Map([
    ["vue", pathToFileURL(requireVue.resolve("vue")).href],
    ["vue/server-renderer", pathToFileURL(requireVue.resolve("vue/server-renderer")).href],
    [helperId, dataUrl(helperCode)],
  ]);
  // Rewrite only real import declarations for Node loading. This cannot alter a
  // template literal containing words such as `from "vue"`. Returned code/map are
  // the untouched stock client transform, never this executable import resolution.
  const resolved = transformSync(code, {
    filename,
    configFile: false,
    babelrc: false,
    plugins: [
      () => ({
        visitor: {
          ImportDeclaration(path) {
            const original = path.node.source.value;
            assert.ok(imports.has(original), `unsupported stock SSR import: ${original}`);
            path.node.source.value = imports.get(original);
          },
        },
      }),
    ],
  });
  const module = await import(dataUrl(resolved.code + `\n// stock SFC SSR ${sequence++}`));
  assert.equal(module.default?.__vapor, true, "stock default component Vapor marker");
  assert.equal(typeof module.default.ssrRender, "function", "stock ssrRender attachment");
  assert.equal(typeof module.default.__multiRoot, "boolean", "stock root metadata");
  return module.default;
}

/**
 * Compile the complete unchanged source with audited stock plugin-vue and rc.9.
 * serverHtml is rendered by the stock SSR default component, including genuine
 * fragment delimiters. The optional caller configuration models real app props
 * and inheritAttrs, without reconstructing either client or server components.
 */
export async function compileVaporSfcReference(
  source,
  filename = "/native-vapor/NativeVapor.vue",
  { props = {}, inheritAttrs = true } = {},
) {
  const client = await transform(source, filename, false);
  const server = await transform(source, filename, true);
  assert.equal(server.helperCode, client.helperCode, "same actual export-helper in both modes");
  const component = await serverComponent(server.code, server.helperCode, filename);
  component.inheritAttrs = inheritAttrs;
  const app = createSSRApp(component, props);
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  const serverHtml = await renderToString(app);
  assert.deepEqual(diagnostics, [], "actual stock component SSR diagnostics");
  return {
    code: client.code,
    // Fixtures pin the complete actual serialized map, including empty arrays.
    map: JSON.parse(JSON.stringify(client.map)),
    helperCode: client.helperCode,
    multiRoot: component.__multiRoot,
    serverHtml,
  };
}
