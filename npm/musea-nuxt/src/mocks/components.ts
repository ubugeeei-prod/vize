/**
 * Mock Nuxt built-in components.
 */

import { defineComponent, h } from "vue";

/**
 * Mock NuxtLink - renders as <RouterLink> or <a>.
 */
export const NuxtLink = defineComponent({
  name: "NuxtLink",
  props: {
    to: { type: [String, Object], default: "/" },
    href: { type: String, default: undefined },
    target: { type: String, default: undefined },
    rel: { type: String, default: undefined },
    external: { type: Boolean, default: false },
    replace: { type: Boolean, default: false },
    prefetch: { type: Boolean, default: true },
    noPrefetch: { type: Boolean, default: false },
    activeClass: { type: String, default: "router-link-active" },
    exactActiveClass: { type: String, default: "router-link-exact-active" },
  },
  setup(props, { slots }) {
    return () => {
      const to = props.href || props.to;
      if (props.external || (typeof to === "string" && to.startsWith("http"))) {
        return h(
          "a",
          {
            href: typeof to === "string" ? to : "/",
            target: props.target,
            rel: props.rel ?? (props.target === "_blank" ? "noopener noreferrer" : undefined),
          },
          slots.default?.(),
        );
      }

      // Try to use RouterLink if available
      try {
        const { RouterLink } = require("vue-router");
        return h(
          RouterLink,
          {
            to,
            replace: props.replace,
            activeClass: props.activeClass,
            exactActiveClass: props.exactActiveClass,
          },
          slots,
        );
      } catch {
        // Fallback to <a> if vue-router is not available
        return h("a", { href: typeof to === "string" ? to : "/" }, slots.default?.());
      }
    };
  },
});

/**
 * Mock NuxtPage - renders <RouterView> or slot content.
 */
export const NuxtPage = defineComponent({
  name: "NuxtPage",
  props: {
    name: { type: String, default: "default" },
    transition: { type: [Boolean, Object], default: undefined },
    keepalive: { type: [Boolean, Object], default: undefined },
    pageKey: { type: [String, Function], default: undefined },
  },
  setup(props, { slots }) {
    return () => {
      if (slots.default) {
        return slots.default();
      }
      try {
        const { RouterView } = require("vue-router");
        return h(RouterView, { name: props.name });
      } catch {
        return h("div", { "data-nuxt-page": "" }, "NuxtPage placeholder");
      }
    };
  },
});

/**
 * Mock ClientOnly - renders default slot on client side (always in browser context).
 */
export const ClientOnly = defineComponent({
  name: "ClientOnly",
  setup(_props, { slots }) {
    return () => slots.default?.() ?? null;
  },
});

/**
 * Mock NuxtLayout - renders slot content with optional layout wrapper.
 */
export const NuxtLayout = defineComponent({
  name: "NuxtLayout",
  props: {
    name: { type: String, default: "default" },
    fallback: { type: String, default: undefined },
  },
  setup(_props, { slots }) {
    return () => slots.default?.() ?? null;
  },
});

/**
 * Mock NuxtLoadingIndicator - renders nothing.
 */
export const NuxtLoadingIndicator = defineComponent({
  name: "NuxtLoadingIndicator",
  render() {
    return null;
  },
});

/**
 * Mock NuxtErrorBoundary - renders default slot.
 */
export const NuxtErrorBoundary = defineComponent({
  name: "NuxtErrorBoundary",
  setup(_props, { slots }) {
    return () => slots.default?.() ?? null;
  },
});
