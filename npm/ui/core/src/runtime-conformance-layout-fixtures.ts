import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";
import { aspectRatioRuntimeFixture } from "./runtime-conformance-aspect-ratio-fixtures.ts";
import { cardRuntimeFixture } from "./runtime-conformance-card-fixtures.ts";
import { clusterRuntimeFixture } from "./runtime-conformance-cluster-fixtures.ts";
import { containerRuntimeFixture } from "./runtime-conformance-container-fixtures.ts";
import { gridRuntimeFixture } from "./runtime-conformance-grid-fixtures.ts";
import { headingRuntimeFixture } from "./runtime-conformance-heading-fixtures.ts";
import { kbdRuntimeFixture } from "./runtime-conformance-kbd-fixtures.ts";
import { separatorRuntimeFixture } from "./runtime-conformance-separator-fixtures.ts";
import { skeletonRuntimeFixture } from "./runtime-conformance-skeleton-fixtures.ts";
import { spacerRuntimeFixture } from "./runtime-conformance-spacer-fixtures.ts";
import { stackRuntimeFixture } from "./runtime-conformance-stack-fixtures.ts";
import { textRuntimeFixture } from "./runtime-conformance-text-fixtures.ts";

export const layoutRuntimeFixtures: readonly RuntimeFixture[] = [
  aspectRatioRuntimeFixture,
  cardRuntimeFixture,
  clusterRuntimeFixture,
  containerRuntimeFixture,
  gridRuntimeFixture,
  headingRuntimeFixture,
  kbdRuntimeFixture,
  separatorRuntimeFixture,
  skeletonRuntimeFixture,
  spacerRuntimeFixture,
  stackRuntimeFixture,
  textRuntimeFixture,
];
