import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

/**
 * Package-name `extends` is a second walk out of the fixture (#4461).
 *
 * Isolation follows only relative `extends` for `paths`, so TypeScript still
 * resolves `@vue/tsconfig` by climbing `node_modules` and can load Vize's copy.
 * Recording the ancestor package lets unique isolation link the fixture's own
 * pnpm copy first. The package config is not read for `paths` — that would
 * load Vize's files.
 */

const packageNamePattern = /^(?:@[a-z0-9][a-z0-9-._]*\/)?[a-z0-9][a-z0-9-._]*$/u;

export function packageNameFromExtendsSpecifier(specifier) {
  if (typeof specifier !== "string") return null;
  if (specifier.startsWith("./") || specifier.startsWith("../")) return null;
  if (specifier.startsWith("@")) {
    const slash = specifier.indexOf("/", 1);
    if (slash < 0) return null;
    const rest = specifier.slice(slash + 1);
    const second = rest.indexOf("/");
    const name = second < 0 ? specifier : `${specifier.slice(0, slash + 1)}${rest.slice(0, second)}`;
    return packageNamePattern.test(name) ? name : null;
  }
  const name = specifier.split("/")[0];
  return packageNamePattern.test(name) ? name : null;
}

export function ancestorPackagePath(fixtureRoot, name) {
  const root = resolve(fixtureRoot);
  const segments = name.split("/");
  let directory = dirname(root);
  let previous = null;
  while (directory !== previous) {
    const candidate = join(directory, "node_modules", ...segments);
    if (existsSync(join(candidate, "package.json"))) return candidate;
    previous = directory;
    directory = dirname(directory);
  }
  return null;
}
