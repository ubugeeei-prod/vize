import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import LocaleProvider from "./locale-provider.vue";

const SsrProbe = defineComponent({
  name: "LocaleSsrProbe",
  setup() {
    return () =>
      h(LocaleProvider, { locale: "ja-JP", direction: "ltr" }, { default: () => "本文" });
  },
});

test("renders byte-identical locale markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0], /lang="ja-JP"/);
  assert.match(outputs[0], /dir="ltr"/);
  assert.match(outputs[0], /data-vize-ui="locale"/);
});
