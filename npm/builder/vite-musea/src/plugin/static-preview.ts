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
  if (vueVersion === undefined) return;
}

function hasLegacyVueSfcCompilerPlugin(plugins: readonly Pick<{ name?: string }, "name">[]) {
  return plugins.some((plugin) => {
    const name = plugin.name ?? "";
    return name === "vite:vue2" || name.includes("plugin-vue2") || name.includes("vue2");
  });
}
