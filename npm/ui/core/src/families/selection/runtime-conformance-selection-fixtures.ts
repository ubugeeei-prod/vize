import type { RuntimeFixture } from "../../runtime-conformance-fixtures.ts";
import { checkboxRuntimeFixture } from "./checkbox/runtime-conformance-checkbox-fixtures.ts";
import { listboxRuntimeFixtures } from "./listbox/runtime-conformance-listbox-fixtures.ts";
import { nativeSelectRuntimeFixture } from "./native-select/runtime-conformance-native-select-fixtures.ts";
import { radioGroupRuntimeFixtures } from "./radio-group/runtime-conformance-radio-group-fixtures.ts";
import { switchRuntimeFixture } from "./switch/runtime-conformance-switch-fixtures.ts";
import { toggleGroupRuntimeFixtures } from "./toggle-group/runtime-conformance-toggle-group-fixtures.ts";
import { toggleRuntimeFixture } from "./toggle/runtime-conformance-toggle-fixtures.ts";

export const selectionRuntimeFixtures: readonly RuntimeFixture[] = [
  checkboxRuntimeFixture,
  ...listboxRuntimeFixtures,
  nativeSelectRuntimeFixture,
  ...radioGroupRuntimeFixtures,
  switchRuntimeFixture,
  toggleRuntimeFixture,
  ...toggleGroupRuntimeFixtures,
];
