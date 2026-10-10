import type { RuleMetadata } from "./types.ts";
import { generateVueCategoryPages } from "./vue-category-render.ts";
import { generateRemainingCatalogues } from "./catalogue-render.ts";

/** Compose every existing reader route after its authoritative references are generated. */
export function generateCategoryPages(
  root: string,
  rules: readonly RuleMetadata[],
  checking: boolean,
) {
  generateVueCategoryPages(root, rules, checking);
  generateRemainingCatalogues(root, rules, checking);
}
