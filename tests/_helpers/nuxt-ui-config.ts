import * as fs from "node:fs";

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
