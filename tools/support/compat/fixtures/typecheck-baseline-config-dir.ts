import { resolve } from "node:path";

/**
 * TypeScript 5.5 substitutes `${configDir}` with the directory of the tsconfig
 * that declared the option, then resolves the result. Overlay rewrite used to
 * treat the token as a literal path segment, so Nuxt-style
 * `${configDir}/../node_modules/vue` never looked outside and never retargeted
 * (#4461).
 */

const token = "${configDir}";

export function expandConfigDir<T>(entry: T, tsconfigDir: string): T | string {
  if (typeof entry !== "string" || !entry.includes(token)) return entry;
  return entry.replaceAll(token, tsconfigDir.replaceAll("\\", "/"));
}

export function resolveWithConfigDir(
  mappingRoot: string,
  tsconfigDir: string,
  entry: string,
): string {
  return resolve(mappingRoot, expandConfigDir(entry, tsconfigDir));
}
