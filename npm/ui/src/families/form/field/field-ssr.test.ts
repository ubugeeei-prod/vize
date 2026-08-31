import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import FieldRoot from "./field.vue";
import FieldDescription from "./field-description.vue";
import FieldErrorMessage from "./field-error-message.vue";
import FieldLabel from "./field-label.vue";
import type { FieldRootSlotState } from "./field-types.ts";
import type { FormFieldError } from "../form/form.ts";
import { IdProvider } from "../../foundations/id/id.ts";

const errors: readonly FormFieldError[] = [
  { message: "Enter an email", name: "email", path: ["email"] },
];

const SsrProbe = defineComponent({
  name: "FieldCompositionSsrProbe",
  setup() {
    return () =>
      h(
        IdProvider,
        { prefix: "form", seed: "request" },
        {
          default: () =>
            h(
              FieldRoot,
              { errors, hasDescription: true, name: "email" },
              {
                default: ({ fieldProps }: FieldRootSlotState) => [
                  h(FieldLabel, null, { default: () => "Email" }),
                  h("input", { ...fieldProps, name: "email", type: "email" }),
                  h(FieldDescription, null, { default: () => "Work email" }),
                  h(FieldErrorMessage),
                ],
              },
            ),
        },
      );
  },
});

test("renders byte-identical deterministic field composition across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /data-vize-ui="field"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="invalid"/);
  assert.match(html, /id="form-request-field-0"/);
  assert.match(html, /id="form-request-field-0-label"/);
  assert.match(html, /for="form-request-field-0"/);
  assert.match(html, /aria-labelledby="form-request-field-0-label"/);
  assert.match(
    html,
    /aria-describedby="form-request-field-0-description form-request-field-0-error"/,
  );
  assert.match(html, /aria-errormessage="form-request-field-0-error"/);
  assert.match(html, /aria-invalid="true"/);
  assert.match(html, /Enter an email/);
});
