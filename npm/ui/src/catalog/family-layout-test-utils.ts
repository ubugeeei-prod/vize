import assert from "node:assert/strict";

import { focusFamilyCatalog } from "./family-catalog-focus.ts";
import { foundationFamilyCatalog } from "./family-catalog-foundations.ts";
import { interactionFamilyCatalog } from "./family-catalog-interactions.ts";
import { overlayFamilyCatalog } from "./family-catalog-overlays.ts";
import type { UiFamilyCatalogEntry } from "./family-catalog-types.ts";

type FamilyCatalog = readonly UiFamilyCatalogEntry[];

type RehomedFlatFamily = {
  readonly catalog: FamilyCatalog;
  readonly familyName: string;
  readonly familyRoot: string;
};

export const rehomedFoundationUtilities = ["context", "controllable-state"] as const;

export const rehomedFlatFamilies: readonly RehomedFlatFamily[] = [
  {
    catalog: foundationFamilyCatalog,
    familyName: "collection",
    familyRoot: "src/families/foundations/collection/",
  },
  {
    catalog: foundationFamilyCatalog,
    familyName: "composite-navigation",
    familyRoot: "src/families/foundations/composite-navigation/",
  },
  {
    catalog: focusFamilyCatalog,
    familyName: "drag-and-drop",
    familyRoot: "src/families/interaction/drag-and-drop/",
  },
  {
    catalog: focusFamilyCatalog,
    familyName: "dismissable-layer",
    familyRoot: "src/families/overlays/dismissable-layer/",
  },
  {
    catalog: interactionFamilyCatalog,
    familyName: "primitive",
    familyRoot: "src/families/foundations/primitive/",
  },
  {
    catalog: interactionFamilyCatalog,
    familyName: "shortcut",
    familyRoot: "src/families/interaction/shortcut/",
  },
  {
    catalog: interactionFamilyCatalog,
    familyName: "sortable",
    familyRoot: "src/families/interaction/sortable/",
  },
  {
    catalog: interactionFamilyCatalog,
    familyName: "spatial-navigation",
    familyRoot: "src/families/interaction/spatial-navigation/",
  },
  {
    catalog: interactionFamilyCatalog,
    familyName: "virtualizer",
    familyRoot: "src/families/interaction/virtualizer/",
  },
  {
    catalog: overlayFamilyCatalog,
    familyName: "motion",
    familyRoot: "src/families/overlays/motion/",
  },
];

export function assertFamilyPaths(
  canonicalName: string,
  familyRoot: string,
  label: string,
  files: readonly string[],
): void {
  assert.ok(
    files.every((file) => file.startsWith(familyRoot)),
    `${canonicalName} ${label} must stay beside the family source`,
  );
}
