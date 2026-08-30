import assert from "node:assert/strict";

import { h } from "vue";

import Meter from "./meter.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

export const meterRuntimeFixture: RuntimeFixture = {
  name: "meter",
  sourceFile: "families/feedback/meter/meter.vue",
  render: () =>
    h(
      Meter,
      {
        ariaLabel: "Storage usage",
        high: 90,
        id: "storage-meter",
        low: 30,
        max: 100,
        optimum: 50,
        value: 64,
      },
      {
        default: () => "64%",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<meter/);
    assert.match(html, /id="storage-meter"/);
    assert.match(html, /value="64"/);
    assert.match(html, /max="100"/);
    assert.match(html, /low="30"/);
    assert.match(html, /high="90"/);
    assert.match(html, /optimum="50"/);
    assert.match(html, /aria-label="Storage usage"/);
    assert.match(html, /data-vize-ui="meter"/);
    assert.match(html, /data-state="optimum"/);
  },
  assertHydratedDom(host) {
    const meter = host.querySelector('[data-vize-ui="meter"]');
    assert.ok(meter instanceof HTMLMeterElement);
    assert.equal(meter.id, "storage-meter");
    assert.equal(meter.getAttribute("value"), "64");
    assert.equal(meter.getAttribute("max"), "100");
    assert.equal(meter.getAttribute("low"), "30");
    assert.equal(meter.getAttribute("high"), "90");
    assert.equal(meter.getAttribute("optimum"), "50");
    assert.equal(meter.getAttribute("data-state"), "optimum");
    assert.equal(meter.textContent, "64%");
  },
};
