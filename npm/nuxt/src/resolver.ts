import { createRequire } from "node:module";
import { dirname } from "node:path";
import { createResolver } from "@nuxt/kit";

const nodeRequire = createRequire(`${process.cwd()}/package.json`);

export function createNuxtModuleResolver() {
  return createResolver(dirname(nodeRequire.resolve("@vizejs/nuxt")));
}
