import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { build, createServer, type InlineConfig } from "vite";
import router from "vue-router/vite";

import { vize } from "./index.ts";
import { linkWorkspaceVue } from "./test/startup-scan-project.ts";

const require = createRequire(import.meta.url);

for (const scanPatterns of [undefined, []]) {
  const mode = scanPatterns ? "on demand" : "with the startup scan";
  void test(`#13502 Vue Router reads definePage metadata ${mode}`, async (t) => {
    const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-define-page-")));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    linkWorkspaceVue(root);
    fs.symlinkSync(
      path.dirname(require.resolve("vue-router/package.json")),
      path.join(root, "node_modules/vue-router"),
      process.platform === "win32" ? "junction" : "dir",
    );
    fs.mkdirSync(path.join(root, "src/pages"), { recursive: true });
    fs.writeFileSync(path.join(root, "src/access.ts"), "export const isPublic = true;");
    fs.writeFileSync(
      path.join(root, "src/pages/index.vue"),
      `<script setup lang="ts">
import { isPublic } from "../access";
definePage({ meta: { public: isPublic } });
</script>
<template><p>hello</p></template>`,
    );
    const entry = path.join(root, "routes.mjs");
    fs.writeFileSync(entry, 'export { routes } from "vue-router/auto-routes";');
    const config = (): InlineConfig => ({
      root,
      configFile: false,
      logLevel: "silent",
      plugins: [
        router({ root, dts: false, watch: false }),
        vize({ configMode: false, scanPatterns }),
      ],
    });

    const server = await createServer({
      ...config(),
      server: { middlewareMode: true, ws: false },
      optimizeDeps: { noDiscovery: true, include: [] },
    });
    try {
      const { routes } = await server.ssrLoadModule("/routes.mjs");
      assert.deepEqual(routes[0].meta, { public: true });
    } finally {
      await server.close();
    }

    await build({
      ...config(),
      build: { lib: { entry, formats: ["es"], fileName: () => "routes.mjs" } },
    });
    const { routes } = await import(pathToFileURL(path.join(root, "dist/routes.mjs")).href);
    assert.deepEqual(routes[0].meta, { public: true });
  });
}
