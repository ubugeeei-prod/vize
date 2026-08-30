import type { UiFamilyCatalogEntry } from "../../../family-catalog-types.ts";

import { interactionGestureFamilyCatalog } from "./gestures.ts";
import { interactionSupportFamilyCatalog } from "./support.ts";

export const interactionFamilyCatalog = [
  ...interactionGestureFamilyCatalog,
  ...interactionSupportFamilyCatalog,
] as const satisfies readonly UiFamilyCatalogEntry[];
