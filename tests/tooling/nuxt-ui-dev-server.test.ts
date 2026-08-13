import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

test("nuxt-ui startup waits for a healthy SSR page before returning ready", () => {
  const source = readRepoFile("tests", "app", "dev", "nuxt-ui-dev-server.ts");

  assert.doesNotMatch(source, /waitForHttpReady/);
  assert.match(source, /const BOOT_ATTEMPTS = 3;/);
  assert.match(source, /const SSR_READY_TIMEOUT_MS = 90_000;/);
  assert.match(source, /AbortSignal\.timeout\(SSR_READY_TIMEOUT_MS\)/);
  assert.match(source, /isHealthyNuxtUiSsrResponse\(response\.status, body\)/);
  assert.match(source, /DEAD_NUXT_UI_SSR_BRIDGE\.test\(body\) \|\| hasDeadSsrBridge/);
  assert.match(source, /throw new Error\(`\$\{app\.name\} dev server failed startup:/);
});

test("nuxt-ui warmups do not abort slow hosted SSR compilation too early", () => {
  const source = readRepoFile("tests", "app", "dev", "nuxt-ui.spec.ts");

  assert.match(source, /const SSR_WARMUP_REQUEST_TIMEOUT_MS = 90_000;/);
  assert.match(source, /const BROWSER_WARMUP_NAVIGATION_TIMEOUT_MS = 90_000;/);
  assert.match(source, /DEAD_NUXT_UI_SSR_BRIDGE\.test\(body\)/);
  assert.match(source, /DEAD_NUXT_UI_SSR_BRIDGE\.test\(html\)/);
  assert.match(source, /isHealthyNuxtUiSsrResponse\(res\.status, body\)/);
  assert.match(source, /isHealthyNuxtUiSsrResponse\(status, html\)/);
});

test("nuxt-ui setup avoids the Nuxt Content module on hosted readiness runners", () => {
  const source = readRepoFile("tests", "_helpers", "apps.ts");
  const start = source.indexOf("export const nuxtUiApp: AppConfig = {");
  const end = source.indexOf("export const rekaUiApp: AppConfig = {");
  assert.notEqual(start, -1);
  assert.notEqual(end, -1);
  const nuxtUiSetup = source.slice(start, end);

  assert.doesNotMatch(nuxtUiSetup, /@nuxt\/content/);
  assert.match(nuxtUiSetup, /enabled: false/);
  assert.match(nuxtUiSetup, /content: true/);
});
