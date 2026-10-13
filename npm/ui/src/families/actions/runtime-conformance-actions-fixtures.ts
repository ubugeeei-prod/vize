import type { RuntimeFixture } from "../../conformance/runtime-conformance-fixtures.ts";
import { backToTopRuntimeFixtures } from "./back-to-top/runtime-conformance-back-to-top-fixtures.ts";
import { buttonGroupRuntimeFixtures } from "./button-group/runtime-conformance-button-group-fixtures.ts";
import { copyButtonRuntimeFixture } from "./copy-button/runtime-conformance-copy-button-fixtures.ts";
import { floatingActionButtonRuntimeFixtures } from "./floating-action-button/runtime-conformance-floating-action-button-fixtures.ts";
import { fullscreenButtonRuntimeFixture } from "./fullscreen-button/runtime-conformance-fullscreen-button-fixtures.ts";
import { printButtonRuntimeFixture } from "./print-button/runtime-conformance-print-button-fixtures.ts";
import { shareButtonRuntimeFixture } from "./share-button/runtime-conformance-share-button-fixtures.ts";
import { splitButtonRuntimeFixtures } from "./split-button/runtime-conformance-split-button-fixtures.ts";
import { toolbarRuntimeFixtures } from "./toolbar/runtime-conformance-toolbar-fixtures.ts";

/** Runtime conformance fixtures owned by the actions family; new action components add theirs here. */
export const actionRuntimeFixtures: readonly RuntimeFixture[] = [
  ...backToTopRuntimeFixtures,
  ...buttonGroupRuntimeFixtures,
  copyButtonRuntimeFixture,
  ...floatingActionButtonRuntimeFixtures,
  fullscreenButtonRuntimeFixture,
  printButtonRuntimeFixture,
  shareButtonRuntimeFixture,
  ...splitButtonRuntimeFixtures,
  ...toolbarRuntimeFixtures,
];
