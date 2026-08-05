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
 */
export function ssrModuleRegistrationCode(filePath: string, root: string): string {
  const moduleId = JSON.stringify(toManifestModuleId(filePath, root));
  return [
    `import { useSSRContext as __vize_useSSRContext } from "vue";`,
    `const __vize_sfc_setup = _sfc_main.setup;`,
    `_sfc_main.setup = (props, ctx) => {`,
    `  const ssrContext = __vize_useSSRContext();`,
    `  (ssrContext.modules || (ssrContext.modules = new Set())).add(${moduleId});`,
    `  return __vize_sfc_setup ? __vize_sfc_setup(props, ctx) : undefined;`,
    `};`,
  ].join("\n");
}

/**
 * The client-manifest key for `filePath`: relative to `root`, POSIX separators.
 *
 * A path outside the root keeps its absolute form rather than becoming a `../`
 * chain, matching how Vite keys modules it cannot root-relativize.
 */
export function toManifestModuleId(filePath: string, root: string): string {
  const relative = path.relative(root, filePath);
  if (!relative || relative.startsWith("..") || path.isAbsolute(relative)) {
    return filePath.replace(/\\/g, "/");
  }
  return relative.replace(/\\/g, "/");
}

/**
 * Append the registration to an emitted module.
 *
 * Call this last, so the wrapper closes over whatever `setup` the emitter's own
 * rewrites left in place. `useSSRContext()` throws outside a render, so this is
 * a no-op unless `isSsr`. Modules without an `_sfc_main` binding — a
 * render-function-only output, or a boundary placeholder — have no component
 * object to wrap and are returned untouched.
 */
export function appendSsrModuleRegistration(
  code: string,
  filePath: string,
  root: string,
  isSsr: boolean,
): string {
  if (!isSsr || !/\b_sfc_main\b/.test(code)) {
    return code;
  }
  if (code.includes("__vize_useSSRContext")) {
    return code;
  }
  return `${code}\n${ssrModuleRegistrationCode(filePath, root)}`;
}
