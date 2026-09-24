import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import CheckboxGroup from "./checkbox-group.vue";
import CheckboxGroupItem from "./checkbox-group-item.vue";
import CheckboxGroupSelectAll from "./checkbox-group-select-all.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

const options = ["email", "sms", "push"] as const;

const Probe = defineComponent({
  name: "CheckboxGroupSsrProbe",
  setup: () => () =>
    h(
      CheckboxGroup,
      { options, defaultValue: ["sms"], name: "channels", ariaLabel: "Channels" },
      {
        default: () => [
          h("label", [h(CheckboxGroupSelectAll), "All"]),
          ...options.map((option) =>
            h("label", { key: option }, [h(CheckboxGroupItem, { value: option }), option]),
          ),
        ],
      },
    ),
});

test("renders byte-identical checkbox group markup and hydrates without mismatches", async () => {
  const { html, host, dispose } = await renderAndHydrate(Probe);
  try {
    assert.match(html, /^<div role="group" aria-label="Channels"/);
    assert.match(html, /data-state="some"/);
    assert.match(html, /aria-checked="mixed"/);
    assert.match(html, /name="channels" value="sms" checked/);
    assert.doesNotMatch(html, /name="channels" value="email" checked/);

    const parent = host.querySelector<HTMLInputElement>(
      '[data-vize-ui="checkbox-group-select-all"]',
    );
    assert.ok(parent);
    assert.equal(parent.indeterminate, true, "hydration applies the mixed DOM state");
    parent.click();
    await nextTick();
    const checked = [
      ...host.querySelectorAll<HTMLInputElement>('[data-vize-ui="checkbox-group-item"]'),
    ]
      .filter((input) => input.checked)
      .map((input) => input.value);
    assert.deepEqual(checked, ["email", "sms", "push"]);
  } finally {
    dispose();
  }
});
