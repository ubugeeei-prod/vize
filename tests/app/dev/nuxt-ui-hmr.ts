import { expect, type Locator, type Page } from "@playwright/test";
import type { ChildProcess } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import {
  collectConsoleErrors,
  collectHydrationErrors,
  isFatalError,
} from "../../_helpers/assertions";
import { getProcessLogs } from "../../_helpers/server";
import { installSourceRestore } from "./source-restore";

const OPTIMIZE_RELOAD = /optimized dependencies changed\. reloading/i;
const ROUTE_RULES_RELOAD = /page reload virtual:nuxt:.*route-rules\.mjs/i;

// Nuxt serves the playground from its own Vite root (`playgrounds/nuxt/app`),
// so the authored SFC is addressed by a root-relative module URL instead of its
// repo-relative path. Match the root-relative suffix, which also holds if Nuxt
// ever serves the file through `/@fs/<absolute path>`.
const PROBE_MODULE_PATH = "/components/Matrix.vue.ts";
const PROBE_HMR_UPDATE = /hmr update .*\/components\/Matrix\.vue\.ts\?vue&vize/;
const PROBE_HOT_UPDATED = /\[vite\] hot updated: .*\/components\/Matrix\.vue\.ts\?vue&vize/;

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Nuxt UI discovers client dependencies lazily, then regenerates route rules and
 * reloads the page. Do not edit an SFC while that bounded startup cycle is still
 * replacing Vite's module graph, or Vite can only see the raw, non-accepting SFC.
 */
export async function waitForNuxtUiHmrReady(
  page: Page,
  devServer: ChildProcess,
  logStart: number,
  rehydrate: () => Promise<void>,
): Promise<void> {
  const deadline = Date.now() + 120_000;
  const probe = `nuxt-ui-ready-${Date.now()}`;
  let stableSince = Date.now();
  let startupActivityAt = stableSince;
  let observedOptimizeIndex = -1;
  let routeRulesObservedAt: number | undefined;

  const setProbe = async () => {
    await page.evaluate((value) => {
      (window as Window & { __vizeNuxtUiHmrProbe?: string }).__vizeNuxtUiHmrProbe = value;
    }, probe);
  };
  await setProbe();

  while (Date.now() < deadline) {
    const logs = getProcessLogs(devServer).slice(logStart);
    const lastOptimizeReload = logs.findLastIndex((line) => OPTIMIZE_RELOAD.test(line));
    const lastRouteRulesReload = logs.findLastIndex((line) => ROUTE_RULES_RELOAD.test(line));
    if (lastOptimizeReload > observedOptimizeIndex) {
      observedOptimizeIndex = lastOptimizeReload;
      startupActivityAt = Date.now();
    }
    const retained = await page
      .evaluate(() => (window as Window & { __vizeNuxtUiHmrProbe?: string }).__vizeNuxtUiHmrProbe)
      .catch(() => undefined);

    if (retained !== probe) {
      await rehydrate();
      await setProbe();
      stableSince = Date.now();
      startupActivityAt = stableSince;
    }

    const stableFor = Date.now() - stableSince;
    if (lastOptimizeReload < 0 && stableFor >= 15_000) return;
    if (lastRouteRulesReload > lastOptimizeReload) {
      routeRulesObservedAt ??= Date.now();
      if (Date.now() - routeRulesObservedAt >= 5_000 && stableFor >= 5_000) return;
    } else {
      routeRulesObservedAt = undefined;
    }
    if (lastOptimizeReload >= 0 && Date.now() - Math.max(stableSince, startupActivityAt) >= 75_000)
      return;
    await sleep(500);
  }

  throw new Error("Nuxt UI client kept reloading throughout the 120s readiness window");
}

/**
 * Nuxt 4.5 rewrites `.nuxt/route-rules.mjs` on the first app generation that
 * follows an authored source change and reloads the page through that virtual
 * module. The reload races the SSR render, so the client can come back on the
 * pre-edit module graph with the HMR patch already discarded. Nuxt only rewrites
 * that template once per dev server, so spend it on a throwaway edit instead of
 * letting it swallow the update under test.
 */
async function absorbNuxtTemplateRegeneration(options: {
  devServer: ChildProcess;
  sourcePath: string;
  originalSource: string;
  warmupSource: string;
  original: Locator;
  warmup: Locator;
  waitForHydration: () => Promise<void>;
}): Promise<void> {
  const {
    devServer,
    sourcePath,
    originalSource,
    warmupSource,
    original,
    warmup,
    waitForHydration,
  } = options;
  const logStart = getProcessLogs(devServer).length;
  const deadline = Date.now() + 30_000;
  fs.writeFileSync(sourcePath, warmupSource);
  try {
    while (Date.now() < deadline) {
      if (
        getProcessLogs(devServer)
          .slice(logStart)
          .some((line) => ROUTE_RULES_RELOAD.test(line))
      )
        break;
      // A patch that lands on its own means there is no regeneration to absorb.
      if ((await warmup.count().catch(() => 0)) > 0) break;
      await sleep(250);
    }
  } finally {
    fs.writeFileSync(sourcePath, originalSource);
  }
  await expect(original).toBeVisible({ timeout: 60_000 });
  await expect(warmup).toHaveCount(0, { timeout: 60_000 });
  await waitForHydration();
}

export async function verifyNuxtUiAuthoredSourceHmr(options: {
  page: Page;
  devServer: ChildProcess;
  startupLogStart: number;
  cwd: string;
  mountSelector: string;
  appName: string;
  goto: () => Promise<unknown>;
  waitForHydration: () => Promise<void>;
}): Promise<void> {
  const { page, devServer, startupLogStart, cwd, mountSelector, appName, goto, waitForHydration } =
    options;
  // Nuxt UI 4.5 watches its published component directory and regenerates
  // Nuxt templates when a library component changes. That upstream watcher
  // reloads route-rules.mjs, which cannot prove that Vize accepted an authored
  // SFC update without a page reload. Probe the playground-owned Matrix SFC
  // rendered by the same route instead; it exercises Vize HMR without crossing
  // the library template-regeneration boundary. Nuxt still rewrites that
  // template once per dev server, which the warm-up edit below absorbs.
  const originalText = '<div class="flex items-start gap-2 min-h-0">';
  const warmupText = '<div data-vize-hmr-warmup="true" class="flex items-start gap-2 min-h-0">';
  const updatedProbe = `updated-${Date.now()}`;
  const updatedText = `<div data-vize-hmr-probe="${updatedProbe}" class="flex items-start gap-2 min-h-0">`;
  const sourcePath = path.join(cwd, "playgrounds/nuxt/app/components/Matrix.vue");
  const originalSource = fs.readFileSync(sourcePath, "utf8");
  expect(originalSource.split(originalText)).toHaveLength(2);
  const warmupSource = originalSource.replace(originalText, warmupText);
  const updatedSource = originalSource.replace(originalText, updatedText);
  expect(warmupSource).toContain('data-vize-hmr-warmup="true"');
  expect(updatedSource).toContain(`data-vize-hmr-probe="${updatedProbe}"`);

  const consoleErrors = await collectConsoleErrors(page, appName);
  const hydrationErrors = await collectHydrationErrors(page);
  const initialClientModule = page.waitForResponse(
    (response) => {
      const url = new URL(response.url());
      return (
        response.ok() &&
        url.pathname.endsWith(PROBE_MODULE_PATH) &&
        url.searchParams.has("vize") &&
        !url.searchParams.has("vize-ssr")
      );
    },
    { timeout: 180_000 },
  );
  await goto();
  const mount = page.locator(mountSelector);
  const original = mount.getByRole("button", { name: "Button" }).last();
  const warmup = mount.locator('[data-vize-hmr-warmup="true"]').first();
  const updated = mount.locator(`[data-vize-hmr-probe="${updatedProbe}"]`).first();
  await expect(original).toBeVisible();
  await expect(warmup).toHaveCount(0);
  await expect(updated).toHaveCount(0);
  await waitForHydration();
  await initialClientModule;
  await waitForNuxtUiHmrReady(page, devServer, startupLogStart, async () => {
    await expect(original).toBeVisible();
    await waitForHydration();
  });

  const hmrRequests: string[] = [];
  const completedHmrUpdates: string[] = [];
  page.on("request", (request) => {
    const url = new URL(request.url());
    if (
      url.pathname.endsWith(PROBE_MODULE_PATH) &&
      url.searchParams.has("t") &&
      url.searchParams.has("vize")
    )
      hmrRequests.push(url.href);
  });
  page.on("console", (message) => {
    if (PROBE_HOT_UPDATED.test(message.text())) completedHmrUpdates.push(message.text());
  });
  // A killed worker skips `finally`, so keep a process-level source restore too.
  const restoreGuard = installSourceRestore(sourcePath, originalSource);
  try {
    await absorbNuxtTemplateRegeneration({
      devServer,
      sourcePath,
      originalSource,
      warmupSource,
      original,
      warmup,
      waitForHydration,
    });
  } catch (error) {
    fs.writeFileSync(sourcePath, originalSource);
    restoreGuard.markRestored();
    restoreGuard.detach();
    throw error;
  }

  const probe = `hmr-${Date.now()}`;
  const updateLogStart = getProcessLogs(devServer).length;
  let forwardCompleted = false;
  try {
    await page.evaluate((value) => {
      (window as Window & { __vizeNuxtUiHmrProbe?: string }).__vizeNuxtUiHmrProbe = value;
    }, probe);

    const requestStart = hmrRequests.length;
    const completionStart = completedHmrUpdates.length;
    fs.writeFileSync(sourcePath, updatedSource);
    await expect(updated).toBeVisible({ timeout: 60_000 });
    await expect(original).toBeVisible();
    expect(hmrRequests.length).toBeGreaterThan(requestStart);
    await expect.poll(() => completedHmrUpdates.length).toBeGreaterThan(completionStart);
    await page.waitForTimeout(2_000);
    expect(
      await page.evaluate(
        () => (window as Window & { __vizeNuxtUiHmrProbe?: string }).__vizeNuxtUiHmrProbe,
      ),
    ).toBe(probe);
    forwardCompleted = true;
  } finally {
    const requestStart = hmrRequests.length;
    const completionStart = completedHmrUpdates.length;
    // If this synchronous restore fails, the attached exit handler gets one
    // final best-effort attempt because the following detach is not reached.
    fs.writeFileSync(sourcePath, originalSource);
    restoreGuard.markRestored();
    restoreGuard.detach();
    if (forwardCompleted) {
      await expect(original).toBeVisible({ timeout: 60_000 });
      await expect(updated).toHaveCount(0, { timeout: 60_000 });
      expect(hmrRequests.length).toBeGreaterThan(requestStart);
      await expect.poll(() => completedHmrUpdates.length).toBeGreaterThan(completionStart);
      await page.waitForTimeout(2_000);
    }
  }

  expect(
    await page.evaluate(
      () => (window as Window & { __vizeNuxtUiHmrProbe?: string }).__vizeNuxtUiHmrProbe,
    ),
  ).toBe(probe);
  expect(fs.readFileSync(sourcePath, "utf8")).toBe(originalSource);
  const updateLogs = getProcessLogs(devServer).slice(updateLogStart).join("\n");
  expect(updateLogs).toMatch(PROBE_HMR_UPDATE);
  expect(updateLogs).not.toMatch(/page reload/i);
  expect(consoleErrors.filter(isFatalError)).toHaveLength(0);
  expect(hydrationErrors).toHaveLength(0);
}
