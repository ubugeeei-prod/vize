import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import { nextSegmentIndex } from "../segmented-keyboard.ts";
import SegmentedItem from "../segmented-item.vue";
import Segmented from "../segmented.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function mountSegmented(
  props: Record<string, unknown> = {},
  items: readonly { value: string; disabled?: boolean; label: string }[] = [
    { value: "day", label: "Day" },
    { value: "week", label: "Week" },
    { value: "month", label: "Month" },
  ],
) {
  return mountInteraction(Segmented, {
    props: { label: "Billing", modelValue: "day", ...props },
    slots: {
      default: () =>
        items.map((item) =>
          h(SegmentedItem, { value: item.value, disabled: item.disabled }, () => item.label),
        ),
    },
  });
}

test("checks the selected radio and roves the tab stop", () => {
  const handle = mountSegmented();
  const group = handle.getByRole("radiogroup", { name: "Billing" });
  const day = handle.getByRole("radio", { name: "Day" });
  const week = handle.getByRole("radio", { name: "Week" });

  assert.equal(group.getAttribute("data-vize-ui"), "segmented");
  assert.equal(group.getAttribute("aria-orientation"), "horizontal");
  assert.equal(day.getAttribute("aria-checked"), "true");
  assert.equal(day.getAttribute("tabindex"), "0");
  assert.equal(week.getAttribute("aria-checked"), "false");
  assert.equal(week.getAttribute("tabindex"), "-1");
  handle.unmount();
});

test("moves and wraps with the arrow keys", async () => {
  const handle = mountSegmented();
  const day = handle.getByRole("radio", { name: "Day" });
  const month = handle.getByRole("radio", { name: "Month" });

  const next = await handle.press(day, "ArrowRight");
  assert.equal(next.keydownPrevented, true);
  assert.equal(handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0], "week");
  assert.equal(handle.activeElement(), handle.getByRole("radio", { name: "Week" }));

  await handle.press(month, "ArrowRight");
  assert.equal(handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0], "day");
  assert.equal(handle.activeElement(), day);
  handle.unmount();
});

test("skips disabled items", async () => {
  const handle = mountSegmented({}, [
    { value: "day", label: "Day" },
    { value: "week", label: "Week", disabled: true },
    { value: "month", label: "Month" },
  ]);
  await handle.press(handle.getByRole("radio", { name: "Day" }), "ArrowRight");
  assert.equal(handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0], "month");
  assert.equal(handle.getByRole("radio", { name: "Week" }).disabled, true);
  handle.unmount();
});

test("a disabled group ignores selection", async () => {
  const handle = mountSegmented({ disabled: true });
  const day = handle.getByRole("radio", { name: "Day" });

  assert.equal(handle.getByRole("radiogroup").getAttribute("aria-disabled"), "true");
  assert.equal(day.disabled, true);
  await handle.press(day, "ArrowRight");
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
  handle.unmount();
});

test("resolves roving indexes for both axes", () => {
  assert.equal(nextSegmentIndex(0, 3, "ArrowRight"), 1);
  assert.equal(nextSegmentIndex(0, 3, "ArrowDown"), 1);
  assert.equal(nextSegmentIndex(0, 3, "ArrowLeft"), 2);
  assert.equal(nextSegmentIndex(2, 3, "ArrowUp"), 1);
  assert.equal(nextSegmentIndex(0, 3, "ArrowUp"), 2);
  assert.equal(nextSegmentIndex(1, 3, "Home"), 0);
  assert.equal(nextSegmentIndex(1, 3, "End"), 2);
  assert.equal(nextSegmentIndex(1, 3, "Escape"), undefined);
});
