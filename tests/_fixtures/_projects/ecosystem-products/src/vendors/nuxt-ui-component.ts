import type { DefineComponent } from "vue";

export const nuxtUiComponentModules = [
  "@nuxt/ui/components/Button.vue",
  "@nuxt/ui/components/Card.vue",
  "@nuxt/ui/components/Input.vue",
] as const;

declare const component: DefineComponent<Record<string, unknown>, {}, any>;

export default component;
