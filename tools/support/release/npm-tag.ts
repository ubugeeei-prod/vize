/** Derives the npm release channel from a package version. */
export function npmTagForVersion(version: string): string {
  if (version.includes("-alpha")) return "alpha";
  if (version.includes("-beta")) return "beta";
  if (version.includes("-rc")) return "rc";
  return "latest";
}
