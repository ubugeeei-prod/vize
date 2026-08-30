import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";
import { aspectRatioRuntimeFixture } from "./runtime-conformance-aspect-ratio-fixtures.ts";
import { avatarRuntimeFixture } from "./runtime-conformance-avatar-fixtures.ts";
import { cardRuntimeFixture } from "./runtime-conformance-card-fixtures.ts";
import { clusterRuntimeFixture } from "./runtime-conformance-cluster-fixtures.ts";
import { containerRuntimeFixture } from "./runtime-conformance-container-fixtures.ts";
import { gridRuntimeFixture } from "./runtime-conformance-grid-fixtures.ts";
import { iconRuntimeFixtures } from "./families/layout/icon/runtime-conformance-icon-fixtures.ts";
import { listRuntimeFixture } from "./runtime-conformance-list-fixtures.ts";
import { scrollAreaRuntimeFixture } from "./families/layout/scroll-area/runtime-conformance-scroll-area-fixtures.ts";
import { separatorRuntimeFixture } from "./runtime-conformance-separator-fixtures.ts";
import { skeletonRuntimeFixture } from "./runtime-conformance-skeleton-fixtures.ts";
import { spacerRuntimeFixture } from "./runtime-conformance-spacer-fixtures.ts";
import { stackRuntimeFixture } from "./runtime-conformance-stack-fixtures.ts";
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
