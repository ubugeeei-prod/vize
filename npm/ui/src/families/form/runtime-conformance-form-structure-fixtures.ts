import { fieldsetRuntimeFixtures } from "./fieldset/runtime-conformance-fieldset-fixtures.ts";
import { formWizardRuntimeFixtures } from "./form-wizard/runtime-conformance-form-wizard-fixtures.ts";
import { rotaryRuntimeFixtures } from "./knob/runtime-conformance-knob-fixtures.ts";
import { phoneFieldRuntimeFixtures } from "./phone-field/runtime-conformance-phone-field-fixtures.ts";
import type { RuntimeFixture } from "../../conformance/runtime-conformance-fixtures.ts";

/** SSR and hydration fixtures for form structure and specialised input SFCs. */
export const formStructureRuntimeFixtures: readonly RuntimeFixture[] = [
  ...fieldsetRuntimeFixtures,
  ...formWizardRuntimeFixtures,
  ...rotaryRuntimeFixtures,
  ...phoneFieldRuntimeFixtures,
];
