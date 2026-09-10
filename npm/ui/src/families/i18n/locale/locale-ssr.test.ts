import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  useDateTimeFormatter,
  useDisplayNames,
  useNumberFormatter,
  useSearchCollator,
} from "./locale.ts";
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

type SsrFormatterDefaults = {
  readonly calendar: string;
  readonly locale: string;
  readonly numberingSystem: string;
  readonly timeZone: string;
  readonly timeZoneDisambiguation: "earlier" | "later";
};

function createFormatterDefaultsProbe(defaults: SsrFormatterDefaults) {
  const Inner = defineComponent({
    name: "LocaleDefaultsSsrInner",
    setup() {
      const dateTime = useDateTimeFormatter({ dateStyle: "short" });
      return () => dateTime.value.resolvedOptions().timeZone;
    },
  });

  return defineComponent({
    name: "LocaleDefaultsSsrProbe",
    setup() {
      return () =>
        h(LocaleProvider, defaults, {
          default: (props: {
            readonly calendar: string | undefined;
            readonly numberingSystem: string | undefined;
            readonly timeZone: string;
            readonly timeZoneDisambiguation: string;
          }) =>
            h("span", [
              `${props.numberingSystem}:${props.calendar}:${props.timeZone}:${props.timeZoneDisambiguation}:`,
              h(Inner),
            ]),
        });
    },
  });
}

test("isolates locale formatter defaults during concurrent SSR", async () => {
  const first = {
    calendar: "japanese",
    locale: "ja-JP",
    numberingSystem: "arab",
    timeZone: "Asia/Tokyo",
    timeZoneDisambiguation: "earlier",
  } as const;
  const second = {
    calendar: "gregory",
    locale: "en-US",
    numberingSystem: "latn",
    timeZone: "UTC",
    timeZoneDisambiguation: "later",
  } as const;

  const outputs = await Promise.all([
    renderToString(createSSRApp(createFormatterDefaultsProbe(first))),
    renderToString(createSSRApp(createFormatterDefaultsProbe(second))),
  ]);
  assert.match(outputs[0], /data-vize-ui-numbering-system="arab"/);
  assert.match(outputs[0], /data-vize-ui-calendar="japanese"/);
  assert.match(outputs[0], /data-vize-ui-time-zone="Asia\/Tokyo"/);
  assert.match(outputs[0], /data-vize-ui-time-zone-disambiguation="earlier"/);
  assert.match(outputs[0], />arab:japanese:Asia\/Tokyo:earlier:Asia\/Tokyo</);
  assert.doesNotMatch(outputs[0], /latn:gregory:UTC:later/);
  assert.match(outputs[1], /data-vize-ui-numbering-system="latn"/);
  assert.match(outputs[1], /data-vize-ui-calendar="gregory"/);
  assert.match(outputs[1], /data-vize-ui-time-zone="UTC"/);
  assert.match(outputs[1], /data-vize-ui-time-zone-disambiguation="later"/);
  assert.match(outputs[1], />latn:gregory:UTC:later:UTC</);
  assert.doesNotMatch(outputs[1], /arab:japanese:Asia\/Tokyo:earlier/);
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

test("uses the SSR fallback locale for display names and search collators", async () => {
  const Probe = defineComponent({
    name: "LocaleSearchSsrProbe",
    setup() {
      const displayNames = useDisplayNames({ type: "region" });
      const collator = useSearchCollator();
      return () =>
        h(
          "span",
          `${displayNames.value.resolvedOptions().locale}:${collator.value.resolvedOptions().usage}`,
        );
    },
  });

  const output = await renderToString(createSSRApp(Probe));
  assert.match(output, />en-US:search</);
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
