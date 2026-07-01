export function rewriteReportedPaths(
  output: string,
  replacements: ReadonlyMap<string, string>,
): string {
  let rewritten = output;

  const orderedReplacements = buildReplacementVariants(replacements).sort(
    (left, right) => right[0].length - left[0].length,
  );

  for (const [from, to] of orderedReplacements) {
    rewritten = rewritten.split(from).join(to);
  }

  return rewritten;
}

function buildReplacementVariants(
  replacements: ReadonlyMap<string, string>,
): Array<[string, string]> {
  const variants = new Map<string, string>();

  for (const [from, to] of replacements) {
    registerReplacementVariant(variants, from, to);
    registerReplacementVariant(
      variants,
      escapeJsonStringSegment(from),
      escapeJsonStringSegment(to),
    );
  }

  return [...variants];
}

function registerReplacementVariant(variants: Map<string, string>, from: string, to: string): void {
  if (!from || !to) {
    return;
  }

  variants.set(from, to);
}

function escapeJsonStringSegment(value: string): string {
  return JSON.stringify(value).slice(1, -1);
}
