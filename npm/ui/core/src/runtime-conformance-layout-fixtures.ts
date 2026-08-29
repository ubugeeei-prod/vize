import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";
import { aspectRatioRuntimeFixture } from "./runtime-conformance-aspect-ratio-fixtures.ts";
import { separatorRuntimeFixture } from "./runtime-conformance-separator-fixtures.ts";
import { skeletonRuntimeFixture } from "./runtime-conformance-skeleton-fixtures.ts";
import { spacerRuntimeFixture } from "./runtime-conformance-spacer-fixtures.ts";
import { stackRuntimeFixture } from "./runtime-conformance-stack-fixtures.ts";

export const layoutRuntimeFixtures: readonly RuntimeFixture[] = [
  aspectRatioRuntimeFixture,
  separatorRuntimeFixture,
  skeletonRuntimeFixture,
  spacerRuntimeFixture,
  stackRuntimeFixture,
];
