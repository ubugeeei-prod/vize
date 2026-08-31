import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ErrorSummary from "./error-summary.vue";

const SsrProbe = defineComponent({
  name: "ErrorSummarySsrProbe",
  setup() {
    return () =>
      h(ErrorSummary, {
        autoFocus: false,
        fields: [{ id: "email", label: "Email", message: "Enter a valid address" }],
        heading: "There is a problem",
      });
  },
});

test("renders a byte-identical labelled summary across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /data-vize-ui="error-summary"/);
  assert.match(html, /role="group"/);
  assert.match(html, /tabindex="-1"/);
  assert.match(html, /aria-labelledby="[^"]+"/);
  assert.match(html, /href="#email"/);
  assert.match(html, /There is a problem/);
  assert.match(html, /Email: Enter a valid address/);
});

test("renders only the host while every field is valid on the server", async () => {
  const EmptyProbe = defineComponent({
    name: "ErrorSummaryEmptySsrProbe",
    setup() {
      return () => h(ErrorSummary);
    },
  });
  const html = await renderToString(createSSRApp(EmptyProbe));
  assert.match(html, /data-vize-ui="error-summary-host"/);
  assert.doesNotMatch(html, /data-vize-ui="error-summary"[^-]/);
  assert.doesNotMatch(html, /role="group"/);
});
