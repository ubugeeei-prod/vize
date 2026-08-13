import { basicFamilyCatalog } from "./family-catalog-basics.ts";
import { focusFamilyCatalog } from "./family-catalog-focus.ts";
import { interactionFamilyCatalog } from "./family-catalog-interactions.ts";
import type { UiFamilyCatalogEntry } from "./family-catalog-types.ts";

export {
  UI_FAMILY_CATALOG_SCHEMA_VERSION,
  type UiFamilyBundleBudget,
  type UiFamilyCatalogEntry,
  type UiFamilyMaturity,
  type UiFamilyQualityGate,
} from "./family-catalog-types.ts";

export const uiFamilyCatalog = [
  ...basicFamilyCatalog,
  ...focusFamilyCatalog,
  ...interactionFamilyCatalog,
] as const satisfies readonly UiFamilyCatalogEntry[];

export type UiFamilyCatalog = typeof uiFamilyCatalog;
export type UiFamilyName = UiFamilyCatalog[number]["canonicalName"];
