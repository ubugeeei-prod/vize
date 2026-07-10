const VUE_CLIENT_RUNTIME_IMPORT = "vue/dist/vue.runtime.esm-bundler.js";
const COMPONENT_BRIDGE_RE = /(?:_?resolveComponent\s*\(|from\s+(["'])#components\1)/;
const I18N_BRIDGE_RE = /\b(?:\$t|\$rt|\$d|\$n|\$tm|\$te)\s*\(/;
const STABLE_KEY_BRIDGE_RE = /\b(?:useFetch|useLazyFetch)\s*\(|\/\*\s*nuxt-injected\s*\*\//;

export function hasComponentBridgeInput(code: string): boolean {
  return COMPONENT_BRIDGE_RE.test(code);
}

export function hasI18nBridgeInput(code: string): boolean {
  return I18N_BRIDGE_RE.test(code);
}

export function hasStableKeyBridgeInput(code: string): boolean {
  return STABLE_KEY_BRIDGE_RE.test(code);
}

export function rewriteBareVueImportsToClientRuntime(code: string): string {
  return code
    .replace(/(\bfrom\s*)(["'])vue\2/g, (_, prefix: string, quote: string) => {
      return `${prefix}${quote}${VUE_CLIENT_RUNTIME_IMPORT}${quote}`;
    })
    .replace(/(\bimport\s*)(["'])vue\2/g, (_, prefix: string, quote: string) => {
      return `${prefix}${quote}${VUE_CLIENT_RUNTIME_IMPORT}${quote}`;
    });
}
