import { checkboxGroupRuntimeFixtures } from "../selection/checkbox-group/runtime-conformance-checkbox-group-fixtures.ts";
import { passwordFieldRuntimeFixtures } from "./password-field/runtime-conformance-password-field-fixtures.ts";
import { pinInputRuntimeFixtures } from "./pin-input/runtime-conformance-pin-input-fixtures.ts";
import { editableRuntimeFixtures } from "./editable/runtime-conformance-editable-fixtures.ts";
import type { RuntimeFixture } from "../../conformance/runtime-conformance-fixtures.ts";

/** SSR and hydration fixtures for every composite form-input SFC. */
export const formCompositeRuntimeFixtures: readonly RuntimeFixture[] = [
  ...checkboxGroupRuntimeFixtures,
  ...passwordFieldRuntimeFixtures,
  ...pinInputRuntimeFixtures,
  ...editableRuntimeFixtures,
];
