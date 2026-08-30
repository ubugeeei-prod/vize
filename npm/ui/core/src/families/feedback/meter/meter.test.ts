import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { getMeterState } from "./meter-state.ts";
import Meter from "./meter.vue";
import type { MeterExpose, MeterSlotState } from "./meter-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

test("normalizes finite ranges, thresholds, and optimum state", () => {
  assert.deepEqual(getMeterState({ value: 9, min: 0, max: 10, low: 3, high: 7, optimum: 8 }), {
    value: 9,
    min: 0,
    max: 10,
    low: 3,
    high: 7,
    optimum: 8,
    percent: 90,
    range: "high",
    optimal: true,
    invalid: false,
    state: "optimum",
  });
  assert.deepEqual(getMeterState({ value: 4, min: 0, max: 10, low: 3, high: 7, optimum: 8 }), {
    value: 4,
    min: 0,
    max: 10,
    low: 3,
    high: 7,
    optimum: 8,
    percent: 40,
    range: "medium",
    optimal: false,
    invalid: false,
    state: "medium",
  });
});

test("repairs unsafe native meter inputs before rendering", () => {
  assert.deepEqual(
    getMeterState({
      value: Number.NaN,
      min: 2,
      max: 2,
      low: 5,
      high: 1,
      optimum: Number.POSITIVE_INFINITY,
    }),
    {
      value: 2,
      min: 2,
      max: 3,
      low: 2,
      high: 3,
      optimum: null,
      percent: 0,
      range: "medium",
      optimal: false,
      invalid: true,
      state: "empty",
    },
  );
});

test("renders a labelled native meter with threshold semantics", () => {
  const handle = mountInteraction(Meter, {
    props: {
      id: "storage-meter",
      value: 64,
      min: 0,
      max: 100,
      low: 30,
      high: 90,
      optimum: 50,
      ariaLabel: "Storage usage",
      ariaDescribedby: "storage-help",
    },
    slots: {
      default: (state: MeterSlotState) => `${state.percent}% ${state.range}`,
    },
  });
  const meter = handle.root() as HTMLMeterElement;

  assert.ok(meter instanceof HTMLMeterElement);
  assert.equal(meter.id, "storage-meter");
  assert.equal(meter.getAttribute("aria-label"), "Storage usage");
  assert.equal(meter.getAttribute("value"), "64");
  assert.equal(meter.getAttribute("min"), "0");
  assert.equal(meter.getAttribute("max"), "100");
  assert.equal(meter.getAttribute("low"), "30");
  assert.equal(meter.getAttribute("high"), "90");
  assert.equal(meter.getAttribute("optimum"), "50");
  assert.equal(meter.getAttribute("aria-describedby"), "storage-help");
  assert.equal(meter.getAttribute("part"), "root");
  assert.equal(meter.getAttribute("data-vize-ui"), "meter");
  assert.equal(meter.getAttribute("data-state"), "optimum");
  assert.equal(meter.getAttribute("data-range"), "medium");
  assert.equal(meter.getAttribute("data-optimal"), "true");
  assert.equal(meter.getAttribute("data-invalid"), null);
  assert.equal(meter.getAttribute("data-value"), "64");
  assert.equal(meter.getAttribute("data-min"), "0");
  assert.equal(meter.getAttribute("data-max"), "100");
  assert.equal(meter.getAttribute("data-low"), "30");
  assert.equal(meter.getAttribute("data-high"), "90");
  assert.equal(meter.getAttribute("data-optimum"), "50");
  assert.equal(meter.getAttribute("data-percent"), "64");
  assert.equal(meter.textContent, "64% medium");
  handle.unmount();
});

test("clamps invalid props and updates slot plus exposed state", async () => {
  const handle = mountInteraction(Meter, {
    props: {
      ariaLabel: "Quota usage",
      value: -5,
      min: 0,
      max: 10,
      low: 8,
      high: 2,
    },
    slots: {
      default: (state: MeterSlotState) => `${state.state}:${state.invalid}:${state.percent}`,
    },
  });
  const exposed = handle.exposes<MeterExpose>();
  const meter = handle.root() as HTMLMeterElement;

  assert.equal(exposed.value, 0);
  assert.equal(exposed.low, 2);
  assert.equal(exposed.high, 8);
  assert.equal(exposed.invalid, true);
  assert.equal(exposed.state, "empty");
  assert.equal(meter.getAttribute("data-invalid"), "true");
  assert.equal(meter.textContent, "empty:true:0");

  await handle.wrapper.setProps({ value: 10, low: null, high: null, optimum: 10 });
  assert.equal(exposed.value, 10);
  assert.equal(exposed.percent, 100);
  assert.equal(exposed.optimal, true);
  assert.equal(exposed.state, "full");
  assert.equal(exposed.invalid, false);
  assert.equal(meter.getAttribute("low"), null);
  assert.equal(meter.getAttribute("high"), null);
  assert.equal(meter.getAttribute("optimum"), "10");
  assert.equal(meter.textContent, "full:false:100");
  handle.unmount();
});

test("does not enter the tab order or create a live region", async () => {
  const handle = mountInteraction(Meter, {
    props: { ariaLabel: "Memory usage", value: 0.5 },
  });
  const meter = handle.root();

  assert.equal(meter.getAttribute("tabindex"), null);
  assert.equal(meter.getAttribute("aria-live"), null);
  assert.equal(meter.getAttribute("role"), null);
  assert.ok((await handle.tab()) === null);
  handle.unmount();
});
