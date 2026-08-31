import { feedbackCoreCatalog } from "./core.ts";
import { feedbackLoadingCatalog } from "./loading.ts";
import { feedbackMessageCatalog } from "./messages.ts";

import type { UiFamilyCatalogEntry } from "../../../catalog/family-catalog-types.ts";

export const feedbackFamilyCatalog = [
  ...feedbackCoreCatalog,
  ...feedbackMessageCatalog,
  ...feedbackLoadingCatalog,
] as const satisfies readonly UiFamilyCatalogEntry[];
