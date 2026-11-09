/**
 * Replace machine-specific absolute paths while preserving relative paths and
 * the surrounding diagnostic text. A path must contain at least two slashes,
 * matching the historical corpus fingerprint contract.
 */
export function normalizeMachinePaths(text: string): string {
  return text.replace(
    /(^|[^A-Za-z0-9/._+~@-])(\/[A-Za-z0-9/._+~@-]+)/gu,
    (_match, prefix: string, candidate: string) => {
      const trailingEllipsis = candidate.endsWith("...") ? "..." : "";
      const path = trailingEllipsis === "" ? candidate : candidate.slice(0, -3);
      const slashCount = path.split("/").length - 1;
      return slashCount >= 2 ? `${prefix}<abs>${trailingEllipsis}` : `${prefix}${candidate}`;
    },
  );
}
