import * as path from "node:path";
import { fileURLToPath } from "node:url";

import type { AppConfig } from "./apps.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PROJECTS_DIR = path.resolve(__dirname, "../_fixtures/_projects");

export const genericBuildApp: AppConfig = {
  name: "generic-build",
  cwd: path.join(PROJECTS_DIR, "generic-build"),
  command: "",
  args: [],
  port: 0,
  url: "",
  mountSelector: "",
  readyPattern: /./,
  startupTimeout: 0,
  check: {
    cwd: path.join(PROJECTS_DIR, "generic-build"),
    patterns: ["src/**/*.vue"],
  },
};

export const typecheckVueImportsApp: AppConfig = {
  name: "typecheck-vue-imports",
  cwd: path.join(PROJECTS_DIR, "typecheck-vue-imports"),
  command: "",
  args: [],
  port: 0,
  url: "",
  mountSelector: "",
  readyPattern: /./,
  startupTimeout: 0,
  check: {
    cwd: path.join(PROJECTS_DIR, "typecheck-vue-imports"),
    patterns: ["src/**/*.vue"],
    tsconfig: "tsconfig.json",
  },
};
