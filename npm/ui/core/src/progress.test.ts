import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { PROGRESS_DEFAULT_MAX, getProgressState } from "./progress.ts";
import ProgressBar from "./progress-bar.vue";
import type { ProgressExpose, ProgressSlotState } from "./progress-types.ts";
import { mountInteraction } from "./testing/mount.ts";

test("normalizes determinate, complete, and indeterminate state", () => {
  assert.deepEqual(getProgressState({ value: 40, max: 100 }), {
    value: 40,
    max: 100,
    percent: 40,
    indeterminate: false,
    complete: false,
    state: "loading",
  });
  assert.deepEqual(getProgressState({ value: 120, max: 100 }), {
    value: 100,
    max: 100,
    percent: 100,
    indeterminate: false,
    complete: true,
    state: "complete",
  });
  assert.deepEqual(getProgressState({ value: Number.NaN, max: 0 }), {
    value: null,
    max: PROGRESS_DEFAULT_MAX,
    percent: null,
    indeterminate: true,
    complete: false,
    state: "indeterminate",
  });
});

test("renders a named native determinate progressbar", () => {
  const handle = mountInteraction(ProgressBar, {
    props: {
      id: "upload-progress",
      value: 40,
      max: 100,
      ariaLabel: "Upload progress",
      ariaDescribedby: "upload-help",
      ariaValueText: "40 of 100 files",
    },
    slots: {
      default: (state: ProgressSlotState) => `${state.percent}%`,
    },
  });
  const progress = handle.getByRole("progressbar", {
    name: "Upload progress",
  }) as HTMLProgressElement;

  assert.ok(progress instanceof HTMLProgressElement);
  assert.equal(progress.id, "upload-progress");
  assert.equal(progress.value, 40);
  assert.equal(progress.max, 100);
  assert.equal(progress.getAttribute("value"), "40");
  assert.equal(progress.getAttribute("max"), "100");
  assert.equal(progress.getAttribute("aria-describedby"), "upload-help");
  assert.equal(progress.getAttribute("aria-valuetext"), "40 of 100 files");
  assert.equal(progress.getAttribute("aria-live"), null);
  assert.equal(progress.getAttribute("part"), "root");
  assert.equal(progress.getAttribute("data-vize-ui"), "progress");
  assert.equal(progress.getAttribute("data-state"), "loading");
  assert.equal(progress.getAttribute("data-indeterminate"), "false");
  assert.equal(progress.getAttribute("data-complete"), "false");
  assert.equal(progress.getAttribute("data-value"), "40");
  assert.equal(progress.getAttribute("data-max"), "100");
  assert.equal(progress.getAttribute("data-percent"), "40");
  assert.equal(progress.textContent, "40%");
  handle.unmount();
});

test("omits the native value for indeterminate progress", () => {
  const handle = mountInteraction(ProgressBar, {
    props: { ariaLabel: "Import progress" },
    slots: {
      default: (state: ProgressSlotState) =>
        state.indeterminate ? "Waiting for server" : `${state.percent}%`,
    },
  });
  const progress = handle.getByRole("progressbar", {
    name: "Import progress",
  }) as HTMLProgressElement;

  assert.equal(progress.hasAttribute("value"), false);
  assert.equal(progress.getAttribute("max"), "100");
  assert.equal(progress.getAttribute("data-state"), "indeterminate");
  assert.equal(progress.getAttribute("data-indeterminate"), "true");
  assert.equal(progress.getAttribute("data-complete"), "false");
  assert.equal(progress.getAttribute("data-value"), null);
  assert.equal(progress.getAttribute("data-percent"), null);
  assert.equal(progress.textContent, "Waiting for server");
  handle.unmount();
});

test("clamps native attributes to the safe progress range", async () => {
  const handle = mountInteraction(ProgressBar, {
    props: {
      ariaLabel: "Sync progress",
      value: -10,
      max: 0,
    },
  });
  const progress = handle.getByRole("progressbar", {
    name: "Sync progress",
  }) as HTMLProgressElement;

  assert.equal(progress.value, 0);
  assert.equal(progress.max, PROGRESS_DEFAULT_MAX);
  assert.equal(progress.getAttribute("value"), "0");
  assert.equal(progress.getAttribute("max"), "100");
  assert.equal(progress.getAttribute("data-percent"), "0");
  assert.equal(progress.getAttribute("data-state"), "loading");

  await handle.wrapper.setProps({ value: 150, max: 100 });
  assert.equal(progress.value, 100);
  assert.equal(progress.getAttribute("data-value"), "100");
  assert.equal(progress.getAttribute("data-percent"), "100");
  assert.equal(progress.getAttribute("data-state"), "complete");
  assert.equal(progress.getAttribute("data-complete"), "true");

  await handle.wrapper.setProps({ value: Number.POSITIVE_INFINITY });
  assert.equal(progress.hasAttribute("value"), false);
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
      default: (state: ProgressSlotState) => `${state.state}:${state.percent}`,
    },
  });
  const exposed = handle.exposes<ProgressExpose>();

  assert.equal(exposed.value, 25);
  assert.equal(exposed.max, 50);
  assert.equal(exposed.percent, 50);
  assert.equal(exposed.indeterminate, false);
  assert.equal(exposed.complete, false);
  assert.equal(exposed.state, "loading");
  assert.ok(exposed.element === handle.root());
  assert.equal(handle.root().textContent, "loading:50");

  await handle.wrapper.setProps({ value: 50 });
  assert.equal(exposed.value, 50);
  assert.equal(exposed.percent, 100);
  assert.equal(exposed.complete, true);
  assert.equal(exposed.state, "complete");
  assert.equal(handle.root().textContent, "complete:100");

  await handle.wrapper.setProps({ value: null });
  assert.equal(exposed.value, null);
  assert.equal(exposed.percent, null);
  assert.equal(exposed.indeterminate, true);
  assert.equal(exposed.state, "indeterminate");
  assert.equal(handle.root().textContent, "indeterminate:null");
  handle.unmount();
});

test("does not enter the tab order or create a live region by default", async () => {
  const handle = mountInteraction(ProgressBar, {
    props: { ariaLabel: "Background task", value: 5 },
  });
  const progress = handle.getByRole("progressbar", { name: "Background task" });

  assert.equal(progress.getAttribute("tabindex"), null);
  assert.equal(progress.getAttribute("aria-live"), null);
  assert.equal(progress.getAttribute("role"), null);
  assert.ok((await handle.tab()) === null);
  handle.unmount();
});
