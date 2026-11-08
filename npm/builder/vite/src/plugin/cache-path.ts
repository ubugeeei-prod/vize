/**
 * Stable cache identity for an SFC path.
 *
 * tinyglobby reports absolute paths with forward slashes on Windows while
 * Vite and node:path may report the same file with backslashes. Cache keys
 * must not depend on which producer supplied the path.
 */
export function normalizeCachePath(file: string): string {
  return file.replaceAll("\\", "/");
}
