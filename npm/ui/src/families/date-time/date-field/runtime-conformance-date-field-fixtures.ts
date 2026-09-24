import assert from "node:assert/strict";

import { h } from "vue";

import { createPlainDate } from "../calendar/plain-date.ts";
import DateField from "./date-field.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const dateFieldRuntimeFixture: RuntimeFixture = {
  name: "date-field",
  sourceFile: "families/date-time/date-field/date-field.vue",
  render: () =>
    h(DateField, {
      id: "runtime-date-field",
      locale: "en-US",
      name: "runtime-date",
      ariaLabel: "Date",
      defaultValue: createPlainDate(2026, 9, 25),
    }),
  assertServerMarkup(html) {
    assert.match(html, /role="group"/);
    assert.match(html, /role="spinbutton"/);
    assert.match(html, /value="2026-09-25"/);
  },
  assertHydratedDom(host) {
    const segments = host.querySelectorAll('[role="spinbutton"]');
    assert.equal(segments.length, 3);
    assert.equal(segments[0]?.getAttribute("aria-valuenow"), "9");
  },
};
