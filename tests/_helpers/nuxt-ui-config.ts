import * as fs from "node:fs";

// Disabling devtools and serving Nuxt UI's own prose components instead of the
// Nuxt Content module keeps the playground light enough for hosted readiness,
// but both drop markup from the SSR payload (the devtools time-metric bootstrap
// script and the `mdc` public runtime config). Any change here therefore has to
// come with regenerated `tests/app/dev/nuxt-ui.spec.ts-snapshots` fixtures.
const DEVTOOLS_ENABLED = "  devtools: {\n    enabled: true\n  },";
const DEVTOOLS_DISABLED = "  devtools: {\n    enabled: false\n  },";
const CSS_ENTRY = "  css: ['~/assets/css/main.css'],";

export function patchNuxtUiPlaygroundConfig(configPath: string): string {
  const source = fs.readFileSync(configPath, "utf-8");
  let nextSource = source.replace(DEVTOOLS_ENABLED, DEVTOOLS_DISABLED);
  if (!nextSource.includes("content: true")) {
    nextSource = nextSource.replace(CSS_ENTRY, `${CSS_ENTRY}\n\n  ui: { content: true },`);
  }
  if (nextSource !== source) {
    fs.writeFileSync(configPath, nextSource);
  }
  return nextSource;
}
