import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import FormWizard from "./form-wizard.vue";
import FormWizardNext from "./form-wizard-next.vue";
import FormWizardProgress from "./form-wizard-progress.vue";
import FormWizardStep from "./form-wizard-step.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

const steps = ["details", "confirm"] as const;

test("renders the default step on the server and applies drafts only after hydration", async () => {
  let loads = 0;
  const Probe = defineComponent({
    name: "FormWizardSsrProbe",
    setup: () => () =>
      h(
        FormWizard,
        {
          steps,
          ariaLabel: "Checkout",
          draft: {
            load: () => {
              loads += 1;
              return { step: "confirm", visited: ["details", "confirm"] };
            },
            save: () => undefined,
          },
        },
        {
          default: () => [
            ...steps.map((step) => h(FormWizardStep, { key: step, step }, { default: () => step })),
            h(FormWizardProgress),
            h(FormWizardNext, null, { default: () => "Next" }),
          ],
        },
      ),
  });
  const { html, host, dispose } = await renderAndHydrate(Probe);
  try {
    assert.equal(loads, 1, "only the hydrating client loads the draft");
    assert.match(html, /data-step="details"/);
    assert.match(html, /aria-valuetext="Step 1 of 2"/);
    assert.match(html, /id="vize-v-\d+-wizard-confirm"[^>]*hidden/);
    await nextTick();
    assert.equal(host.firstElementChild?.getAttribute("data-step"), "confirm");
  } finally {
    dispose();
  }
});
