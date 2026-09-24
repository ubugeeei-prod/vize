import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import PasswordField from "./password-field.vue";
import PasswordFieldInput from "./password-field-input.vue";
import PasswordFieldToggle from "./password-field-toggle.vue";
import { estimatePasswordStrength } from "./password-field-strength.ts";
import type { PasswordStrengthEstimate } from "./password-field-strength.ts";
import type { PasswordFieldSlotState } from "./password-field-types.ts";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

const Probe = defineComponent({
  name: "PasswordFieldSsrProbe",
  setup: () => () =>
    h(
      PasswordField,
      {
        ariaLabel: "New password",
        autocomplete: "new-password",
        defaultValue: "Abcdefg1",
        evaluateStrength: estimatePasswordStrength,
        name: "password",
      },
      {
        default: (state: PasswordFieldSlotState<PasswordStrengthEstimate>) => [
          h(PasswordFieldInput),
          h(PasswordFieldToggle),
          h("meter", { max: 4, value: state.strength?.score ?? 0 }),
        ],
      },
    ),
});

test("renders byte-identical password markup and hydrates without mismatches", async () => {
  const { html, host, dispose } = await renderAndHydrate(Probe);
  try {
    assert.match(html, /^<div part="root" data-vize-ui="password-field" data-state="hidden"/);
    assert.match(html, /type="password"/);
    assert.match(html, /autocomplete="new-password"/);
    assert.match(html, /aria-pressed="false"/);
    assert.match(html, /<meter max="4" value="3"/);

    const input = host.querySelector("input");
    const toggle = host.querySelector("button");
    assert.ok(input && toggle);
    assert.equal(toggle.getAttribute("aria-controls"), input.id);
    toggle.click();
    await nextTick();
    assert.equal(input.type, "text", "hydrated toggle is interactive");
  } finally {
    dispose();
  }
});
