import assert from "node:assert/strict";

import { h } from "vue";

import FormWizard from "./form-wizard.vue";
import FormWizardBack from "./form-wizard-back.vue";
import FormWizardNext from "./form-wizard-next.vue";
import FormWizardProgress from "./form-wizard-progress.vue";
import FormWizardStep from "./form-wizard-step.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const steps = ["one", "two"] as const;

const render = () =>
  h(
    FormWizard,
    { steps, id: "wizard", ariaLabel: "Wizard" },
    {
      default: () => [
        ...steps.map((step) => h(FormWizardStep, { key: step, step }, { default: () => step })),
        h(FormWizardProgress),
        h(FormWizardBack, null, { default: () => "Back" }),
        h(FormWizardNext, null, { default: () => "Next" }),
      ],
    },
  );

function assertServerMarkup(html: string): void {
  assert.match(html, /data-vize-ui="form-wizard"/);
  assert.match(html, /id="wizard-two"[^>]*hidden/);
  assert.match(html, /<progress/);
}

function assertHydratedDom(host: HTMLElement): void {
  assert.equal(host.querySelector<HTMLElement>("#wizard-one")?.hidden, false);
  assert.equal(
    host.querySelector<HTMLButtonElement>('[data-vize-ui="form-wizard-back"]')?.disabled,
    true,
  );
}

export const formWizardRuntimeFixtures: readonly RuntimeFixture[] = [
  "form-wizard.vue",
  "form-wizard-step.vue",
  "form-wizard-next.vue",
  "form-wizard-back.vue",
  "form-wizard-progress.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/form/form-wizard/${file}`,
  render,
  assertServerMarkup,
  assertHydratedDom,
}));
