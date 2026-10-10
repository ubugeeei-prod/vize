/** Approval treats literal path/name characters literally and supports * and **. */
export function matchesSnapshotPattern(name: string, pattern: string): boolean {
  if (name.includes(pattern)) return true;
  const source = pattern
    .split("**")
    .map((part) => part.split("*").map(escapeRegex).join("[^/]*"))
    .join(".*");
  return new RegExp(`^${source}$`).test(name);
}

function escapeRegex(value: string): string {
  return value.replace(/[|\\{}()[\]^$+?.]/g, "\\$&");
}
