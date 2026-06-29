import type { MuseaOptions } from "../types/index.js";

export function resolveStaticPreviewVueVersion(
  configured: MuseaOptions["vueVersion"],
  plugins: readonly Pick<{ name?: string }, "name">[],
): MuseaOptions["vueVersion"] {
  if (configured !== undefined) return configured;
  if (hasLegacyVueSfcCompilerPlugin(plugins)) return 2;
  return 3;
}

export function assertStaticPreviewRuntimeSupported(
  vueVersion: MuseaOptions["vueVersion"],
  staticBuildEnabled: boolean,
): void {
  if (!staticBuildEnabled) return;
  if (vueVersion === 3 || vueVersion === undefined) return;

  throw new Error(
    "[musea] Static Musea builds currently require the Vue 3 preview runtime. Vue 2/Nuxt 2 projects should use the dev gallery or run Nuxt static generation with @vizejs/nuxt until a Vue 2 static preview runtime is available.",
  );
}

function hasLegacyVueSfcCompilerPlugin(plugins: readonly Pick<{ name?: string }, "name">[]) {
  return plugins.some((plugin) => {
    const name = plugin.name ?? "";
    return name === "vite:vue2" || name.includes("plugin-vue2") || name.includes("vue2");
  });
}
