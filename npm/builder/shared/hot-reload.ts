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
export function genHotReloadCode(
  id: string,
  metadata?: HmrMetadata,
  imports = "null",
  hot = "module.hot",
  component = "_sfc_main",
): string {
  return `
/* hot reload */
if (${hot}) {
  ${component}.__hmrId = "${id}"
  const api = __VUE_HMR_RUNTIME__
  const previous = ${hot}.data && ${hot}.data.vize
  // Own writable mappings; CSS module namespace exports may be read-only.
  if (${component}.__cssModules) {
    Object.keys(${component}.__cssModules).forEach(name => {
      ${component}.__cssModules[name] = Object.assign(Object.create(null), ${component}.__cssModules[name])
    })
  }
  const current = {
    metadata: ${JSON.stringify(metadata ?? null)},
    imports: (() => {
      // A circular import may still be uninitialized. An unavailable snapshot
      // disables state preservation without changing module loading semantics.
      try { return ${imports} } catch { return null }
    })(),
    component: ${component},
    setup: ${component}.setup && ${component}.setup.toString()
  }
  ${hot}.dispose(data => { data.vize = current })
  ${hot}.accept()
  if (!api.createRecord('${id}', ${component})) {
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
         before.template !== after.template && typeof ${component}.render === 'function'))) {
      // Retain both the module table and each mapping captured by setup.
      if (previous.component.__cssModules && ${component}.__cssModules) {
        Object.keys(${component}.__cssModules).forEach(name => ${genCSSModuleUpdateCode(
          "previous.component.__cssModules[name]",
          `${component}.__cssModules[name]`,
        )})
        ${component}.__cssModules = previous.component.__cssModules
      }
      if (before.module !== after.module || ${component}.__cssModules) {
        api.rerender('${id}', ${component}.render)
      }
    } else {
      api.reload('${id}', ${component})
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
  hot = "module.hot",
  component = "_sfc_main",
): string {
  return `
if (${hot}) {
  ${hot}.accept(${request}, () => {
    ${genCSSModuleUpdateCode(`${component}.__cssModules[${JSON.stringify(bindingName)}]`, varName)}
    __VUE_HMR_RUNTIME__.rerender("${id}", ${component}.render)
  })
}`;
}
