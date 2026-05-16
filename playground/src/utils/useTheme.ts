import { computed, inject, type ComputedRef } from "vue";

/** Inject the theme provided by App.vue */
export function useTheme() {
  const _injectedTheme = inject<ComputedRef<"dark" | "light">>("theme");
  const theme = computed<"dark" | "light">(() => _injectedTheme?.value ?? "light");
  return { theme };
}
