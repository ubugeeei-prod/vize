import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import {
  localeTextMatches,
  normalizeLocaleText,
  resolveDirection,
  resolveDisplayNames,
  resolveLocale,
  resolveNumberFormatter,
  resolveSearchCollator,
  useCollator,
  useDateTimeFormatter,
  useDirection,
  useDisplayNames,
  useListFormatter,
  useLocale,
  useNumberFormatter,
  useRelativeTimeFormatter,
  useSearchCollator,
} from "./locale.ts";
import LocaleProvider from "./locale-provider.vue";

test("provides default locale and direction", () => {
  const handle = mountInteraction(LocaleProvider, {
    slots: { default: "Hello" },
  });

  assert.equal(handle.root().getAttribute("data-vize-ui"), "locale");
  assert.equal(handle.root().getAttribute("lang"), "en-US");
  assert.equal(handle.root().getAttribute("dir"), "ltr");
  assert.equal(handle.root().textContent, "Hello");
  handle.unmount();
});

test("publishes an explicit rtl locale", async () => {
  const handle = mountInteraction(LocaleProvider, {
    props: { locale: "ar", direction: "rtl" },
    slots: {
      default: (props: { locale: string; direction: string }) =>
        `${props.locale}:${props.direction}`,
    },
  });

  assert.equal(handle.root().getAttribute("lang"), "ar");
  assert.equal(handle.root().getAttribute("dir"), "rtl");
  assert.match(handle.root().textContent ?? "", /ar:rtl/);
  handle.unmount();
});

test("resolves auto direction from the locale", () => {
  const resolved = resolveDirection("auto", "ar");
  assert.ok(resolved === "rtl" || resolved === "ltr");
  assert.equal(resolveDirection("rtl", "en-US"), "rtl");
  assert.equal(resolveDirection("ltr", "ar"), "ltr");
});

test("canonicalizes invalid locale tags before formatter construction", () => {
  assert.equal(resolveLocale(" ja-jp "), "ja-JP");
  assert.equal(resolveLocale("not a locale"), "en-US");
  assert.equal(resolveNumberFormatter("not a locale").resolvedOptions().locale, "en-US");
  assert.equal(
    resolveDisplayNames("not a locale", { type: "region" }).resolvedOptions().locale,
    "en-US",
  );
});

test("resolves formatters from the provider locale and explicit options", () => {
  const date = new Date(Date.UTC(2026, 0, 2, 3, 4, 5));
  let snapshot:
    | {
        readonly locale: string;
        readonly number: string;
        readonly numberLocale: string;
        readonly date: string;
        readonly dateLocale: string;
        readonly list: string;
        readonly listLocale: string;
        readonly relativeTime: string;
        readonly relativeTimeLocale: string;
      }
    | undefined;

  const Probe = defineComponent({
    name: "LocaleFormatterProbe",
    setup() {
      const locale = useLocale();
      const number = useNumberFormatter({ currency: "EUR", style: "currency" });
      const dateTime = useDateTimeFormatter({ dateStyle: "medium", timeZone: "UTC" });
      const list = useListFormatter({ style: "short", type: "conjunction" });
      const relativeTime = useRelativeTimeFormatter({ numeric: "auto", style: "long" });
      return () => {
        snapshot = {
          locale: locale.value,
          number: number.value.format(1234.5),
          numberLocale: number.value.resolvedOptions().locale,
          date: dateTime.value.format(date),
          dateLocale: dateTime.value.resolvedOptions().locale,
          list: list.value.format(["Alpha", "Beta", "Gamma"]),
          listLocale: list.value.resolvedOptions().locale,
          relativeTime: relativeTime.value.format(-1, "day"),
          relativeTimeLocale: relativeTime.value.resolvedOptions().locale,
        };
        return h(
          "span",
          [snapshot.number, snapshot.date, snapshot.list, snapshot.relativeTime].join("|"),
        );
      };
    },
  });

  const handle = mountInteraction(
    defineComponent({
      name: "LocaleFormatterHost",
      setup() {
        return () => h(LocaleProvider, { locale: "fr-FR" }, { default: () => h(Probe) });
      },
    }),
  );

  assert.deepEqual(snapshot, {
    locale: "fr-FR",
    number: new Intl.NumberFormat("fr-FR", { currency: "EUR", style: "currency" }).format(1234.5),
    numberLocale: "fr-FR",
    date: new Intl.DateTimeFormat("fr-FR", { dateStyle: "medium", timeZone: "UTC" }).format(date),
    dateLocale: "fr-FR",
    list: new Intl.ListFormat("fr-FR", { style: "short", type: "conjunction" }).format([
      "Alpha",
      "Beta",
      "Gamma",
    ]),
    listLocale: "fr-FR",
    relativeTime: new Intl.RelativeTimeFormat("fr-FR", {
      numeric: "auto",
      style: "long",
    }).format(-1, "day"),
    relativeTimeLocale: "fr-FR",
  });
  assert.equal(
    handle.root().textContent,
    [snapshot?.number, snapshot?.date, snapshot?.list, snapshot?.relativeTime].join("|"),
  );
  handle.unmount();
});

test("resolves display names and search collators from the provider locale", () => {
  let snapshot:
    | {
        readonly displayName: string | undefined;
        readonly displayLocale: string;
        readonly searchLocale: string;
        readonly searchUsage: string;
        readonly accentInsensitive: boolean;
      }
    | undefined;

  const Probe = defineComponent({
    name: "LocaleDisplayAndSearchProbe",
    setup() {
      const displayNames = useDisplayNames({ style: "long", type: "region" });
      const searchCollator = useSearchCollator();
      return () => {
        snapshot = {
          displayName: displayNames.value.of("US"),
          displayLocale: displayNames.value.resolvedOptions().locale,
          searchLocale: searchCollator.value.resolvedOptions().locale,
          searchUsage: searchCollator.value.resolvedOptions().usage,
          accentInsensitive:
            localeTextMatches("Cafe\u0301 noir", "cafe", {
              collator: searchCollator.value,
            }) &&
            localeTextMatches("Cafe\u0301 noir", "noir", {
              collator: searchCollator.value,
              match: "contains",
            }),
        };
        return h("span", snapshot.displayName);
      };
    },
  });

  const handle = mountInteraction(
    defineComponent({
      name: "LocaleDisplayAndSearchHost",
      setup() {
        return () => h(LocaleProvider, { locale: "fr-FR" }, { default: () => h(Probe) });
      },
    }),
  );

  const expectedDisplayNames = new Intl.DisplayNames("fr-FR", {
    style: "long",
    type: "region",
  });
  const expectedDisplayName = expectedDisplayNames.of("US");
  const expectedSearch = new Intl.Collator("fr-FR", {
    sensitivity: "base",
    usage: "search",
  });
  assert.deepEqual(snapshot, {
    displayName: expectedDisplayName,
    displayLocale: expectedDisplayNames.resolvedOptions().locale,
    searchLocale: expectedSearch.resolvedOptions().locale,
    searchUsage: "search",
    accentInsensitive: true,
  });
  assert.equal(handle.root().textContent, expectedDisplayName);
  handle.unmount();
});

test("matches normalized locale text with exact, prefix, and contains policies", () => {
  const collator = resolveSearchCollator("en-US");

  assert.equal(normalizeLocaleText("  Cafe\u0301\nnoir  "), "Café noir");
  assert.equal(
    localeTextMatches("  Cafe\u0301\nnoir  ", "café", {
      collator,
    }),
    true,
  );
  assert.equal(
    localeTextMatches("  Cafe\u0301\nnoir  ", "noir", {
      collator,
      match: "contains",
    }),
    true,
  );
  assert.equal(
    localeTextMatches("  Cafe\u0301\nnoir  ", "Café noir", {
      collator,
      match: "exact",
    }),
    true,
  );
  assert.equal(
    localeTextMatches("  Cafe\u0301\nnoir  ", "noir", {
      collator,
      match: "prefix",
    }),
    false,
  );
});

test("updates formatter composables when provider locale or options change", async () => {
  const locale = ref("en-US");
  const currency = ref("USD");
  let formatted = "";

  const Probe = defineComponent({
    name: "LocaleReactiveFormatterProbe",
    setup() {
      const formatter = useNumberFormatter(() => ({
        currency: currency.value,
        style: "currency",
      }));
      return () => {
        formatted = formatter.value.format(12);
        return h("span", formatted);
      };
    },
  });
  const Host = defineComponent({
    name: "LocaleReactiveFormatterHost",
    setup() {
      return () => h(LocaleProvider, { locale: locale.value }, { default: () => h(Probe) });
    },
  });

  const handle = mountInteraction(Host);
  assert.equal(
    formatted,
    new Intl.NumberFormat("en-US", { currency: "USD", style: "currency" }).format(12),
  );

  locale.value = "ja-JP";
  currency.value = "JPY";
  await nextTick();

  assert.equal(
    formatted,
    new Intl.NumberFormat("ja-JP", { currency: "JPY", style: "currency" }).format(12),
  );
  handle.unmount();
});

test("normalizes invalid provider locales before publishing context", () => {
  let formattedLocale: string | undefined;
  const Probe = defineComponent({
    name: "InvalidLocaleProbe",
    setup() {
      const locale = useLocale();
      const direction = useDirection();
      const number = useNumberFormatter();
      return () => {
        formattedLocale = number.value.resolvedOptions().locale;
        return h("span", `${locale.value}:${direction.value}:${formattedLocale}`);
      };
    },
  });

  const handle = mountInteraction(LocaleProvider, {
    props: { direction: "auto", locale: "not a locale" },
    slots: { default: () => h(Probe) },
  });

  assert.equal(handle.root().getAttribute("lang"), "en-US");
  assert.equal(handle.root().getAttribute("dir"), "ltr");
  assert.equal(handle.root().textContent, "en-US:ltr:en-US");
  handle.unmount();
});

test("falls back without a provider", () => {
  const Probe = defineComponent({
    name: "LocaleFallbackProbe",
    setup() {
      const locale = useLocale();
      const direction = useDirection();
      return () => h("span", { lang: locale.value, dir: direction.value }, locale.value);
    },
  });
  const handle = mountInteraction(Probe);
  assert.equal(handle.root().getAttribute("lang"), "en-US");
  assert.equal(handle.root().getAttribute("dir"), "ltr");
  handle.unmount();
});

test("rejects composable use outside setup", () => {
  assert.throws(() => useLocale(), /VIZE_UI_LOCALE_SETUP/);
  assert.throws(() => useDirection(), /VIZE_UI_LOCALE_SETUP/);
  assert.throws(() => useNumberFormatter(), /VIZE_UI_LOCALE_SETUP/);
  assert.throws(() => useDateTimeFormatter(), /VIZE_UI_LOCALE_SETUP/);
  assert.throws(() => useListFormatter(), /VIZE_UI_LOCALE_SETUP/);
  assert.throws(() => useRelativeTimeFormatter(), /VIZE_UI_LOCALE_SETUP/);
  assert.throws(() => useDisplayNames({ type: "region" }), /VIZE_UI_LOCALE_SETUP/);
  assert.throws(() => useCollator(), /VIZE_UI_LOCALE_SETUP/);
  assert.throws(() => useSearchCollator(), /VIZE_UI_LOCALE_SETUP/);
});
