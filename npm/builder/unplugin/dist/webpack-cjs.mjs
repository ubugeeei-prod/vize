//#region src/webpack-cjs.ts
function isLegacyVueVersion(version) {
  return (
    version === "legacy" ||
    version === "2" ||
    version === "2.6" ||
    version === "2.7" ||
    version === 0.11 ||
    version === 1 ||
    version === 2
  );
}
function shouldUseHostCompiler(options) {
  const compatibility = options?.compatibility;
  return (
    compatibility?.hostCompiler ??
    (compatibility?.nuxtVersion === 2 ||
      isLegacyVueVersion(options?.vueVersion ?? compatibility?.vueVersion))
  );
}
function createUnsupportedCjsPlugin() {
  return {
    apply() {
      throw new Error(
        "[vize] @vizejs/unplugin/webpack was loaded through CommonJS, which is only supported for Nuxt 2/Vue 2 host-compiler configs. Use an ESM webpack config, or pass { vueVersion: 2 } / { compatibility: { hostCompiler: true } }.",
      );
    },
  };
}
function createHostCompilerPlugin() {
  return { apply() {} };
}
function vizeWebpackCjs(options) {
  return shouldUseHostCompiler(options) ? createHostCompilerPlugin() : createUnsupportedCjsPlugin();
}
//#endregion
export { vizeWebpackCjs as default, vizeWebpackCjs };
