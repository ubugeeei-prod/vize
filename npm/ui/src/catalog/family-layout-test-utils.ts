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

export const uiFamilyRoots = new Map<string, string>([
  ["alert", "src/families/feedback/alert/"],
  ["alert-dialog", "src/families/overlays/alert-dialog/"],
  ["announcer", "src/families/accessibility/announcer/"],
  ["aspect-ratio", "src/families/layout/aspect-ratio/"],
  ["avatar", "src/families/layout/avatar/"],
  ["banner", "src/families/feedback/banner/"],
  ["badge", "src/families/feedback/badge/"],
  ["breadcrumb", "src/families/navigation/breadcrumb/"],
  ["blockquote", "src/families/typography/blockquote/"],
  ["block-ui", "src/families/feedback/block-ui/"],
  ["button", "src/families/actions/button/"],
  ["button-group", "src/families/actions/button-group/"],
  ["calendar", "src/families/date-time/calendar/"],
  ["callout", "src/families/feedback/callout/"],
  ["card", "src/families/layout/card/"],
  ["carousel", "src/families/media/carousel/"],
  ["checkbox", "src/families/selection/checkbox/"],
  ["cluster", "src/families/layout/cluster/"],
  ["code", "src/families/typography/code/"],
  ["collection", "src/families/foundations/collection/"],
  ["color-picker", "src/families/form/color-picker/"],
  ["composite-navigation", "src/families/foundations/composite-navigation/"],
  ["container", "src/families/layout/container/"],
  ["context-menu", "src/families/menus/context-menu/"],
  ["dialog", "src/families/overlays/dialog/"],
  ["dismissable-layer", "src/families/overlays/dismissable-layer/"],
  ["drag-and-drop", "src/families/interaction/drag-and-drop/"],
  ["dropdown-menu", "src/families/menus/dropdown-menu/"],
  ["empty-state", "src/families/feedback/empty-state/"],
  ["error-summary", "src/families/form/error-summary/"],
  ["file-upload", "src/families/form/file-upload/"],
  ["focus", "src/families/accessibility/focus/"],
  ["focus-guards", "src/families/accessibility/focus-guards/"],
  ["focus-scope", "src/families/accessibility/focus-scope/"],
  ["fullscreen-button", "src/families/actions/fullscreen-button/"],
  ["grid", "src/families/layout/grid/"],
  ["heading", "src/families/typography/heading/"],
  ["hover", "src/families/interaction/hover/"],
  ["icon", "src/families/layout/icon/"],
  ["icon-button", "src/families/layout/icon/"],
  ["image", "src/families/media/image/"],
  ["inert-outside", "src/families/accessibility/inert-outside/"],
  ["interaction-modality", "src/families/accessibility/interaction-modality/"],
  ["infinite-scroll", "src/families/data/infinite-scroll/"],
  ["interaction-hooks", "src/families/interaction/interaction-hooks/"],
  ["kbd", "src/families/typography/kbd/"],
  ["link", "src/families/navigation/link/"],
  ["list", "src/families/layout/list/"],
  ["listbox", "src/families/selection/listbox/"],
  ["live-region", "src/families/accessibility/live-region/"],
  ["locale", "src/families/i18n/locale/"],
  ["long-press", "src/families/interaction/long-press/"],
  ["menu", "src/families/menus/menu/"],
  ["menubar", "src/families/menus/menubar/"],
  ["meter", "src/families/feedback/meter/"],
  ["motion", "src/families/overlays/motion/"],
  ["move", "src/families/interaction/move/"],
  ["native-select", "src/families/selection/native-select/"],
  ["pagination", "src/families/navigation/pagination/"],
  ["pointer-grace", "src/families/interaction/pointer-grace/"],
  ["popover", "src/families/overlays/popover/"],
  ["portal", "src/families/overlays/portal/"],
  ["positioner", "src/families/overlays/positioner/"],
  ["presence", "src/families/overlays/presence/"],
  ["press", "src/families/interaction/press/"],
  ["primitive", "src/families/foundations/primitive/"],
  ["print-button", "src/families/actions/print-button/"],
  ["progress", "src/families/feedback/progress/"],
  ["progress-bar", "src/families/feedback/progress-bar/"],
  ["qr-code", "src/families/media/qr-code/"],
  ["radio-group", "src/families/selection/radio-group/"],
  ["range-calendar", "src/families/date-time/range-calendar/"],
  ["rating", "src/families/form/rating/"],
  ["scroll-area", "src/families/layout/scroll-area/"],
  ["scroll-lock", "src/families/accessibility/scroll-lock/"],
  ["separator", "src/families/layout/separator/"],
  ["share-button", "src/families/actions/share-button/"],
  ["shortcut", "src/families/interaction/shortcut/"],
  ["skeleton", "src/families/feedback/skeleton/"],
  ["skip-link", "src/families/navigation/skip-link/"],
  ["sortable", "src/families/interaction/sortable/"],
  ["spacer", "src/families/layout/spacer/"],
  ["spatial-navigation", "src/families/interaction/spatial-navigation/"],
  ["spinner", "src/families/feedback/spinner/"],
  ["stack", "src/families/layout/stack/"],
  ["status-light", "src/families/feedback/status-light/"],
  ["stepper", "src/families/navigation/stepper/"],
  ["surface", "src/families/layout/surface/"],
  ["switch", "src/families/selection/switch/"],
  ["table", "src/families/data/table/"],
  ["tabs", "src/families/navigation/tabs/"],
  ["text", "src/families/typography/text/"],
  ["toggle", "src/families/selection/toggle/"],
  ["toggle-group", "src/families/selection/toggle-group/"],
  ["toolbar", "src/families/actions/toolbar/"],
  ["tooltip", "src/families/overlays/tooltip/"],
  ["tour", "src/families/overlays/tour/"],
  ["transition", "src/families/overlays/transition/"],
  ["virtualizer", "src/families/interaction/virtualizer/"],
  ["visually-hidden", "src/families/accessibility/visually-hidden/"],
]);

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
