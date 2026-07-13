import {
  a as createFilter,
  i as generateOutput,
  r as compileVueModule,
  t as normalizeOptions,
} from "./unplugin-COmP-2kT.mjs";
//#region src/babel.ts
function vizeBabelPlugin(_api, rawOptions = {}) {
  const options = normalizeOptions(rawOptions);
  const filter = createFilter(options.include, options.exclude);
  const cache = /* @__PURE__ */ new Map();
  return {
    name: "babel-plugin-vize",
    manipulateOptions(_options, parserOptions) {
      ensureParserPlugin(parserOptions, "typescript");
      ensureParserPlugin(parserOptions, "jsx");
    },
    parserOverride(source, parserOptions, parse) {
      const filename = getFilename(parserOptions);
      if (!filename || !filename.endsWith(".vue") || !filter(filename)) return;
      const { compiled, warnings } = compileVueModule(filename, source, options, cache);
      for (const warning of warnings)
        process.emitWarning(`[vize] ${warning}`, { type: "VizeWarning" });
      return parse(
        generateOutput(compiled, {
          isProduction: options.isProduction,
          isDev: false,
          filePath: filename,
        }),
        {
          ...parserOptions,
          filename,
          sourceType: "module",
        },
      );
    },
  };
}
function getFilename(parserOptions) {
  return String(
    parserOptions.filename ?? parserOptions.sourceFilename ?? parserOptions.sourceFileName ?? "",
  );
}
function ensureParserPlugin(parserOptions, pluginName) {
  const plugins = parserOptions.plugins ?? [];
  if (!plugins.some((plugin) => parserPluginName(plugin) === pluginName)) plugins.push(pluginName);
  parserOptions.plugins = plugins;
}
function parserPluginName(plugin) {
  if (typeof plugin === "string") return plugin;
  if (Array.isArray(plugin) && typeof plugin[0] === "string") return plugin[0];
  return null;
}
//#endregion
export { vizeBabelPlugin as default };
