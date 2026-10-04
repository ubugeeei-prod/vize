// Fresh whole stock transforms are observations; rc.9 client code is never a native golden.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const ui = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
export async function primary7502(source: string) {
  const vue = createRequire(ui.resolve("vue/package.json"));
  const compiler = vue("@vue/compiler-sfc");
  const vapor = createRequire(ui.resolve("vue-vapor-runtime/package.json"));
  const originalVaporCompiler = vapor("vue/compiler-sfc");
  assert.equal(compiler.version, "3.5.35");
  for (const name of ["@vue/compiler-dom", "@vue/compiler-ssr", "@vue/compiler-core"])
    assert.equal(vue(`${name}/package.json`).version, "3.5.35");
  assert.equal(originalVaporCompiler.version, "3.6.0-rc.9");
  for (const name of ["@vue/compiler-dom", "@vue/compiler-ssr", "@vue/compiler-core"])
    assert.equal(vapor(`${name}/package.json`).version, "3.6.0-rc.9");
  assert.equal(ui("@vitejs/plugin-vue/package.json").version, "6.0.7");
  const factory = (await import(pathToFileURL(ui.resolve("@vitejs/plugin-vue")).href)).default;
  const helperId = "\0plugin-vue:export-helper";
  const context = {
    error(value: any) {
      throw new Error(typeof value === "string" ? value : value.message);
    },
    warn(value: any) {
      throw new Error(typeof value === "string" ? value : value.message);
    },
    addWatchFile() {},
  };
  const vaporCompiler = {
    ...originalVaporCompiler,
    parse(...args: any[]) {
      const parsed = originalVaporCompiler.parse(...args);
      assert.equal(parsed.descriptor.source, args[0]);
      parsed.descriptor.vapor = true;
      return parsed;
    },
  };
  async function transform(ssr: boolean, selectedCompiler = compiler) {
    const plugin = factory({ compiler: selectedCompiler });
    plugin.configResolved({
      root: "/native-attribute-values-7502",
      command: "build",
      isProduction: true,
      build: { sourcemap: true },
      define: {},
      logger: { warn: context.warn },
    });
    plugin.buildStart.call(context);
    const output = await plugin.transform.handler.call(
      context,
      source,
      "/native-attribute-values-7502/AttributeValues7502.vue",
      { ssr },
    );
    assert.equal(typeof output?.code, "string");
    const helperCode = plugin.load.handler.call(context, helperId, { ssr });
    assert.equal(typeof helperCode, "string");
    return { code: output.code, map: JSON.parse(JSON.stringify(output.map)), helperCode };
  }
  const vaporClient = await transform(false, vaporCompiler);
  const vaporServer = await transform(true, vaporCompiler);
  assert.equal(vaporServer.helperCode, vaporClient.helperCode);
  return {
    versions: { domCompiler: compiler.version, plugin: "6.0.7", vaporCompiler: "3.6.0-rc.9" },
    dom: await transform(false),
    ssr: await transform(true),
    vapor: { ...vaporClient, serverCode: vaporServer.code, serverMap: vaporServer.map },
  };
}
