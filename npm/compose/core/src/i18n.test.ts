import assert from "node:assert/strict";
import { test } from "node:test";
import { createApp, createSSRApp, defineComponent, effect, h } from "vue";
import { renderToString } from "vue/server-renderer";

import { defineI18n, defineLocale, defineMessages, useI18n } from "./i18n.ts";
import { withTrappedServerGlobals } from "./testing/ssr-harness.ts";

const messages = defineMessages({
  en: {
    greeting: "Hello {name}!",
    cart: { summary: "{count, plural, =0 {Your cart is empty} one {# item} other {# items}}" },
    onlyEnglish: "fallback text",
  },
  ja: {
    greeting: "こんにちは、{name}さん",
    cart: { summary: "{count, plural, other {# 個の商品}}" },
    onlyEnglish: "",
  },
});

const french = defineLocale<typeof messages>()({
  greeting: "Bonjour {name} !",
  cart: { summary: "{count, plural, one {# article} other {# articles}}" },
  onlyEnglish: "texte",
});

function deferred<Value>() {
  let resolve: (value: Value) => void = () => undefined;
  let reject: (reason: unknown) => void = () => undefined;
  const promise = new Promise<Value>((onResolve, onReject) => {
    resolve = onResolve;
    reject = onReject;
  });
  return { promise, resolve, reject };
}

void test("translates typed keys with parameters and nested keys", () => {
  const i18n = defineI18n({ messages }).create();
  assert.equal(i18n.locale.value, "en");
  assert.equal(i18n.t("greeting", { name: "Ada" }), "Hello Ada!");
  assert.equal(i18n.t("cart.summary", { count: 0 }), "Your cart is empty");
  assert.equal(i18n.t("cart.summary", { count: 3 }), "3 items");
  assert.equal(i18n.te("cart.summary"), true);
});

void test("locale switching is reactive and uses Intl for the active locale", async () => {
  const i18n = defineI18n({ messages }).create({ locale: "en" });
  const seen: string[] = [];
  effect(() => {
    seen.push(i18n.t("cart.summary", { count: 1200 }));
  });
  await i18n.setLocale("ja");
  assert.deepEqual(seen, ["1,200 items", "1,200 個の商品"]);
  assert.equal(i18n.direction.value, "ltr");
  assert.equal(i18n.intl.locale.value, "ja");
});

void test("falls back to the fallback locale and reports missing keys", () => {
  const missing: [string, string][] = [];
  const definition = defineI18n({
    messages: { en: { a: "A", b: "B" }, de: { a: "A (de)", b: "B" } },
    fallbackLocale: "en",
    onMissing: (key, locale) => missing.push([key, locale]),
  });
  const i18n = definition.create({ locale: "de" });
  assert.equal(i18n.t("a"), "A (de)");
  const loose: (key: string) => string = (key) => Reflect.apply(i18n.t, undefined, [key]);
  assert.equal(loose("zzz"), "zzz");
  assert.deepEqual(missing, [["zzz", "de"]]);
});

void test("empty translations are used as-is (not treated as missing)", () => {
  const i18n = defineI18n({ messages }).create({ locale: "ja" });
  assert.equal(i18n.t("onlyEnglish"), "");
});

void test("lazy locales load once, report progress, and activate", async () => {
  const load = deferred<{ default: typeof french }>();
  let calls = 0;
  const definition = defineI18n({
    messages,
    loaders: {
      fr: () => {
        calls += 1;
        return load.promise;
      },
    },
  });
  const i18n = definition.create();
  assert.deepEqual(i18n.availableLocales, ["en", "ja", "fr"]);
  const switching = i18n.setLocale("fr");
  const again = i18n.loadLocale("fr");
  assert.equal(i18n.isLoading.value, true);
  assert.equal(i18n.locale.value, "en", "the locale changes only once loaded");
  load.resolve({ default: french });
  await switching;
  await again;
  assert.equal(calls, 1);
  assert.equal(i18n.isLoading.value, false);
  assert.equal(i18n.locale.value, "fr");
  assert.deepEqual(i18n.loadedLocales.value, ["en", "ja", "fr"]);
  assert.equal(i18n.t("cart.summary", { count: 2 }), "2 articles");
});

void test("an initial lazy locale is ready after awaiting ready", async () => {
  const definition = defineI18n({ messages, loaders: { fr: async () => french } });
  const i18n = definition.create({ locale: "fr" });
  await i18n.ready;
  assert.equal(i18n.t("greeting", { name: "Zoé" }), "Bonjour Zoé !");
});

void test("loader failures surface through error and reject", async () => {
  const failure = new Error("offline");
  const definition = defineI18n({ messages, loaders: { fr: () => Promise.reject(failure) } });
  const i18n = definition.create();
  await assert.rejects(i18n.setLocale("fr"), /offline/);
  assert.equal(i18n.error.value, failure);
  assert.equal(i18n.locale.value, "en");
  assert.equal(i18n.isLoading.value, false);
});

void test("rejects unknown locales and dotted keys", () => {
  const definition = defineI18n({ messages });
  assert.throws(
    () => Reflect.apply(definition.create, undefined, [{ locale: "xx" }]),
    /VIZE_COMPOSE_I18N_UNKNOWN_LOCALE/,
  );
  assert.throws(
    () => Reflect.apply(defineMessages, undefined, [{ en: { "a.b": "x" } }]),
    /VIZE_COMPOSE_I18N_INVALID_KEY/,
  );
});

void test("instances are provided to the app and injected by use()", () => {
  const definition = defineI18n({ messages });
  const i18n = definition.create({ locale: "ja" });
  const app = createApp({});
  app.use(i18n);
  assert.equal(
    app.runWithContext(() => useI18n(definition).t("greeting", { name: "太郎" })),
    "こんにちは、太郎さん",
  );
  assert.throws(() => createApp({}).runWithContext(() => definition.use()), /NOT_PROVIDED/);
});

void test("server rendering is per request, deterministic, and matches the client", async () => {
  const definition = defineI18n({ messages, loaders: { fr: async () => ({ default: french }) } });
  const Child = defineComponent({
    setup() {
      const { t, locale } = definition.use();
      return () => h("p", { lang: locale.value }, t("cart.summary", { count: 2 }));
    },
  });
  const render = async (locale: "en" | "ja" | "fr"): Promise<string> => {
    const i18n = definition.create({ locale });
    await i18n.ready;
    const app = createSSRApp({ render: () => h(Child) });
    app.use(i18n);
    return renderToString(app);
  };

  const outputs = await withTrappedServerGlobals(() =>
    Promise.all([render("en"), render("ja"), render("fr"), render("en")]),
  );
  assert.deepEqual(outputs, [
    '<p lang="en">2 items</p>',
    '<p lang="ja">2 個の商品</p>',
    '<p lang="fr">2 articles</p>',
    '<p lang="en">2 items</p>',
  ]);

  const client = definition.create({ locale: "fr" });
  await client.ready;
  assert.equal(`<p lang="fr">${client.t("cart.summary", { count: 2 })}</p>`, outputs[2]);
});
