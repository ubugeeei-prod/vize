import fs from "node:fs";
import path from "node:path";
// renderNuxtOxlintConfig from @vizejs/nuxt 0.432.0 (internal chunk, imported by file path:
// node_modules/@vizejs/nuxt/dist/generation-CXADZreI.mjs, exported there as `o`)
import { o as renderNuxtOxlintConfig } from "./node_modules/@vizejs/nuxt/dist/generation-CXADZreI.mjs";

const rootDir = process.cwd();
const configDir = path.join(rootDir, ".nuxt");
const items = [
  { name: "ignores", ignores: ["**/dist"] },
  { name: "all vue", files: ["**/*.vue"], rules: { "vue/no-inline-style": "warn" } },
  { name: "pages", files: ["app/pages/**/*.vue"], rules: { "vue/no-inline-style": "off" } }
];
fs.mkdirSync(configDir, { recursive: true });
fs.writeFileSync(
  path.join(configDir, "oxlint.config.json"),
  renderNuxtOxlintConfig(items, "oxlint-plugin-vize", { rootDir, configDir })
);
