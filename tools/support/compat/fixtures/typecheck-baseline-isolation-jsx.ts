import {
  ancestorPackagePath,
  packageNameFromExtendsSpecifier,
} from "./typecheck-baseline-isolation-package-extends.mjs";

/**
 * `compilerOptions.jsxImportSource` is a module walk out of the fixture (#4461).
 *
 * TypeScript loads `<name>/jsx-runtime` by climbing `node_modules`. Overlay
 * cannot retarget that require. Recording the package as an ancestor target
 * lets unique isolation link the fixture's own `vue` before Vize's copy.
 *
 * Child `jsxImportSource` replaces the parent, matching tsc. Package-name
 * `extends` configs are not read for it, matching `paths` and `types`.
 */

export function jsxImportSourcePackageName(specifier) {
  return packageNameFromExtendsSpecifier(specifier);
}

export function recordCompilerOptionJsxImportSource(declared, conflicts, fixtureRoot, chain) {
  if (!Array.isArray(chain)) return;
  let specifier;
  for (const entry of [...chain].reverse()) {
    const candidate =
      entry?.config?.compilerOptions?.jsxImportSource ?? entry?.compilerOptions?.jsxImportSource;
    if (typeof candidate === "string") specifier = candidate;
  }
  const name = packageNameFromExtendsSpecifier(specifier);
  if (name == null || conflicts.has(name) || declared.has(name)) return;
  const ancestor = ancestorPackagePath(fixtureRoot, name);
  if (ancestor == null) return;
  declared.set(name, ancestor);
}
