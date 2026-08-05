import path from "node:path";

/**
 * Point a compiled SSR module at the `vue/server-renderer` subpath.
 *
 * The compiler emits `@vue/server-renderer`, which is not a dependency every
 * app declares; the subpath re-export always resolves alongside `vue` itself.
 */
export function normalizeVueServerRendererImport(code: string): string {
  return code.replace(/\bfrom\s+(['"])@vue\/server-renderer\1/g, 'from "vue/server-renderer"');
}

/**
 * Marks a module whose registration has already been appended.
 *
 * Idempotency cannot key on the helper identifiers: an SFC is free to declare
 * `__vize_useSSRContext` itself, and skipping registration for it would cost it
 * its initial stylesheet.
 */
const REGISTRATION_MARKER = "/* @vize-ssr-modules-registered */";

/**
 * Matches the `_sfc_main` component declaration the emitted module carries.
 *
 * A bare `\b_sfc_main\b` also matches the identifier inside a string literal or
 * a comment, and wrapping a component that was never declared is a
 * `ReferenceError` at render time.
 */
const SFC_MAIN_DECLARATION = /\b(?:const|let|var)\s+_sfc_main\s*=/;

/**
 * Register an SFC in `ssrContext.modules` during SSR, the way
 * `@vitejs/plugin-vue` does.
 *
 * After rendering, `vue-bundle-renderer` intersects `ssrContext.modules` with
 * the client manifest to decide which stylesheets belong in the document head.
 * A component that never registers itself contributes no `<link>`, so the
 * server-rendered markup arrives unstyled and only gains its styles once the
 * route chunk loads on the client — a flash of unstyled content that never
 * recovers when JavaScript is disabled (#3868).
 *
 * The key must be the module's path relative to the Vite root with POSIX
 * separators, because that is what the client manifest is keyed on.
 *
 * `useSSRContext()` returns `undefined` when nothing provided a context — a
 * component rendered outside Nuxt's request render, such as by a prerender-time
 * module doing its own `renderToString`. Dereferencing that is a `TypeError`
 * inside `setup`, which surfaces as a bare `[500] Server Error` and fails the
 * whole prerender, so the registration is skipped instead. Such a render has no
 * manifest consumer waiting on `modules`, so there is nothing to lose.
 */
export function ssrModuleRegistrationCode(
  filePath: string,
  root: string,
  helpers: { useSSRContext?: string; sfcSetup?: string } = {},
): string {
  const moduleId = JSON.stringify(toManifestModuleId(filePath, root));
  const useSSRContext = helpers.useSSRContext ?? "__vize_useSSRContext";
  const sfcSetup = helpers.sfcSetup ?? "__vize_sfc_setup";
  return [
    REGISTRATION_MARKER,
    `import { useSSRContext as ${useSSRContext} } from "vue";`,
    `const ${sfcSetup} = _sfc_main.setup;`,
    `_sfc_main.setup = (props, ctx) => {`,
    `  const ssrContext = ${useSSRContext}();`,
    `  if (ssrContext) {`,
    `    (ssrContext.modules || (ssrContext.modules = new Set())).add(${moduleId});`,
    `  }`,
    `  return ${sfcSetup} ? ${sfcSetup}(props, ctx) : undefined;`,
    `};`,
  ].join("\n");
}

/**
 * The client-manifest key for `filePath`: relative to `root`, POSIX separators.
 *
 * A path outside the root keeps its absolute form rather than becoming a `../`
 * chain, matching how Vite keys modules it cannot root-relativize. Only a real
 * parent-directory step counts as outside: a filename that merely begins with
 * `..` still lives in the root and is keyed relative to it.
 */
export function toManifestModuleId(filePath: string, root: string): string {
  const relative = path.relative(root, filePath).replace(/\\/g, "/");
  if (!relative || relative === ".." || relative.startsWith("../") || path.isAbsolute(relative)) {
    return filePath.replace(/\\/g, "/");
  }
  return relative;
}

/**
 * Append the registration to an emitted module.
 *
 * Call this last, so the wrapper closes over whatever `setup` the emitter's own
 * rewrites left in place. `useSSRContext()` throws outside a render, so this is
 * a no-op unless `isSsr`. Modules without an `_sfc_main` declaration — a
 * render-function-only output, or a boundary placeholder — have no component
 * object to wrap and are returned untouched.
 */
export function appendSsrModuleRegistration(
  code: string,
  filePath: string,
  root: string,
  isSsr: boolean,
): string {
  if (!isSsr || !SFC_MAIN_DECLARATION.test(code)) {
    return code;
  }
  if (code.includes(REGISTRATION_MARKER)) {
    return code;
  }
  return `${code}\n${ssrModuleRegistrationCode(filePath, root, {
    useSSRContext: freeIdentifier("__vize_useSSRContext", code),
    sfcSetup: freeIdentifier("__vize_sfc_setup", code),
  })}`;
}

/**
 * `base`, or `base` plus the smallest numeric suffix the module does not use.
 *
 * Re-declaring an identifier the SFC already bound is a `SyntaxError`, which
 * would take down the whole module rather than just its stylesheet.
 */
function freeIdentifier(base: string, code: string): string {
  if (!new RegExp(`\\b${base}\\b`).test(code)) {
    return base;
  }
  for (let suffix = 2; ; suffix++) {
    const candidate = `${base}${suffix}`;
    if (!new RegExp(`\\b${candidate}\\b`).test(code)) {
      return candidate;
    }
  }
}
