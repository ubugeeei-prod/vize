import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";

export function createNuxtModuleResolver(rootDir: string) {
  const moduleDir = dirname(createRequire(`${rootDir}/package.json`).resolve("@vizejs/nuxt"));
  return {
    resolve: (...segments: string[]) => resolve(moduleDir, ...segments),
  };
}
