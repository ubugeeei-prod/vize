import assert from "node:assert/strict";

import { h } from "vue";

import DurationField from "./duration-field.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const durationFieldRuntimeFixture: RuntimeFixture = {
  name: "duration-field",
  sourceFile: "families/date-time/duration-field/duration-field.vue",
  render: () =>
    h(DurationField, {
      id: "runtime-duration-field",
      locale: "en-US",
      name: "runtime-duration",
      ariaLabel: "Duration",
      defaultValue: { hours: 2 },
    }),
  assertServerMarkup(html) {
    assert.match(html, /role="spinbutton"/);
    assert.match(html, /value="PT2H"/);
  },
  assertHydratedDom(host) {
    assert.equal(
      host.querySelector('[data-unit="hours"][role="spinbutton"]')?.getAttribute("aria-valuenow"),
      "2",
    );
  },
};
