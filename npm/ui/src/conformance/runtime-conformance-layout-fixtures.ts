import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";
import { aspectRatioRuntimeFixture } from "../families/layout/aspect-ratio/runtime-conformance-aspect-ratio-fixtures.ts";
import { avatarRuntimeFixture } from "../families/layout/avatar/runtime-conformance-avatar-fixtures.ts";
import { cardRuntimeFixture } from "../families/layout/card/runtime-conformance-card-fixtures.ts";
import { clusterRuntimeFixture } from "../families/layout/cluster/runtime-conformance-cluster-fixtures.ts";
import { containerRuntimeFixture } from "../families/layout/container/runtime-conformance-container-fixtures.ts";
import { gridRuntimeFixture } from "../families/layout/grid/runtime-conformance-grid-fixtures.ts";
import { iconRuntimeFixtures } from "../families/layout/icon/runtime-conformance-icon-fixtures.ts";
import { dashboardGridRuntimeFixtures } from "../families/layout/dashboard-grid/runtime-conformance-dashboard-grid-fixtures.ts";
import { listRuntimeFixture } from "../families/layout/list/runtime-conformance-list-fixtures.ts";
import { masonryRuntimeFixture } from "../families/layout/masonry/runtime-conformance-masonry-fixtures.ts";
import { masterDetailRuntimeFixture } from "../families/layout/master-detail/runtime-conformance-master-detail-fixtures.ts";
import { responsiveRuntimeFixtures } from "../families/layout/responsive/runtime-conformance-responsive-fixtures.ts";
import { windowManagerRuntimeFixtures } from "../families/layout/window-manager/runtime-conformance-window-manager-fixtures.ts";
import { stickyStackRuntimeFixtures } from "../families/layout/sticky-stack/runtime-conformance-sticky-stack-fixtures.ts";
import { scrollAreaRuntimeFixture } from "../families/layout/scroll-area/runtime-conformance-scroll-area-fixtures.ts";
import { separatorRuntimeFixture } from "../families/layout/separator/runtime-conformance-separator-fixtures.ts";
import { skeletonRuntimeFixture } from "../families/feedback/skeleton/runtime-conformance-skeleton-fixtures.ts";
import { spacerRuntimeFixture } from "../families/layout/spacer/runtime-conformance-spacer-fixtures.ts";
import { stackRuntimeFixture } from "../families/layout/stack/runtime-conformance-stack-fixtures.ts";
import { surfaceRuntimeFixture } from "../families/layout/surface/runtime-conformance-surface-fixtures.ts";

export const layoutRuntimeFixtures: readonly RuntimeFixture[] = [
  aspectRatioRuntimeFixture,
  avatarRuntimeFixture,
  cardRuntimeFixture,
  clusterRuntimeFixture,
  containerRuntimeFixture,
  gridRuntimeFixture,
  ...iconRuntimeFixtures,
  ...dashboardGridRuntimeFixtures,
  listRuntimeFixture,
  masonryRuntimeFixture,
  masterDetailRuntimeFixture,
  ...responsiveRuntimeFixtures,
  ...stickyStackRuntimeFixtures,
  ...windowManagerRuntimeFixtures,
  scrollAreaRuntimeFixture,
  separatorRuntimeFixture,
  skeletonRuntimeFixture,
  spacerRuntimeFixture,
  stackRuntimeFixture,
  surfaceRuntimeFixture,
];
