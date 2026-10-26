import {
  ancestorPackagePath,
  packageNameFromExtendsSpecifier,
} from "./typecheck-baseline-isolation-package-extends.mjs";

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

export function pluginPackageNamesFromConfigs(configs) {
  const names = [];
  const seen = new Set();
  if (!Array.isArray(configs)) return names;
  for (const config of configs) {
    addCompilerPlugins(names, seen, config?.compilerOptions?.plugins);
    addVueCompilerPlugins(names, seen, config?.vueCompilerOptions?.plugins);
  }
  return names;
}

export function recordCompilerOptionPlugins(declared, conflicts, fixtureRoot, configs) {
  for (const name of pluginPackageNamesFromConfigs(configs)) {
    if (conflicts.has(name) || declared.has(name)) continue;
    const ancestor = ancestorPackagePath(fixtureRoot, name);
    if (ancestor == null) continue;
    declared.set(name, ancestor);
  }
}

function addCompilerPlugins(names, seen, plugins) {
  if (!Array.isArray(plugins)) return;
  for (const plugin of plugins) {
    if (plugin == null || typeof plugin.name !== "string") continue;
    addName(names, seen, plugin.name);
  }
}

function addVueCompilerPlugins(names, seen, plugins) {
  if (!Array.isArray(plugins)) return;
  for (const plugin of plugins) {
    if (typeof plugin === "string") {
      addName(names, seen, plugin);
      continue;
    }
    if (Array.isArray(plugin) && typeof plugin[0] === "string") {
      addName(names, seen, plugin[0]);
      continue;
    }
    if (plugin != null && typeof plugin.name === "string") {
      addName(names, seen, plugin.name);
    }
  }
}

function addName(names, seen, specifier) {
  const name = packageNameFromExtendsSpecifier(specifier);
  if (name == null || seen.has(name)) return;
  seen.add(name);
  names.push(name);
}
