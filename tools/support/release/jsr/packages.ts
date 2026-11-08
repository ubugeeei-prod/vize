export interface JsrPackageDescriptor {
  /** Checkout directory of the npm package. */
  readonly directory: string;
  /** JSR package name. Scoped npm packages keep the same public name. */
  readonly jsrName: `@vizejs/${string}`;
  /** Published npm package re-exported by the JSR source facade. */
  readonly npmName: string;
  /** Whether the npm root entry point has a default export. */
  readonly hasDefault?: boolean;
}

/**
 * JavaScript packages shipped by the release workflow and mirrored on JSR.
 * Keep this in the same order as RELEASE_JS_PACKAGES: it is a release contract,
 * not a best-effort registry selection.
 */
export const jsrPackages = [
  { directory: "npm/cli", jsrName: "@vizejs/vize", npmName: "vize" },
  {
    directory: "npm/create-fresco",
    jsrName: "@vizejs/create-fresco",
    npmName: "@vizejs/create-fresco",
  },
  {
    directory: "npm/builder/vite",
    jsrName: "@vizejs/vite-plugin",
    npmName: "@vizejs/vite-plugin",
    hasDefault: true,
  },
  {
    directory: "npm/linter/oxlint",
    jsrName: "@vizejs/oxlint-plugin-vize",
    npmName: "oxlint-plugin-vize",
    hasDefault: true,
  },
  {
    directory: "npm/builder/unplugin",
    jsrName: "@vizejs/unplugin",
    npmName: "@vizejs/unplugin",
    hasDefault: true,
  },
  { directory: "npm/fresco", jsrName: "@vizejs/fresco", npmName: "@vizejs/fresco" },
  {
    directory: "npm/mcp-musea",
    jsrName: "@vizejs/musea-mcp-server",
    npmName: "@vizejs/musea-mcp-server",
    hasDefault: true,
  },
  {
    directory: "npm/builder/vite-musea",
    jsrName: "@vizejs/vite-plugin-musea",
    npmName: "@vizejs/vite-plugin-musea",
    hasDefault: true,
  },
  {
    directory: "npm/builder/rspack",
    jsrName: "@vizejs/rspack-plugin",
    npmName: "@vizejs/rspack-plugin",
  },
  {
    directory: "npm/framework/musea-nuxt",
    jsrName: "@vizejs/musea-nuxt",
    npmName: "@vizejs/musea-nuxt",
  },
  {
    directory: "npm/framework/nuxt-lint-config",
    jsrName: "@vizejs/nuxt-lint-config",
    npmName: "@vizejs/nuxt-lint-config",
  },
  {
    directory: "npm/framework/nuxt",
    jsrName: "@vizejs/nuxt",
    npmName: "@vizejs/nuxt",
    hasDefault: true,
  },
  {
    directory: "npm/marquette",
    jsrName: "@vizejs/marquette",
    npmName: "@vizejs/marquette",
  },
  {
    directory: "npm/compose/core",
    jsrName: "@vizejs/composable",
    npmName: "@vizejs/composable",
  },
  {
    directory: "npm/compose/router",
    jsrName: "@vizejs/router",
    npmName: "@vizejs/router",
  },
  { directory: "npm/compose/data", jsrName: "@vizejs/data", npmName: "@vizejs/data" },
  { directory: "npm/ui", jsrName: "@vizejs/ui", npmName: "@vizejs/ui" },
  {
    directory: "npm/plugin-sdk",
    jsrName: "@vizejs/plugin-sdk",
    npmName: "@vizejs/plugin-sdk",
  },
  {
    directory: "npm/builder/babel",
    jsrName: "@vizejs/babel-plugin",
    npmName: "@vizejs/babel-plugin",
    hasDefault: true,
  },
] as const satisfies readonly JsrPackageDescriptor[];

export function jsrPackageDirectory(packageName: string): string {
  const [, name] = packageName.split("/", 2);
  if (!name || !/^[a-z0-9-]+$/u.test(name)) throw new Error(`Invalid JSR package ${packageName}`);
  return name;
}
