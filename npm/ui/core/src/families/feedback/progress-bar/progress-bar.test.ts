import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import {
  PROGRESS_BAR_DEFAULT_MAX,
  PROGRESS_BAR_DEFAULT_MIN,
  getProgressBarState,
} from "./progress-bar.ts";
import ProgressBar from "./progress-bar.vue";
import type { ProgressBarExpose, ProgressBarSlotState } from "./progress-bar-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

test("normalizes determinate, complete, indeterminate, and invalid state", () => {
  assert.deepEqual(getProgressBarState({ value: 40, min: 20, max: 60, dir: "rtl" }), {
    value: 40,
    min: 20,
    max: 60,
    percent: 50,
    ratio: 0.5,
    dir: "rtl",
    indeterminate: false,
    complete: false,
    invalid: false,
    state: "loading",
  });
  assert.deepEqual(getProgressBarState({ value: 120, max: 100 }), {
    value: 100,
    min: PROGRESS_BAR_DEFAULT_MIN,
    max: 100,
    percent: 100,
    ratio: 1,
    dir: "ltr",
    indeterminate: false,
    complete: true,
    invalid: true,
    state: "complete",
  });
  assert.deepEqual(getProgressBarState({ value: Number.NaN, min: 10, max: 0 }), {
    value: null,
    min: 10,
    max: 110,
    percent: null,
    ratio: null,
    dir: "ltr",
    indeterminate: true,
    complete: false,
    invalid: true,
    state: "indeterminate",
  });
});

test("renders a named determinate progressbar with parts and CSS hooks", () => {
  const handle = mountInteraction(ProgressBar, {
    props: {
      id: "upload-progress",
      label: "Upload progress",
      value: 40,
      min: 20,
      max: 100,
      ariaDescribedby: "upload-help",
      ariaValueText: "40 of 100 files",
      valueLabel: "50%",
    },
    slots: {
      indicator: (state: ProgressBarSlotState) => `${state.percent}%`,
    },
  });
  const progress = handle.getByRole("progressbar", { name: "Upload progress" });
  const track = progress.querySelector('[data-vize-ui="progress-bar-track"]');
  const indicator = progress.querySelector('[data-vize-ui="progress-bar-indicator"]');

  assert.ok(progress instanceof HTMLElement);
  assert.equal(progress.id, "upload-progress");
  assert.equal(progress.getAttribute("aria-valuemin"), "20");
  assert.equal(progress.getAttribute("aria-valuemax"), "100");
  assert.equal(progress.getAttribute("aria-valuenow"), "40");
  assert.equal(progress.getAttribute("aria-describedby"), "upload-help");
  assert.equal(progress.getAttribute("aria-valuetext"), "40 of 100 files");
  assert.equal(progress.getAttribute("aria-live"), null);
  assert.equal(progress.getAttribute("part"), "root");
  assert.equal(progress.getAttribute("data-vize-ui"), "progress-bar");
  assert.equal(progress.getAttribute("data-state"), "loading");
  assert.equal(progress.getAttribute("data-indeterminate"), "false");
  assert.equal(progress.getAttribute("data-complete"), "false");
  assert.equal(progress.getAttribute("data-value"), "40");
  assert.equal(progress.getAttribute("data-min"), "20");
  assert.equal(progress.getAttribute("data-max"), "100");
  assert.equal(progress.getAttribute("data-percent"), "25");
  assert.equal(progress.style.getPropertyValue("--vize-ui-progress-bar-percent"), "25%");
  assert.ok(track instanceof HTMLSpanElement);
  assert.equal(track.getAttribute("part"), "track");
  assert.ok(indicator instanceof HTMLSpanElement);
  assert.equal(indicator.getAttribute("part"), "indicator");
  assert.equal(progress.textContent, "Upload progress25%50%");
  handle.unmount();
});

test("omits value semantics for indeterminate progress", () => {
  const handle = mountInteraction(ProgressBar, {
    props: { ariaLabel: "Import progress", valueLabel: "Waiting" },
    slots: {
      default: (state: ProgressBarSlotState) =>
        state.indeterminate ? "Waiting for server" : `${state.percent}%`,
    },
  });
  const progress = handle.getByRole("progressbar", { name: "Import progress" });

  assert.equal(progress.hasAttribute("aria-valuenow"), false);
  assert.equal(progress.getAttribute("aria-valuemin"), "0");
  assert.equal(progress.getAttribute("aria-valuemax"), "100");
  assert.equal(progress.getAttribute("aria-valuetext"), "Waiting");
  assert.equal(progress.getAttribute("data-state"), "indeterminate");
  assert.equal(progress.getAttribute("data-indeterminate"), "true");
  assert.equal(progress.getAttribute("data-complete"), "false");
  assert.equal(progress.getAttribute("data-value"), null);
  assert.equal(progress.getAttribute("data-percent"), null);
  assert.equal(progress.textContent, "WaitingWaiting for server");
  handle.unmount();
});

test("clamps ARIA attributes to the safe progress range", async () => {
  const handle = mountInteraction(ProgressBar, {
    props: {
      ariaLabel: "Sync progress",
      value: -10,
      max: 0,
    },
  });
  const progress = handle.getByRole("progressbar", { name: "Sync progress" });

  assert.equal(progress.getAttribute("aria-valuenow"), "0");
  assert.equal(progress.getAttribute("aria-valuemax"), String(PROGRESS_BAR_DEFAULT_MAX));
  assert.equal(progress.getAttribute("data-percent"), "0");
  assert.equal(progress.getAttribute("data-invalid"), "true");
  assert.equal(progress.getAttribute("data-state"), "loading");

  await handle.wrapper.setProps({ value: 150, max: 100 });
  assert.equal(progress.getAttribute("data-value"), "100");
  assert.equal(progress.getAttribute("data-percent"), "100");
  assert.equal(progress.getAttribute("data-state"), "complete");
  assert.equal(progress.getAttribute("data-complete"), "true");

  await handle.wrapper.setProps({ value: Number.POSITIVE_INFINITY });
  assert.equal(progress.hasAttribute("aria-valuenow"), false);
  assert.equal(progress.getAttribute("data-state"), "indeterminate");
  handle.unmount();
});

test("updates slot and exposed state from props", async () => {
  const handle = mountInteraction(ProgressBar, {
    props: {
      ariaLabel: "Build progress",
      value: 25,
      max: 50,
    },
    slots: {
      default: (state: ProgressBarSlotState) => `${state.state}:${state.percent}:${state.ratio}`,
    },
  });
  const exposed = handle.exposes<ProgressBarExpose>();

  assert.equal(exposed.value, 25);
  assert.equal(exposed.min, 0);
  assert.equal(exposed.max, 50);
  assert.equal(exposed.percent, 50);
  assert.equal(exposed.ratio, 0.5);
  assert.equal(exposed.dir, "ltr");
  assert.equal(exposed.indeterminate, false);
  assert.equal(exposed.complete, false);
  assert.equal(exposed.invalid, false);
  assert.equal(exposed.state, "loading");
  assert.ok(exposed.root === handle.root());
  assert.ok(exposed.track instanceof HTMLSpanElement);
  assert.ok(exposed.indicator instanceof HTMLSpanElement);
  assert.equal(exposed.style["--vize-ui-progress-bar-percent"], "50%");
  assert.equal(handle.root().textContent, "loading:50:0.5");

  await handle.wrapper.setProps({ value: 50 });
  assert.equal(exposed.value, 50);
  assert.equal(exposed.percent, 100);
  assert.equal(exposed.ratio, 1);
  assert.equal(exposed.complete, true);
  assert.equal(exposed.state, "complete");
  assert.equal(handle.root().textContent, "complete:100:1");

  await handle.wrapper.setProps({ value: null });
  assert.equal(exposed.value, null);
  assert.equal(exposed.percent, null);
  assert.equal(exposed.ratio, null);
  assert.equal(exposed.indeterminate, true);
  assert.equal(exposed.state, "indeterminate");
  assert.equal(handle.root().textContent, "indeterminate:null:null");
  handle.unmount();
});

test("uses visible label slots, RTL direction, and consumer host components", () => {
  const Host = {
    setup(
      _: unknown,
      {
        attrs,
        slots,
      }: {
        attrs: Record<string, unknown>;
        slots: {
          default?: () => unknown;
        };
      },
    ) {
      return () => h("section", attrs, slots.default?.());
    },
  };
  const handle = mountInteraction(ProgressBar, {
    props: {
      as: Host,
      dir: "rtl",
      value: 75,
    },
    slots: {
      label: (state: ProgressBarSlotState) => `Deploy ${state.dir}`,
      value: (state: ProgressBarSlotState) => `${state.percent}%`,
    },
  });
  const progress = handle.getByRole("progressbar", { name: "Deploy rtl" });
  const labelId = progress.getAttribute("aria-labelledby");

  assert.equal(progress.tagName, "SECTION");
  assert.equal(progress.getAttribute("dir"), "rtl");
  assert.equal(progress.getAttribute("data-dir"), "rtl");
  assert.ok(labelId);
  assert.equal(progress.querySelector(`#${labelId}`)?.getAttribute("part"), "label");
  assert.equal(progress.querySelector('[part="value"]')?.textContent, "75%");
  handle.unmount();
});

test("does not enter the tab order or create a live region by default", async () => {
  const handle = mountInteraction(ProgressBar, {
    props: { ariaLabel: "Background task", value: 5 },
  });
  const progress = handle.getByRole("progressbar", { name: "Background task" });

  assert.equal(progress.getAttribute("tabindex"), null);
  assert.equal(progress.getAttribute("aria-live"), null);
  assert.equal(progress.getAttribute("role"), "progressbar");
  assert.ok((await handle.tab()) === null);
  handle.unmount();
});
