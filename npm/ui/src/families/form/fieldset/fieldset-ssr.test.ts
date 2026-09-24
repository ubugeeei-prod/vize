import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import Fieldset from "./fieldset.vue";
import FieldsetDescription from "./fieldset-description.vue";
import FieldsetErrorMessage from "./fieldset-error-message.vue";
import FieldsetLegend from "./fieldset-legend.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

test("renders byte-identical fieldset markup and hydrates without mismatches", async () => {
  const { html, dispose } = await renderAndHydrate(
    defineComponent({
      name: "FieldsetSsrProbe",
      setup: () => () =>
        h(
          Fieldset,
          {
            name: "contact",
            hasDescription: true,
            errors: [{ name: "contact", message: "Pick one", path: ["contact"] }],
          },
          {
            default: () => [
              h(FieldsetLegend, null, { default: () => "Contact" }),
              h(FieldsetDescription, null, { default: () => "How we reach you" }),
              h(FieldsetErrorMessage),
            ],
          },
        ),
    }),
  );
  try {
    assert.match(html, /^<fieldset id="vize-v-\d+-field" name="contact"/);
    assert.match(html, /aria-describedby="vize-v-\d+-field-description vize-v-\d+-field-error"/);
    assert.match(html, /<legend[^>]*>(?:<!--\[-->)?Contact/);
    assert.match(html, />(?:<!--\[-->)?Pick one/);
  } finally {
    dispose();
  }
});
