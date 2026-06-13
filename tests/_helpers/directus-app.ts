import path from "node:path";
import { fileURLToPath } from "node:url";

import type { AppConfig } from "./apps.ts";

const TESTS_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DIRECTUS_DIR = path.join(TESTS_DIR, "_fixtures", "_git", "directus");

export const directusApp: AppConfig = {
  name: "directus",
  cwd: DIRECTUS_DIR,
  command: "npx",
  args: ["-y", "pnpm@10", "dev"],
  port: 0,
  url: "",
  mountSelector: "",
  readyPattern: /./,
  startupTimeout: 0,
  check: {
    cwd: DIRECTUS_DIR,
    patterns: ["app/src/**/*.vue"],
    tsconfig: "app/tsconfig.json",
  },
  lint: {
    cwd: DIRECTUS_DIR,
    patterns: ["app/src/**/*.vue"],
  },
};
