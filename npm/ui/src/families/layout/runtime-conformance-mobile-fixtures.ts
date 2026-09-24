import { safeAreaRuntimeFixtures } from "./safe-area/runtime-conformance-safe-area-fixtures.ts";
import { bottomNavigationRuntimeFixtures } from "../navigation/bottom-navigation/runtime-conformance-bottom-navigation-fixtures.ts";
import { actionSheetRuntimeFixtures } from "../overlays/action-sheet/runtime-conformance-action-sheet-fixtures.ts";
import { pullToRefreshRuntimeFixtures } from "../interaction/pull-to-refresh/runtime-conformance-pull-to-refresh-fixtures.ts";
import { swipeActionsRuntimeFixtures } from "../interaction/swipe-actions/runtime-conformance-swipe-actions-fixtures.ts";
import { pagerRuntimeFixtures } from "../navigation/pager/runtime-conformance-pager-fixtures.ts";
import type { RuntimeFixture } from "../../conformance/runtime-conformance-fixtures.ts";

/** SSR and hydration fixtures for mobile-first pattern SFCs. */
export const mobileRuntimeFixtures: readonly RuntimeFixture[] = [
  ...safeAreaRuntimeFixtures,
  ...bottomNavigationRuntimeFixtures,
  ...actionSheetRuntimeFixtures,
  ...pullToRefreshRuntimeFixtures,
  ...swipeActionsRuntimeFixtures,
  ...pagerRuntimeFixtures,
];
