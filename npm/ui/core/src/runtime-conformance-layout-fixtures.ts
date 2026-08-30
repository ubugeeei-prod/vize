import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";
import { aspectRatioRuntimeFixture } from "./families/layout/aspect-ratio/runtime-conformance-aspect-ratio-fixtures.ts";
import { avatarRuntimeFixture } from "./families/layout/avatar/runtime-conformance-avatar-fixtures.ts";
import { cardRuntimeFixture } from "./families/layout/card/runtime-conformance-card-fixtures.ts";
import { clusterRuntimeFixture } from "./families/layout/cluster/runtime-conformance-cluster-fixtures.ts";
import { containerRuntimeFixture } from "./families/layout/container/runtime-conformance-container-fixtures.ts";
import { gridRuntimeFixture } from "./families/layout/grid/runtime-conformance-grid-fixtures.ts";
import { iconRuntimeFixtures } from "./families/layout/icon/runtime-conformance-icon-fixtures.ts";
import { listRuntimeFixture } from "./families/layout/list/runtime-conformance-list-fixtures.ts";
import { scrollAreaRuntimeFixture } from "./families/layout/scroll-area/runtime-conformance-scroll-area-fixtures.ts";
import { separatorRuntimeFixture } from "./families/layout/separator/runtime-conformance-separator-fixtures.ts";
import { skeletonRuntimeFixture } from "./families/feedback/skeleton/runtime-conformance-skeleton-fixtures.ts";
import { spacerRuntimeFixture } from "./families/layout/spacer/runtime-conformance-spacer-fixtures.ts";
import { stackRuntimeFixture } from "./families/layout/stack/runtime-conformance-stack-fixtures.ts";
import { surfaceRuntimeFixture } from "./families/layout/surface/runtime-conformance-surface-fixtures.ts";

export const layoutRuntimeFixtures: readonly RuntimeFixture[] = [
  aspectRatioRuntimeFixture,
  avatarRuntimeFixture,
  cardRuntimeFixture,
  clusterRuntimeFixture,
  containerRuntimeFixture,
  gridRuntimeFixture,
  ...iconRuntimeFixtures,
  listRuntimeFixture,
  scrollAreaRuntimeFixture,
  separatorRuntimeFixture,
  skeletonRuntimeFixture,
  spacerRuntimeFixture,
  stackRuntimeFixture,
  surfaceRuntimeFixture,
];
