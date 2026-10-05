import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import test from "node:test";

import { renderNuxtOxlintConfig } from "../../npm/framework/nuxt/src/lint/emitter.ts";

const directory = new URL(
  "../_fixtures/differential/linter/nuxt-oxlint-config-root/",
  import.meta.url,
);
const read = (name) => fs.readFileSync(new URL(name, directory), "utf8");
const corpus = JSON.parse(read("corpus.json"));

test("original Nuxt Oxlint issue, renderer, SFC and failed artifact remain whole", () => {
  const hashes = Object.fromEntries(
    Object.keys(corpus.files).map((name) => [
      name,
      createHash("sha256").update(read(name)).digest("hex"),
    ]),
  );
  assert.deepEqual(hashes, corpus.files);
  assert.deepEqual(Object.keys(corpus.files), [
    "original-issue.md",
    "render.mjs",
    "Input.vue.txt",
    "original-broken-config.json",
  ]);
  assert.deepEqual(corpus.originalPaths, ["app/pages/about.vue", "app/components/InfoCard.vue"]);
  assert.equal(
    read("Input.vue.txt"),
    '<script setup lang="ts">\nconst title = "About";\n</script>\n\n<template>\n  <h1 style="color: red">{{ title }}</h1>\n</template>\n',
  );
});

test("the original ordered plan renders a full valid root artifact while the old location refuses", () => {
  const expected = {
    plugins: ["vue"],
    jsPlugins: [{ name: "vize", specifier: "oxlint-plugin-vize" }],
    settings: { vize: { preset: "incremental" } },
    ignorePatterns: ["**/dist"],
    overrides: [
      { files: ["**/*.vue"], rules: { "vize/vue/no-inline-style": "warn" } },
      { files: ["app/pages/**/*.vue"], rules: { "vize/vue/no-inline-style": "off" } },
    ],
  };
  assert.equal(
    renderNuxtOxlintConfig(corpus.items, "oxlint-plugin-vize", {
      rootDir: "/project",
      configDir: "/project",
    }),
    JSON.stringify(expected, null, 2) + "\n",
  );
  assert.throws(
    () =>
      renderNuxtOxlintConfig(corpus.items, "oxlint-plugin-vize", {
        rootDir: "/project",
        configDir: "/project/.nuxt",
      }),
    { message: "Generated oxlint config must be in the lint root or an ancestor directory" },
  );
});
