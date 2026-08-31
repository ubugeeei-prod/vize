import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import { useFieldWiring } from "./field-wiring.ts";
import { IdProvider } from "../../foundations/id/id.ts";

const FieldProbe = defineComponent({
  name: "FieldWiringSsrFieldProbe",
  setup() {
    const wiring = useFieldWiring({ hasDescription: true, invalid: true });
    return () =>
      h("div", [
        h("label", wiring.labelProps.value, "Email"),
        h("input", { ...wiring.fieldProps.value, type: "email" }),
        h("p", wiring.descriptionProps.value, "We never share it."),
        h("p", wiring.errorMessageProps.value, "Enter a valid address."),
      ]);
  },
});

const SsrProbe = defineComponent({
  name: "FieldWiringSsrProbe",
  setup() {
    return () =>
      h(IdProvider, { prefix: "form", seed: "request" }, { default: () => h(FieldProbe) });
  },
});

test("renders byte-identical deterministic wiring across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /id="form-request-field-0"/);
  assert.match(html, /for="form-request-field-0"/);
  assert.match(html, /aria-labelledby="form-request-field-0-label"/);
  assert.match(
    html,
    /aria-describedby="form-request-field-0-description form-request-field-0-error"/,
  );
  assert.match(html, /aria-errormessage="form-request-field-0-error"/);
  assert.match(html, /aria-invalid="true"/);
});
