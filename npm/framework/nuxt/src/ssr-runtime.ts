// Nitro must see the same external Vue runtime from Nuxt's entry and from
// Vize-compiled SFC chunks. Nuxt 4's Rolldown server build otherwise inlines
// Vue into its entry while Nitro externalizes Vue from the async SFC chunks.
const VUE_SSR_EXTERNAL = ["vue", "vue/server-renderer", "@vue/server-renderer"] as const;
const VUE_RUNTIME_DEDUPE = [
  "vue",
  "@vue/reactivity",
  "@vue/runtime-core",
  "@vue/runtime-dom",
  "@vue/shared",
];

export function dedupeVueRuntimePackages(vite: { resolve?: { dedupe?: string[] } }): void {
  vite.resolve ||= {};
  const dedupe = new Set(vite.resolve.dedupe ?? []);
  for (const packageName of VUE_RUNTIME_DEDUPE) {
    dedupe.add(packageName);
  }
  vite.resolve.dedupe = [...dedupe];
}

type NuxtServerViteConfig = {
  build?: {
    rolldownOptions?: {
      external?: unknown;
    };
  };
};

export function externalizeVueRuntimeForNuxtSsr(config: NuxtServerViteConfig): void {
  const rolldownOptions = config.build?.rolldownOptions;
  if (!rolldownOptions) return;

  const previous = rolldownOptions.external;
  if (typeof previous === "function") {
    rolldownOptions.external = (id: string, ...args: unknown[]) =>
      VUE_SSR_EXTERNAL.some((specifier) => specifier === id) || previous(id, ...args);
    return;
  }

  const existing = previous == null ? [] : Array.isArray(previous) ? previous : [previous];
  rolldownOptions.external = [...new Set([...existing, ...VUE_SSR_EXTERNAL])];
}
