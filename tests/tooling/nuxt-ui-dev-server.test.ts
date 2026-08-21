import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  VITE_NODE_REQUEST_TIMEOUT_MS,
  withViteNodeRequestBudget,
} from "../app/dev/nuxt-ui-vite-node.ts";
import { normalizeNuxtUiSnapshotHtml } from "../app/dev/nuxt-ui-snapshot.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

test("nuxt-ui startup waits for a healthy SSR page before returning ready", () => {
  const source = readRepoFile("tests", "app", "dev", "nuxt-ui-dev-server.ts");

  assert.doesNotMatch(source, /waitForHttpReady/);
  assert.match(source, /const BOOT_ATTEMPTS = 3;/);
  assert.match(source, /const SSR_READY_TIMEOUT_MS = 180_000;/);
  assert.match(source, /const deadline = Date\.now\(\) \+ SSR_READY_TIMEOUT_MS;/);
  assert.match(source, /AbortSignal\.timeout\(remainingMs\)/);
  assert.match(source, /isHealthyNuxtUiSsrResponse\(response\.status, body\)/);
  assert.match(source, /DEAD_NUXT_UI_SSR_BRIDGE\.test\(body\) \|\| hasDeadSsrBridge/);
  assert.match(source, /throw new Error\(`\$\{app\.name\} dev server failed startup:/);
});

test("nuxt-ui boots widen the playground vite-node request budget", () => {
  const source = readRepoFile("tests", "app", "dev", "nuxt-ui-dev-server.ts");

  // A budget under our own SSR readiness window would let vite-node reject the slow
  // first module fetch and destroy the bridge before a probe can observe it.
  assert.equal(VITE_NODE_REQUEST_TIMEOUT_MS, 600_000);
  assert.ok(VITE_NODE_REQUEST_TIMEOUT_MS > 180_000);
  assert.match(source, /ensureViteNodeRequestBudget\(\);\n  await ensurePortFree/);

  const patched = withViteNodeRequestBudget(
    [
      "export default defineNuxtConfig({",
      "  vite: {",
      "    optimizeDeps: { include: [] },",
      "  },",
      "",
      "  compatibilityDate: '2024-07-09',",
      "})",
      "",
    ].join("\n"),
  );
  assert.match(patched, /vite: \{\n {4}viteNode: \{ requestTimeout: 600000 \},\n {4}optimizeDeps:/);
  assert.equal(withViteNodeRequestBudget(patched), patched);

  const withoutViteBlock = withViteNodeRequestBudget(
    ["export default defineNuxtConfig({", "  compatibilityDate: '2024-07-09',", "})", ""].join(
      "\n",
    ),
  );
  assert.match(withoutViteBlock, /vite: \{\n {4}viteNode: \{ requestTimeout: 600000 \},\n {2}\},/);
});

test("nuxt-ui warmups do not abort slow SSR compilation too early", () => {
  const source = readRepoFile("tests", "app", "dev", "nuxt-ui.spec.ts");

  assert.match(source, /const SSR_WARMUP_REQUEST_TIMEOUT_MS = 90_000;/);
  assert.match(source, /const BROWSER_WARMUP_NAVIGATION_TIMEOUT_MS = 90_000;/);
  assert.match(source, /DEAD_NUXT_UI_SSR_BRIDGE\.test\(body\)/);
  assert.match(source, /DEAD_NUXT_UI_SSR_BRIDGE\.test\(html\)/);
  assert.match(source, /isHealthyNuxtUiSsrResponse\(res\.status, body\)/);
  assert.match(source, /isHealthyNuxtUiSsrResponse\(status, html\)/);
});

test("nuxt-ui SSR snapshots ignore dev-only entry scripts", () => {
  const cwd = "/tmp/nuxt ui";
  const encodedCwd = encodeURIComponent(cwd);
  const normalized = normalizeNuxtUiSnapshotHtml(
    [
      `<head><link rel="modulepreload" as="script" crossorigin href="/_nuxt${encodedCwd}/node_modules/nuxt/dist/app/entry.async.js">`,
      '<script type="module" src="/_nuxt/@vite/client" crossorigin></script>',
      `<script type="module" src="/_nuxt${encodedCwd}/node_modules/nuxt/dist/app/entry.async.js" crossorigin></script>`,
      '<meta name="description" content="Explore and test all Nuxt UI components in an interactive environment"><script>',
      "if (!window.__NUXT_DEVTOOLS_TIME_METRIC__) {",
      "  Object.defineProperty(window, '__NUXT_DEVTOOLS_TIME_METRIC__', {",
      "    value: {},",
      "    enumerable: false,",
      "    configurable: true,",
      "  })",
      "}",
      "window.__NUXT_DEVTOOLS_TIME_METRIC__.appInit = Date.now()",
      '</script><script>"use strict";</script></head>',
      `<body>${cwd}<script type="application/json" data-nuxt-logs="nuxt-app">[{"date":1234567890123}]</script></body>`,
    ].join("\n"),
    { cwd },
  );

  assert.equal(
    normalized,
    [
      '<head><meta name="description" content="Explore and test all Nuxt UI components in an interactive environment"><script>"use strict";</script></head>',
      '<body>__NUXT_UI_WORKTREE__<script type="application/json" data-nuxt-logs="nuxt-app">__NUXT_UI_LOGS__</script></body>',
    ].join("\n"),
  );
});

test("nuxt-ui setup avoids the Nuxt Content module on readiness runners", () => {
  const source = readRepoFile("tests", "_helpers", "apps.ts");
  const configPatch = readRepoFile("tests", "_helpers", "nuxt-ui-config.ts");
  const start = source.indexOf("export const nuxtUiApp: AppConfig = {");
  const end = source.indexOf("export const rekaUiApp: AppConfig = {");
  assert.notEqual(start, -1);
  assert.notEqual(end, -1);
  const nuxtUiSetup = source.slice(start, end);

  assert.doesNotMatch(nuxtUiSetup, /@nuxt\/content/);
  assert.doesNotMatch(configPatch, /@nuxt\/content/);
  assert.match(configPatch, /enabled: false/);
  assert.match(configPatch, /content: true/);
});
