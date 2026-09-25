import type { ComposableProbeMap } from "./probe-types.ts";

/** Conformance probes for catalog entries in group G (test-only). */
export const probesGroupG: ComposableProbeMap = {
  "./i18n": {
    kind: "component",
    setup: `import { defineI18n } from "@/i18n.ts";
const appI18n = defineI18n({
  messages: {
    en: { greeting: "Hello {name}", cart: "{count, plural, one {# item} other {# items}}" },
    ja: { greeting: "こんにちは {name}", cart: "{count} 点" },
  },
});
const i18n = appI18n.create({ locale: "en" });`,
    state: `{ locale: i18n.locale.value, greeting: i18n.t("greeting", { name: "Vize" }), cart: i18n.t("cart", { count: 2 }) }`,
    server: { locale: "en", greeting: "Hello Vize", cart: "2 items" },
    client: { locale: "en", greeting: "Hello Vize", cart: "2 items" },
  },
  "./icu-message": {
    kind: "component",
    setup: `import { formatMessage, parseMessage } from "@/icu-message.ts";
const text = formatMessage("en-US", "{count, plural, =0 {none} one {# file} other {# files}}", { count: 3 });
const nodes = parseMessage("Hi {name}").length;`,
    state: "{ text, nodes }",
    server: { text: "3 files", nodes: 2 },
    client: { text: "3 files", nodes: 2 },
  },
  "./resolver": {
    kind: "exempt",
    reason:
      "Build-time auto-import resolver for Vite/Nuxt config; it imports node:fs and never runs in a component or ships to the browser.",
  },
  "./use-form-field-props": {
    kind: "component",
    setup: `import { useForm } from "@/use-form.ts";
import { useFormFieldProps } from "@/use-form-field-props.ts";
const form = useForm({ initialValues: { email: "ada@example.com" } });
const email = useFormFieldProps(form, "email");`,
    state:
      "{ id: email.id.value, invalid: email.invalid.value, value: email.modelProps.value.modelValue, label: email.labelProps.value.for }",
    server: { id: "field-email", invalid: false, value: "ada@example.com", label: "field-email" },
    client: { id: "field-email", invalid: false, value: "ada@example.com", label: "field-email" },
  },
  "./use-route-query": {
    kind: "component",
    setup: `import { queryParsers, useRouteQuery } from "@/use-route-query.ts";
const route = { params: {}, query: { page: "3", q: "vue" } };
const page = useRouteQuery(route, "page", { parse: queryParsers.integer, default: 1 });
const q = useRouteQuery(route, "q");`,
    state: "{ page, q }",
    server: { page: 3, q: "vue" },
    client: { page: 3, q: "vue" },
  },
  "./use-visual-viewport": {
    kind: "component",
    setup: `import { useVisualViewport } from "@/use-visual-viewport.ts";
const fakeHost = {
  visualViewport: Object.assign(new EventTarget(), {
    width: 390, height: 500, offsetTop: 0, offsetLeft: 0, scale: 1,
  }),
  innerHeight: 844,
};
const viewport = useVisualViewport({ host: typeof window === "undefined" ? undefined : fakeHost });`,
    state:
      "{ width: viewport.width.value, height: viewport.height.value, keyboardOpen: viewport.keyboardOpen.value, supported: viewport.isSupported.value }",
    server: { width: 0, height: 0, keyboardOpen: false, supported: false },
    client: { width: 390, height: 500, keyboardOpen: true, supported: true },
  },
};
