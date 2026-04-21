export function getVariantSectionId(variantName: string, fallbackIndex = 0): string {
  const slug = variantName
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");

  return `variant-${slug || fallbackIndex + 1}`;
}
