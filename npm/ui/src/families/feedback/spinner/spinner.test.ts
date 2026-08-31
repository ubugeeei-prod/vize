import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import Spinner from "./spinner.vue";
import type { SpinnerExpose, SpinnerSlotState } from "./spinner.ts";
import { mountInteraction } from "../../../testing/mount.ts";

test("renders a polite status spinner by default", async () => {
  const handle = mountInteraction(Spinner, {
    props: {
      ariaDescribedby: "sync-help",
      ariaLabel: "Syncing profile",
    },
    slots: {
      default: (state: SpinnerSlotState) => `${state.state}:${state.ariaState}`,
    },
  });
  const status = handle.getByRole("status", { name: "Syncing profile" });

  assert.equal(status.tagName, "SPAN");
  assert.match(status.id, /^vize-v-\d+-spinner$/);
  assert.equal(status.getAttribute("aria-live"), "polite");
  assert.equal(status.getAttribute("aria-atomic"), "true");
  assert.equal(status.getAttribute("aria-describedby"), "sync-help");
  assert.equal(status.getAttribute("data-vize-ui"), "spinner");
  assert.equal(status.getAttribute("part"), "root");
  assert.equal(status.getAttribute("data-state"), "loading");
  assert.equal(status.getAttribute("data-loading"), "true");
  assert.equal(status.getAttribute("data-visible"), "true");
  assert.equal(status.getAttribute("data-aria-state"), "status");
  assert.equal(status.getAttribute("data-progress-state"), "none");
  assert.equal(status.getAttribute("data-complete"), "false");
  assert.equal(status.getAttribute("aria-valuenow"), null);
  assert.equal(status.textContent, "loading:status");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("renders progressbar semantics with normalized determinate values", async () => {
  const handle = mountInteraction(Spinner, {
    props: {
      ariaLabel: "Upload progress",
      ariaValueText: "25 of 50 chunks",
      max: 50,
      role: "progressbar",
      value: 25,
    },
  });
  const spinner = handle.getByRole("progressbar", { name: "Upload progress" });

  assert.equal(spinner.getAttribute("aria-live"), null);
  assert.equal(spinner.getAttribute("aria-valuemin"), "0");
  assert.equal(spinner.getAttribute("aria-valuemax"), "50");
  assert.equal(spinner.getAttribute("aria-valuenow"), "25");
  assert.equal(spinner.getAttribute("aria-valuetext"), "25 of 50 chunks");
  assert.equal(spinner.getAttribute("data-progress-state"), "determinate");
  assert.equal(spinner.getAttribute("data-value"), "25");
  assert.equal(spinner.getAttribute("data-percent"), "50");
  assert.equal(spinner.getAttribute("data-state"), "loading");

  await handle.wrapper.setProps({ value: 75 });
  assert.equal(spinner.getAttribute("aria-valuenow"), "50");
  assert.equal(spinner.getAttribute("data-value"), "50");
  assert.equal(spinner.getAttribute("data-percent"), "100");
  assert.equal(spinner.getAttribute("data-state"), "complete");
  assert.equal(spinner.getAttribute("data-complete"), "true");

  await handle.wrapper.setProps({ value: Number.POSITIVE_INFINITY });
  assert.equal(spinner.getAttribute("aria-valuenow"), null);
  assert.equal(spinner.getAttribute("data-progress-state"), "indeterminate");
  assert.equal(spinner.getAttribute("data-value"), null);
  assert.equal(spinner.getAttribute("data-percent"), null);
  handle.unmount();
});

test("lets ariaHidden make labelled progress spinners decorative", () => {
  const handle = mountInteraction(Spinner, {
    props: {
      ariaHidden: true,
      ariaLabel: "Ignored loading label",
      role: "progressbar",
      value: 1,
    },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-valuenow"), null);
  assert.equal(root.getAttribute("data-aria-state"), "decorative");
  assert.equal(root.getAttribute("data-progress-state"), "none");
  assert.equal(handle.queryByRole("status"), null);
  assert.equal(handle.queryByRole("progressbar"), null);
  handle.unmount();
});

test("updates visibility, loading, slot state, and exposed state", async () => {
  const handle = mountInteraction(Spinner, {
    props: {
      ariaLabel: "Export progress",
      role: "progressbar",
    },
    slots: {
      default: (state: SpinnerSlotState) =>
        `${state.state}:${state.progressState}:${state.visible}:${state.value}`,
    },
  });
  const exposed = handle.exposes<SpinnerExpose>();
  const root = handle.root();

  assert.ok(exposed.element === root);
  assert.equal(exposed.loading, true);
  assert.equal(exposed.visible, true);
  assert.equal(exposed.state, "loading");
  assert.equal(exposed.progressState, "indeterminate");
  assert.equal(exposed.value, null);
  assert.equal(root.textContent, "loading:indeterminate:true:null");

  await handle.wrapper.setProps({ loading: false, value: 20 });
  assert.equal(exposed.loading, false);
  assert.equal(exposed.state, "idle");
  assert.equal(exposed.progressState, "determinate");
  assert.equal(exposed.value, 20);
  assert.equal(root.textContent, "idle:determinate:true:20");

  await handle.wrapper.setProps({ visible: false });
  assert.equal(exposed.visible, false);
  assert.equal(exposed.state, "hidden");
  assert.ok(root.hasAttribute("hidden"));
  assert.equal(root.textContent, "hidden:determinate:false:20");
  handle.unmount();
});

test("honors explicit ids and labelledby naming", () => {
  const label = document.createElement("span");
  label.id = "import-spinner-label";
  label.textContent = "Import status";
  document.body.append(label);
  const handle = mountInteraction(Spinner, {
    props: {
      ariaLabelledby: "import-spinner-label",
      id: "import-spinner",
    },
  });
  const status = handle.getByRole("status", { name: "Import status" });

  assert.equal(status.id, "import-spinner");
  assert.equal(status.getAttribute("aria-labelledby"), "import-spinner-label");
  handle.unmount();
  label.remove();
});
