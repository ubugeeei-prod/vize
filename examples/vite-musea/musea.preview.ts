import { watch, type App } from "vue";
import type { MuseaPreviewContext } from "@vizejs/vite-plugin-musea";

export default function setup(app: App, context?: MuseaPreviewContext) {
  if (!context) return;
  const { globals } = context;
  const stop = watch(
    globals,
    (values) => {
      const root = document.documentElement;
      root.lang = String(values.locale);
      root.dataset.brand = String(values.brand);
      root.dataset.scheme = String(values.scheme);
      root.style.colorScheme = String(values.scheme);
      for (const name of [
        "--musea-accent",
        "--musea-accent-hover",
        "--musea-paper",
        "--musea-ink",
      ]) {
        root.style.removeProperty(name);
      }
      if (values.brand === "ocean") {
        root.style.setProperty("--musea-accent", "#176b92");
        root.style.setProperty("--musea-accent-hover", "#125574");
      }
      if (values.scheme === "dark") {
        root.style.setProperty("--musea-paper", "#1b2430");
        root.style.setProperty("--musea-ink", "#f1f5f9");
      }
    },
    { immediate: true },
  );
  // Props controls can remount the app: release this app's watcher first.
  app.onUnmount(stop);
  app.provide("musea-globals", globals);
}
