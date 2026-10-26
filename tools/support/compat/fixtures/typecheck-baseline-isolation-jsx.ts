import {
  ancestorPackagePath,
  packageNameFromExtendsSpecifier,
} from "./typecheck-baseline-isolation-package-extends.ts";

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

export function jsxImportSourcePackageName(specifier: unknown): string | null {
  return packageNameFromExtendsSpecifier(specifier);
}

/** A chain entry: `{ config, dir }` from the extends walk, or a bare parsed config. */
interface JsxChainEntry {
  readonly config?: { readonly compilerOptions?: { readonly jsxImportSource?: unknown } };
  readonly compilerOptions?: { readonly jsxImportSource?: unknown };
}

export function recordCompilerOptionJsxImportSource(
  declared: Map<string, string>,
  conflicts: ReadonlySet<string>,
  fixtureRoot: string,
  chain: unknown,
): void {
  if (!Array.isArray(chain)) return;
  let specifier: unknown;
  for (const entry of [...(chain as readonly (JsxChainEntry | null | undefined)[])].reverse()) {
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
