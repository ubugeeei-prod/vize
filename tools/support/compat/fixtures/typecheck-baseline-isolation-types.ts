import {
  ancestorPackagePath,
  packageNameFromExtendsSpecifier,
} from "./typecheck-baseline-isolation-package-extends.ts";

/**
 * `compilerOptions.types` is another walk out of the fixture (#4461).
 *
 * TypeScript resolves each entry as a type-reference directive: it looks in
 * `typeRoots` (default `node_modules/@types`) and in `node_modules/<name>`,
 * climbing out of the fixture. Overlay cannot retarget that walk. Recording
 * the package — and `@types/<name>` for unscoped entries — as ancestor
 * targets lets unique isolation link the fixture's own copy first.
 *
 * Package-name `extends` configs are not read for `types`, matching `paths`.
 * `typeRoots` is a directory list, not a package name; outside `typeRoots`
 * belong to overlay rewrite, not unique-link.
 */

export function typePackageNamesFromTypes(types: unknown): string[] {
  const names: string[] = [];
  const seen = new Set<string>();
  if (!Array.isArray(types)) return names;
  for (const entry of types as readonly unknown[]) {
    const name = packageNameFromExtendsSpecifier(entry);
    if (name == null || seen.has(name)) continue;
    seen.add(name);
    names.push(name);
    if (name.startsWith("@")) continue;
    const ambient = `@types/${name}`;
    if (seen.has(ambient)) continue;
    seen.add(ambient);
    names.push(ambient);
  }
  return names;
}

export function recordCompilerOptionTypes(
  declared: Map<string, string>,
  conflicts: ReadonlySet<string>,
  fixtureRoot: string,
  types: unknown,
): void {
  for (const name of typePackageNamesFromTypes(types)) {
    if (conflicts.has(name) || declared.has(name)) continue;
    const ancestor = ancestorPackagePath(fixtureRoot, name);
    if (ancestor == null) continue;
    declared.set(name, ancestor);
  }
}
