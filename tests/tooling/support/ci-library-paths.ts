/** Map the tools/ci layout back to the paths frozen workflow records name. */
export function restoreCiLibraryPaths(text: string): string {
  return text
    .replaceAll("../../../../tools/ci/lib/scripts/", "../shared/")
    .replaceAll("tools/ci/commands/", "tools/commands/ci/")
    .replaceAll("tools/ci/lib/scripts/", ".github/scripts/workflows/shared/");
}

/** Apply restoreCiLibraryPaths to every string inside a parsed workflow value. */
export function restoreCiLibraryValue<T>(value: T): T {
  return JSON.parse(restoreCiLibraryPaths(JSON.stringify(value))) as T;
}
