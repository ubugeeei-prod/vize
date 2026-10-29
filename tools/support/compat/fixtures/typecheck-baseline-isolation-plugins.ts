import {
  ancestorPackagePath,
  packageNameFromExtendsSpecifier,
} from "./typecheck-baseline-isolation-package-extends.ts";

/**
 * `compilerOptions.plugins` and `vueCompilerOptions.plugins` load by climbing
 * `node_modules` (#4461). Overlay cannot retarget that require. Recording the
 * plugin package as an ancestor target lets unique isolation link the
 * fixture's own copy first.
 *
 * Package-name `extends` configs are not read for plugins, matching `paths`
 * and `types`. Relative `extends` chains are unioned: TypeScript concatenates
 * plugin lists rather than replacing the parent object.
 */

/** The plugin lists of one parsed tsconfig; entries come from fixture files. */
interface PluginConfig {
  readonly compilerOptions?: { readonly plugins?: unknown };
  readonly vueCompilerOptions?: { readonly plugins?: unknown };
}

export function pluginPackageNamesFromConfigs(configs: unknown): string[] {
  const names: string[] = [];
  const seen = new Set<string>();
  if (!Array.isArray(configs)) return names;
  for (const config of configs as readonly (PluginConfig | null | undefined)[]) {
    addCompilerPlugins(names, seen, config?.compilerOptions?.plugins);
    addVueCompilerPlugins(names, seen, config?.vueCompilerOptions?.plugins);
  }
  return names;
}

export function recordCompilerOptionPlugins(
  declared: Map<string, string>,
  conflicts: ReadonlySet<string>,
  fixtureRoot: string,
  configs: unknown,
): void {
  for (const name of pluginPackageNamesFromConfigs(configs)) {
    if (conflicts.has(name) || declared.has(name)) continue;
    const ancestor = ancestorPackagePath(fixtureRoot, name);
    if (ancestor == null) continue;
    declared.set(name, ancestor);
  }
}

function addCompilerPlugins(names: string[], seen: Set<string>, plugins: unknown): void {
  if (!Array.isArray(plugins)) return;
  for (const plugin of plugins as readonly ({ readonly name?: unknown } | null)[]) {
    if (plugin == null || typeof plugin.name !== "string") continue;
    addName(names, seen, plugin.name);
  }
}

function addVueCompilerPlugins(names: string[], seen: Set<string>, plugins: unknown): void {
  if (!Array.isArray(plugins)) return;
  for (const plugin of plugins as readonly unknown[]) {
    if (typeof plugin === "string") {
      addName(names, seen, plugin);
      continue;
    }
    if (Array.isArray(plugin) && typeof plugin[0] === "string") {
      addName(names, seen, plugin[0]);
      continue;
    }
    if (isNamed(plugin)) {
      addName(names, seen, plugin.name);
    }
  }
}

function isNamed(value: unknown): value is { readonly name: string } {
  return (
    value != null && typeof value === "object" && "name" in value && typeof value.name === "string"
  );
}

function addName(names: string[], seen: Set<string>, specifier: string): void {
  const name = packageNameFromExtendsSpecifier(specifier);
  if (name == null || seen.has(name)) return;
  seen.add(name);
  names.push(name);
}
