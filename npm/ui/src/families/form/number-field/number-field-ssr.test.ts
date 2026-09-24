import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import NumberField from "./number-field.vue";
import NumberFieldDecrement from "./number-field-decrement.vue";
import NumberFieldIncrement from "./number-field-increment.vue";
import NumberFieldInput from "./number-field-input.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

const Probe = defineComponent({
  name: "NumberFieldSsrProbe",
  setup: () => () =>
    h(
      NumberField,
      {
        ariaLabel: "Price",
        defaultValue: 1234.5,
        formatOptions: { style: "currency", currency: "EUR" },
        locale: "de-DE",
        max: 5000,
        min: 0,
        name: "price",
      },
      {
        default: () => [h(NumberFieldDecrement), h(NumberFieldInput), h(NumberFieldIncrement)],
      },
    ),
});

test("renders byte-identical spinbutton markup and hydrates without mismatches", async () => {
  const { html, host, dispose } = await renderAndHydrate(Probe);
  try {
    assert.match(html, /^<div part="root" data-vize-ui="number-field" data-state="in-range"/);
    assert.match(html, /type="hidden" name="price" value="1234\.5"/);
    assert.match(html, /role="spinbutton"/);
    assert.match(html, /value="1\.234,50 €"/);
    assert.match(html, /aria-valuenow="1234\.5"/);
    assert.match(html, /aria-valuemin="0"/);
    assert.match(html, /aria-valuemax="5000"/);
    assert.match(html, /id="vize-v-\d+-number-field"/);
    assert.doesNotMatch(html, /NaN|Infinity/);

    const input = host.querySelector<HTMLInputElement>('[role="spinbutton"]');
    const increment = host.querySelector('[data-vize-ui="number-field-increment"]');
    assert.ok(input);
    assert.equal(increment?.getAttribute("aria-controls"), input.id);
    increment?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    await nextTick();
    assert.equal(input.value, "1.235,00\u00a0€", "hydrated triggers step to the next grid value");
  } finally {
    dispose();
  }
});

test("omits unsafe bound attributes for unbounded and empty fields on the server", async () => {
  const { html, dispose } = await renderAndHydrate(
    defineComponent({
      name: "NumberFieldEmptySsrProbe",
      setup: () => () =>
        h(NumberField, { ariaLabel: "Offset" }, { default: () => h(NumberFieldInput) }),
    }),
  );
  try {
    assert.match(html, /data-state="empty"/);
    assert.match(html, /data-empty="true"/);
    assert.doesNotMatch(html, /aria-valuenow|aria-valuemin|aria-valuemax|aria-valuetext/);
    assert.doesNotMatch(html, /NaN|Infinity|type="hidden"/);
  } finally {
    dispose();
  }
});
