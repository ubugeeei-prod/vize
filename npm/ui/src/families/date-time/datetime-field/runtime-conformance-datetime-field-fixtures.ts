import assert from "node:assert/strict";

import { h } from "vue";

import DateTimeField from "./datetime-field.vue";
import { createPlainDateTime } from "./plain-date-time.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const dateTimeFieldRuntimeFixture: RuntimeFixture = {
  name: "datetime-field",
  sourceFile: "families/date-time/datetime-field/datetime-field.vue",
  render: () =>
    h(DateTimeField, {
      id: "runtime-datetime-field",
      locale: "en-US",
      name: "runtime-datetime",
      ariaLabel: "Start",
      defaultValue: createPlainDateTime(2026, 9, 25, 21, 5),
    }),
  assertServerMarkup(html) {
    assert.match(html, /role="group"/);
    assert.match(html, /value="2026-09-25T21:05"/);
  },
  assertHydratedDom(host) {
    assert.equal(host.querySelectorAll('[role="spinbutton"]').length, 6);
  },
};
