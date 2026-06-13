import path from "node:path";
import { fileURLToPath } from "node:url";

import type { AppConfig } from "./apps.ts";

const TESTS_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const GIT_DIR = path.join(TESTS_DIR, "_fixtures", "_git");

function componentLibraryApp(
  name: string,
  fixtureDir: string,
  patterns: string[],
  tsconfig?: string,
): AppConfig {
  const cwd = path.join(GIT_DIR, fixtureDir);

  return {
    name,
    cwd,
    command: "npx",
    args: ["-y", "pnpm@10", "dev"],
    port: 0,
    url: "",
    mountSelector: "",
    readyPattern: /./,
    startupTimeout: 0,
    check: {
      cwd,
      patterns,
      tsconfig,
    },
    lint: {
      cwd,
      patterns,
    },
  };
}

export const primeVueApp = componentLibraryApp(
  "primevue",
  "primevue",
  [
    "packages/primevue/src/**/*.vue",
    "packages/icons/src/**/*.vue",
    "packages/forms/src/**/*.vue",
    "apps/showcase/**/*.vue",
    "apps/volt/**/*.vue",
  ],
  "packages/primevue/tsconfig.json",
);

export const vuetifyApp = componentLibraryApp(
  "vuetify",
  "vuetify",
  ["packages/vuetify/**/*.vue", "packages/docs/src/**/*.vue"],
  "tsconfig.json",
);

export const naiveUiApp = componentLibraryApp(
  "naive-ui",
  "naive-ui",
  ["src/**/*.vue", "demo/**/*.vue", "generic/**/*.vue"],
  "tsconfig.json",
);
