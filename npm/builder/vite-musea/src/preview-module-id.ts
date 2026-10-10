/** Keep the final colon delimiter unambiguous while retaining ordinary module IDs. */
export function previewModuleId(artPath: string, variantName: string): string {
  const name = variantName.replaceAll("%", "%25").replaceAll(":", "%3A");
  return `virtual:musea-preview:${artPath}:${name}`;
}
