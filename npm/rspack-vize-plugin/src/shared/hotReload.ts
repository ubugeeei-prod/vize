/** HMR code generation for Vue SFCs using `module.hot` (Rspack/webpack CJS API). */

/** Generate `module.hot` HMR boilerplate for a Vue SFC. */
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

/** Generate HMR code for CSS Module — updates binding and triggers rerender. */
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
