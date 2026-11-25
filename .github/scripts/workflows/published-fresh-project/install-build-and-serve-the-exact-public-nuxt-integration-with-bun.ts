import { writeFile } from "node:fs/promises";
import { requiredEnv } from "../../../../tools/ci/lib/scripts/files.ts";

const root = requiredEnv("PROJECT");
const version = process.env.RELEASE_VERSION;
await writeFile(
  `${root}/package.json`,
  `${JSON.stringify(
    {
      name: "vize-public-nuxt-bun",
      private: true,
      type: "module",
      scripts: { build: "nuxt build" },
      dependencies: {
        "@vizejs/nuxt": version,
        nuxt: "4.6.4",
        vue: "3.5.43",
      },
    },
    null,
    2,
  )}\n`,
);
await writeFile(
  `${root}/nuxt.config.ts`,
  `export default defineNuxtConfig({\n  compatibilityDate: "2025-07-15",\n  devtools: { enabled: false },\n  modules: ["@vizejs/nuxt"],\n  vize: { lint: false },\n});\n`,
);
await writeFile(
  `${root}/app/app.vue`,
  `<script setup lang="ts">\nconst marker: string = "vize-public-nuxt-bun";\n</script>\n\n<template>\n  <main id="public-nuxt-marker">{{ marker }}</main>\n</template>\n\n<style scoped>\nmain { color: rgb(12 34 56); }\n</style>\n`,
);
