import { createRequire } from "node:module";

// semver ships no type declarations; this names the one function the smoke uses.
const semver = createRequire(import.meta.url)("semver") as {
  satisfies(version: string, range: string): boolean;
};

export function satisfiesVersionRange(version: string, range: string): boolean {
  return semver.satisfies(version, range);
}
