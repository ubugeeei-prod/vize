import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite-plus";

// Exercise the unchanged UI suite against the Vue actually shipped by an example.
const example = fileURLToPath(new URL("../builder/vite/example/package.json", import.meta.url));
const requireExample = createRequire(example);
const vueManifest = fs.realpathSync(requireExample.resolve("vue/package.json"));
const requireVue = createRequire(vueManifest);
const sha256 = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const names = [
  "vue",
  "@vue/compiler-sfc",
  "@vue/compiler-dom",
  "@vue/compiler-core",
  "@vue/runtime-dom",
  "@vue/runtime-core",
  "@vue/reactivity",
  "@vue/shared",
  "@vue/server-renderer",
];
const parents: Record<string, string> = {
  "@vue/compiler-core": "@vue/compiler-dom",
  "@vue/runtime-core": "@vue/runtime-dom",
  "@vue/reactivity": "@vue/runtime-core",
};
const scopes = new Map<string, NodeJS.Require>();
const packages = names.map((name) => {
  const scope = parents[name] ? scopes.get(parents[name]) : requireVue;
  assert.ok(scope);
  const manifestPath = fs.realpathSync(scope.resolve(`${name}/package.json`));
  const bytes = fs.readFileSync(manifestPath);
  const manifest = JSON.parse(bytes.toString());
  assert.equal(manifest.name, name);
  assert.equal(manifest.version, "3.5.42");
  const directory = path.dirname(manifestPath);
  scopes.set(name, createRequire(manifestPath));
  const entry = name === "vue" ? "dist/vue.runtime.esm-bundler.js" : manifest.module;
  assert.equal(typeof entry, "string");
  const entryPath = fs.realpathSync(path.join(directory, entry));
  return {
    name,
    version: manifest.version,
    manifestPath,
    manifestSha256: sha256(bytes),
    entryPath,
    entrySha256: sha256(fs.readFileSync(entryPath)),
  };
});
const entry = (name: string) => {
  const selected = packages.find((item) => item.name === name);
  assert.ok(selected);
  return selected.entryPath;
};
const compiler = requireVue("@vue/compiler-sfc");
assert.equal(compiler.version, "3.5.42");
const compilerPath = fs.realpathSync(requireVue.resolve("@vue/compiler-sfc"));
console.log(
  JSON.stringify({
    scope: "installed-production-vue-ui",
    example,
    packages,
    compilerPath,
    compilerSha256: sha256(fs.readFileSync(compilerPath)),
  }),
);

export default defineConfig({
  plugins: [
    vue({ compiler }),
    {
      name: "production-vue-module-custody",
      enforce: "post",
      transform(code, id, options) {
        if (id.split("?")[0].endsWith("/native-select/native-select.vue")) {
          console.log(
            JSON.stringify({
              scope: "production-vue-native-select-module",
              id,
              ssr: options?.ssr ?? false,
              compilerVersion: compiler.version,
              sourceSha256: sha256(fs.readFileSync(id.split("?")[0])),
              codeSha256: sha256(code),
              code,
            }),
          );
        }
      },
    },
  ],
  resolve: {
    alias: [
      { find: /^vue\/server-renderer$/, replacement: entry("@vue/server-renderer") },
      { find: /^vue\/compiler-sfc$/, replacement: entry("@vue/compiler-sfc") },
      ...packages.map(({ name, entryPath }) => ({
        find: new RegExp(`^${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`),
        replacement: entryPath,
      })),
    ],
  },
  test: {
    environment: "happy-dom",
    include: ["src/**/*.test.ts"],
    server: { deps: { inline: ["@vue/test-utils"] } },
  },
});
