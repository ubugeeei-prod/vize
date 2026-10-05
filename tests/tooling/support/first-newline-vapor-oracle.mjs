// Independent whole-SFC control, including script setup's genuine inline SSR.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const officialCompiler = fromVue("vue/compiler-sfc");
const pluginFactory = (await import(pathToFileURL(fromUi.resolve("@vitejs/plugin-vue")).href))
  .default;
const { transformSync } = fromUi("@babel/core");
const { createSSRApp } = fromVue("vue");
const { renderToString } = fromVue("vue/server-renderer");
assert.equal(fromUi("@vitejs/plugin-vue/package.json").version, "6.0.7");
assert.equal(officialCompiler.version, "3.6.0-rc.9");
assert.equal(fromVue("./package.json").version, officialCompiler.version);
assert.equal(fromVue("@vue/server-renderer/package.json").version, officialCompiler.version);

const helperId = "\0plugin-vue:export-helper";
const dataUrl = (code) => `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`;
let sequence = 0;
const compiler = {
  ...officialCompiler,
  parse(...args) {
    const parsed = officialCompiler.parse(...args);
    assert.deepEqual(parsed.errors, []);
    assert.equal(parsed.descriptor.source, args[0], "complete unchanged SFC");
    // Choose the same explicit Vapor target as the CLI, preserving every block.
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
  const plugin = pluginFactory({ compiler, template: { compilerOptions: { comments: false } } });
  plugin.configResolved({
    root: "/first-newline",
    command: "build",
    isProduction: true,
    build: { sourcemap: true },
    define: {},
    logger: { warn: (warning) => context.warn(warning) },
  });
  plugin.buildStart.call(context);
  const result = await plugin.transform.handler.call(context, source, filename, { ssr });
  assert.equal(typeof result?.code, "string");
  const helperCode = plugin.load.handler.call(context, helperId, { ssr });
  assert.equal(typeof helperCode, "string");
  return { code: result.code, helperCode };
}

export async function compileFirstNewlineVaporReference(source, filename) {
  const client = await transform(source, filename, false);
  const server = await transform(source, filename, true);
  assert.equal(client.helperCode, server.helperCode);
  const imports = new Map([
    ["vue", pathToFileURL(fromVue.resolve("vue")).href],
    ["vue/server-renderer", pathToFileURL(fromVue.resolve("vue/server-renderer")).href],
    [helperId, dataUrl(server.helperCode)],
  ]);
  const resolved = transformSync(server.code, {
    configFile: false,
    babelrc: false,
    plugins: [
      () => ({
        visitor: {
          ImportDeclaration(path) {
            assert(imports.has(path.node.source.value), path.node.source.value);
            path.node.source.value = imports.get(path.node.source.value);
          },
        },
      }),
    ],
  });
  const component = (await import(dataUrl(resolved.code + `\n// ${sequence++}`))).default;
  assert.equal(component.__vapor, true, "actual stock default Vapor component");
  assert.equal(component.__ssrInlineRender, true, "script setup's genuine inline SSR form");
  assert.equal(typeof component.setup, "function");
  const app = createSSRApp(component);
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  const serverHtml = await renderToString(app);
  assert.deepEqual(diagnostics, []);
  return { ...client, serverCode: server.code, serverHtml };
}
