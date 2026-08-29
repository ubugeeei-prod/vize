import { basicFamilyCatalog } from "./family-catalog-basics.ts";
import { focusFamilyCatalog } from "./family-catalog-focus.ts";
import { foundationFamilyCatalog } from "./family-catalog-foundations.ts";
import { interactionFamilyCatalog } from "./family-catalog-interactions.ts";
import { overlayFamilyCatalog } from "./family-catalog-overlays.ts";
import type { UiFamilyCatalogEntry } from "./family-catalog-types.ts";

export {
  UI_FAMILY_CATALOG_SCHEMA_VERSION,
  type UiFamilyBundleBudget,
  type UiFamilyCatalogEntry,
  type UiFamilyMaturity,
  type UiFamilyQualityGate,
} from "./family-catalog-types.ts";

const allFamilyCatalogEntries = [
  ...basicFamilyCatalog,
  ...foundationFamilyCatalog,
  ...focusFamilyCatalog,
  ...interactionFamilyCatalog,
  ...overlayFamilyCatalog,
] as const satisfies readonly UiFamilyCatalogEntry[];

// Lane modules group families thematically, so canonical order is restored
// here rather than by concatenation order.
export const uiFamilyCatalog = [...allFamilyCatalogEntries].sort((a, b) =>
  a.canonicalName < b.canonicalName ? -1 : a.canonicalName > b.canonicalName ? 1 : 0,
);

export type UiFamilyCatalog = typeof uiFamilyCatalog;
export type UiFamilyName = UiFamilyCatalog[number]["canonicalName"];
