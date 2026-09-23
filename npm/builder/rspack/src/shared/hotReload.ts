/** HMR code generation for Vue SFCs using `module.hot` (Rspack/webpack CJS API). */

export interface HmrMetadata {
  source: string;
  script?: string;
  template?: string;
  options: string;
  canRerender: boolean;
  module: string;
  styles: string;
}

/** Update an owned CSS mapping without invalidating useCssModule() references. */
function genCSSModuleUpdateCode(target: string, source: string): string {
  return `{
      const current = ${target}
      const next = ${source}
      Object.keys(current).forEach(key => {
        if (!Object.prototype.hasOwnProperty.call(next, key)) delete current[key]
      })
      Object.assign(current, next)
    }`;
}

/** Generate `module.hot` HMR boilerplate for a Vue SFC. */
export function genHotReloadCode(id: string, metadata?: HmrMetadata, imports = "null"): string {
  return `
/* hot reload */
if (module.hot) {
  _sfc_main.__hmrId = "${id}"
  const api = __VUE_HMR_RUNTIME__
  const previous = module.hot.data && module.hot.data.vize
  // Own writable mappings; CSS module namespace exports may be read-only.
  if (_sfc_main.__cssModules) {
    Object.keys(_sfc_main.__cssModules).forEach(name => {
      _sfc_main.__cssModules[name] = Object.assign(Object.create(null), _sfc_main.__cssModules[name])
    })
  }
  const current = {
    metadata: ${JSON.stringify(metadata ?? null)},
    imports: (() => {
      // A circular import may still be uninitialized. An unavailable snapshot
      // disables state preservation without changing module loading semantics.
      try { return ${imports} } catch { return null }
    })(),
    component: _sfc_main,
    setup: _sfc_main.setup && _sfc_main.setup.toString()
  }
  module.hot.dispose(data => { data.vize = current })
  module.hot.accept()
  if (!api.createRecord('${id}', _sfc_main)) {
    const before = previous && previous.metadata
    const after = current.metadata
    const sameImports = previous && previous.imports && current.imports &&
      previous.imports.length === current.imports.length &&
      current.imports.every((entry, index) => entry[0] === previous.imports[index][0] && Object.is(entry[1], previous.imports[index][1]))
    const canPreserve = before && after && before.canRerender && after.canRerender &&
      before.source !== after.source &&
      before.options === after.options && before.styles === after.styles && sameImports &&
      previous.setup === current.setup
    if (canPreserve && (before.module === after.module ||
        (typeof before.script === 'string' && before.script === after.script &&
         typeof before.template === 'string' && typeof after.template === 'string' &&
         before.template !== after.template && typeof _sfc_main.render === 'function'))) {
      // Retain both the module table and each mapping captured by setup.
      if (previous.component.__cssModules && _sfc_main.__cssModules) {
        Object.keys(_sfc_main.__cssModules).forEach(name => ${genCSSModuleUpdateCode(
          "previous.component.__cssModules[name]",
          "_sfc_main.__cssModules[name]",
        )})
        _sfc_main.__cssModules = previous.component.__cssModules
      }
      if (before.module !== after.module || _sfc_main.__cssModules) {
        api.rerender('${id}', _sfc_main.render)
      }
    } else {
      api.reload('${id}', _sfc_main)
    }
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
    ${genCSSModuleUpdateCode(`_sfc_main.__cssModules[${JSON.stringify(bindingName)}]`, varName)}
    __VUE_HMR_RUNTIME__.rerender("${id}", _sfc_main.render)
  })
}`;
}
