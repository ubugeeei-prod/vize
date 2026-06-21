import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";

const nodeRequire = createRequire(`${process.cwd()}/package.json`);

export function createNuxtModuleResolver() {
  const moduleDir = dirname(nodeRequire.resolve("@vizejs/nuxt"));
  return {
    resolve: (...segments: string[]) => resolve(moduleDir, ...segments),
  };
}
