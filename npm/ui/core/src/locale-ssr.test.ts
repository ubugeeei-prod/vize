import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import { useNumberFormatter } from "./locale.ts";
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

test("uses the SSR fallback locale for formatters without a provider", async () => {
  const Probe = defineComponent({
    name: "LocaleFormatterSsrProbe",
    setup() {
      const formatter = useNumberFormatter({ style: "unit", unit: "byte" });
      return () => h("span", formatter.value.resolvedOptions().locale);
    },
  });

  const output = await renderToString(createSSRApp(Probe));
  assert.match(output, />en-US</);
});

test("normalizes invalid provider locales during SSR", async () => {
  const Probe = defineComponent({
    name: "InvalidLocaleSsrProbe",
    setup() {
      return () =>
        h(
          LocaleProvider,
          { direction: "auto", locale: "not a locale" },
          {
            default: (props: { readonly locale: string; readonly direction: string }) =>
              `${props.locale}:${props.direction}`,
          },
        );
    },
  });

  const output = await renderToString(createSSRApp(Probe));
  assert.match(output, /lang="en-US"/);
  assert.match(output, /dir="ltr"/);
  assert.match(output, />en-US:ltr</);
});
