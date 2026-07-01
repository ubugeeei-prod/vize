/**
 * Mock Nuxt built-in components.
 */

import { defineComponent, h } from "vue";
import type { PropType } from "vue";

import { resolveNavigationTarget } from "../context.js";
import { navigateTo } from "./navigation.js";
import { useRoute } from "./composables.js";
import type { NuxtMuseaNavigationTarget } from "../types.js";

/**
 * Mock NuxtLink - renders as <RouterLink> or <a>.
 */
export const NuxtLink = defineComponent({
  name: "NuxtLink",
  props: {
    to: { type: [String, Object] as PropType<NuxtMuseaNavigationTarget>, default: "/" },
    href: { type: String, default: undefined },
    target: { type: String, default: undefined },
    rel: { type: String, default: undefined },
    external: { type: Boolean, default: false },
    replace: { type: Boolean, default: false },
    prefetch: { type: Boolean, default: true },
    noPrefetch: { type: Boolean, default: false },
    activeClass: { type: String, default: "router-link-active" },
    exactActiveClass: { type: String, default: "router-link-exact-active" },
    custom: { type: Boolean, default: false },
  },
  setup(props, { slots }) {
    return () => {
      const target = props.href ?? props.to;
      const href = typeof target === "string" ? target : routeTargetToHref(target);
      const navigate = () => navigateTo(target, { replace: props.replace });

      if (props.custom) {
        return slots.default?.({
          href,
          navigate,
          route: useRoute(),
          isActive: false,
          isExactActive: false,
        });
      }

      return h(
        "a",
        {
          "data-nuxt-link": "",
          href,
          target: props.target,
          rel: props.rel ?? (props.target === "_blank" ? "noopener noreferrer" : undefined),
          onClick: (event: MouseEvent) => {
            if (props.external || props.target === "_blank" || isExternalHref(href)) return;
            event.preventDefault();
            void navigate();
          },
        },
        slots.default?.(),
      );
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
      return h("div", { "data-nuxt-page": props.name }, "NuxtPage placeholder");
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

export const NuxtRouteAnnouncer = defineComponent({
  name: "NuxtRouteAnnouncer",
  props: {
    politeness: { type: String, default: "polite" },
  },
  setup(props) {
    return () =>
      h("span", {
        "aria-live": props.politeness,
        "data-nuxt-route-announcer": "",
        style: "position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0);",
      });
  },
});

export const NuxtWelcome = defineComponent({
  name: "NuxtWelcome",
  setup() {
    return () => h("div", { "data-nuxt-welcome": "" }, "NuxtWelcome placeholder");
  },
});

export const NuxtIsland = defineComponent({
  name: "NuxtIsland",
  setup(_props, { slots }) {
    return () => slots.default?.() ?? null;
  },
});

export const NuxtClientFallback = defineComponent({
  name: "NuxtClientFallback",
  setup(_props, { slots }) {
    return () => slots.default?.() ?? slots.fallback?.() ?? null;
  },
});

export const NuxtImg = defineComponent({
  name: "NuxtImg",
  props: {
    src: { type: String, required: true },
    alt: { type: String, default: "" },
    width: { type: [String, Number], default: undefined },
    height: { type: [String, Number], default: undefined },
  },
  setup(props, { attrs }) {
    return () =>
      h("img", {
        ...attrs,
        src: props.src,
        alt: props.alt,
        width: props.width,
        height: props.height,
      });
  },
});

export const NuxtPicture = defineComponent({
  name: "NuxtPicture",
  props: {
    src: { type: String, required: true },
    alt: { type: String, default: "" },
  },
  setup(props, { attrs }) {
    return () => h("picture", {}, [h("img", { ...attrs, src: props.src, alt: props.alt })]);
  },
});

function routeTargetToHref(to: NuxtMuseaNavigationTarget): string {
  return resolveNavigationTarget(to).fullPath;
}

function isExternalHref(href: string): boolean {
  return /^(?:[a-z][a-z0-9+.-]*:)?\/\//i.test(href);
}
