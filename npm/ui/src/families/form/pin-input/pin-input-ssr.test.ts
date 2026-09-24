import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import PinInput from "./pin-input.vue";
import PinInputField from "./pin-input-field.vue";
import type { PinInputSlotState } from "./pin-input-types.ts";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

const Probe = defineComponent({
  name: "PinInputSsrProbe",
  setup: () => () =>
    h(
      PinInput,
      { length: 4, name: "otp", defaultValue: "42", ariaLabel: "Code" },
      {
        default: (state: PinInputSlotState) =>
          state.indexes.map((index) => h(PinInputField, { key: index, index })),
      },
    ),
});

test("renders byte-identical code fields and hydrates without mismatches", async () => {
  const { html, host, dispose } = await renderAndHydrate(Probe);
  try {
    assert.match(html, /^<div role="group" aria-label="Code" part="root" data-vize-ui="pin-input"/);
    assert.match(html, /type="hidden" name="otp" value="42"/);
    assert.match(html, /id="vize-v-\d+-pin-0"[^>]*autocomplete="one-time-code"/);
    assert.equal(html.match(/data-vize-ui="pin-input-field"/g)?.length, 4);

    const inputs = host.querySelectorAll<HTMLInputElement>('[data-vize-ui="pin-input-field"]');
    const third = inputs[2];
    assert.ok(third);
    third.value = "7";
    third.dispatchEvent(new Event("input", { bubbles: true }));
    await nextTick();
    assert.equal(host.querySelector<HTMLInputElement>('input[type="hidden"]')?.value, "427");
  } finally {
    dispose();
  }
});
