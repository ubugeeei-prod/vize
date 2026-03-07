/**
 * HMR (Hot Module Replacement) code generation for Vue SFC modules.
 *
 * Uses `module.hot` (Rspack/webpack CommonJS HMR API), NOT `import.meta.hot`.
 * Reference: rspack-vue-loader's hotReload.ts
 *
 * __VUE_HMR_RUNTIME__ is injected into global scope by @vue/runtime-core.
 */

/**
 * Generate HMR boilerplate code for a compiled Vue SFC module.
 *
 * The generated code:
 * 1. Assigns `__hmrId` to the component for Vue devtools
 * 2. Calls `module.hot.accept()` to make the module self-accepting
 * 3. Registers or reloads the component via `__VUE_HMR_RUNTIME__`
 *
 * @param id - Unique HMR identifier for this component (typically the scopeId hash)
 */
export function genHotReloadCode(id: string): string {
  return `
/* hot reload */
if (module.hot) {
  _sfc_main.__hmrId = "${id}"
  const api = __VUE_HMR_RUNTIME__
  module.hot.accept()
  if (!api.createRecord('${id}', _sfc_main)) {
    api.reload('${id}', _sfc_main)
  }
}`;
}

/**
 * Generate HMR code for a CSS Module import.
 *
 * When the CSS Module file changes, this code:
 * 1. Updates the __cssModules binding with the new values
 * 2. Triggers a Vue component rerender (NOT a full reload)
 */
export function genCSSModuleHotReloadCode(
  id: string,
  request: string,
  varName: string,
  bindingName: string,
): string {
  return `
if (module.hot) {
  module.hot.accept(${request}, () => {
    _sfc_main.__cssModules[${JSON.stringify(bindingName)}] = ${varName}
    __VUE_HMR_RUNTIME__.rerender("${id}")
  })
}`;
}
