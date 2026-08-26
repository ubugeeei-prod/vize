import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import { mountInteraction } from "./testing/mount.ts";
import { resolveDirection, useDirection, useLocale } from "./locale.ts";
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
});
