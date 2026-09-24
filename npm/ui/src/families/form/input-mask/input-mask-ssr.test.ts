import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import MaskedInput from "./masked-input.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

test("renders byte-identical masked markup and hydrates without mismatches", async () => {
  const { html, host, dispose } = await renderAndHydrate(
    defineComponent({
      name: "MaskedInputSsrProbe",
      setup: () => () =>
        h(MaskedInput, {
          mask: "99/99/9999",
          lazy: false,
          defaultValue: "0102",
          ariaLabel: "Birthday",
          name: "birthday",
        }),
    }),
  );
  try {
    assert.match(html, /^<input id="vize-v-\d+-masked-input" type="text" name="birthday"/);
    assert.match(html, /value="01\/02\/____"/);
    assert.match(html, /inputmode="numeric"/);
    assert.match(html, /data-state="incomplete"/);

    const input = host.querySelector("input");
    assert.ok(input);
    input.value = "01/02/2024";
    input.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText" }));
    await nextTick();
    assert.equal(input.getAttribute("data-state"), "complete", "hydrated input is interactive");
  } finally {
    dispose();
  }
});
