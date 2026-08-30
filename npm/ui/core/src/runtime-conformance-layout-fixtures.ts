import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";
import { aspectRatioRuntimeFixture } from "./runtime-conformance-aspect-ratio-fixtures.ts";
import { avatarRuntimeFixture } from "./runtime-conformance-avatar-fixtures.ts";
import { blockquoteRuntimeFixture } from "./runtime-conformance-blockquote-fixtures.ts";
import { cardRuntimeFixture } from "./runtime-conformance-card-fixtures.ts";
import { clusterRuntimeFixture } from "./runtime-conformance-cluster-fixtures.ts";
import { codeRuntimeFixture } from "./runtime-conformance-code-fixtures.ts";
import { containerRuntimeFixture } from "./runtime-conformance-container-fixtures.ts";
import { gridRuntimeFixture } from "./runtime-conformance-grid-fixtures.ts";
import { headingRuntimeFixture } from "./runtime-conformance-heading-fixtures.ts";
import { iconRuntimeFixtures } from "./families/layout/icon/runtime-conformance-icon-fixtures.ts";
import { kbdRuntimeFixture } from "./runtime-conformance-kbd-fixtures.ts";
import { listRuntimeFixture } from "./runtime-conformance-list-fixtures.ts";
import { scrollAreaRuntimeFixture } from "./families/layout/scroll-area/runtime-conformance-scroll-area-fixtures.ts";
import { separatorRuntimeFixture } from "./runtime-conformance-separator-fixtures.ts";
import { skeletonRuntimeFixture } from "./runtime-conformance-skeleton-fixtures.ts";
import { spacerRuntimeFixture } from "./runtime-conformance-spacer-fixtures.ts";
import { stackRuntimeFixture } from "./runtime-conformance-stack-fixtures.ts";
import { surfaceRuntimeFixture } from "./families/layout/surface/runtime-conformance-surface-fixtures.ts";
import { textRuntimeFixture } from "./runtime-conformance-text-fixtures.ts";

export const layoutRuntimeFixtures: readonly RuntimeFixture[] = [
  aspectRatioRuntimeFixture,
  avatarRuntimeFixture,
  blockquoteRuntimeFixture,
  cardRuntimeFixture,
  clusterRuntimeFixture,
  codeRuntimeFixture,
  containerRuntimeFixture,
  gridRuntimeFixture,
  headingRuntimeFixture,
  ...iconRuntimeFixtures,
  kbdRuntimeFixture,
  listRuntimeFixture,
  scrollAreaRuntimeFixture,
  separatorRuntimeFixture,
  skeletonRuntimeFixture,
  spacerRuntimeFixture,
  stackRuntimeFixture,
  surfaceRuntimeFixture,
  textRuntimeFixture,
];
