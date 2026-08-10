import { expect, type Page } from "@playwright/test";
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
  const originalText = 'data-slot="base"';
  const updatedText = `${originalText}\n      data-vize-hmr-probe="updated"`;
  const sourcePath = path.join(cwd, "src/runtime/components/Button.vue");
  const originalSource = fs.readFileSync(sourcePath, "utf8");
  expect(originalSource.split(originalText)).toHaveLength(2);
  const updatedSource = originalSource.replace(originalText, updatedText);
  expect(updatedSource).toContain('data-vize-hmr-probe="updated"');

  const consoleErrors = await collectConsoleErrors(page, appName);
  const hydrationErrors = await collectHydrationErrors(page);
  const initialClientModule = page.waitForResponse(
    (response) => {
      const url = new URL(response.url());
      return (
        response.ok() &&
        url.pathname.endsWith("/src/runtime/components/Button.vue.ts") &&
        url.searchParams.has("vize") &&
        !url.searchParams.has("vize-ssr")
      );
    },
    { timeout: 180_000 },
  );
  await goto();
  const mount = page.locator(mountSelector);
  const original = mount.getByRole("button", { name: "Button" }).last();
  const updated = mount.locator('[data-vize-hmr-probe="updated"]').first();
  await expect(original).toBeVisible();
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
      url.pathname.endsWith("/src/runtime/components/Button.vue.ts") &&
      url.searchParams.has("t") &&
      url.searchParams.has("vize")
    )
      hmrRequests.push(url.href);
  });
  page.on("console", (message) => {
    if (
      /\[vite\] hot updated: .*\/src\/runtime\/components\/Button\.vue\.ts\?vue&vize/.test(
        message.text(),
      )
    )
      completedHmrUpdates.push(message.text());
  });
  const probe = `hmr-${Date.now()}`;
  const updateLogStart = getProcessLogs(devServer).length;
  let forwardCompleted = false;
  // A killed worker skips `finally`, so keep a process-level source restore too.
  const restoreGuard = installSourceRestore(sourcePath, originalSource);
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
  expect(updateLogs).toMatch(/hmr update .*\/src\/runtime\/components\/Button\.vue\.ts\?vue&vize/);
  expect(updateLogs).not.toMatch(/page reload/i);
  expect(consoleErrors.filter(isFatalError)).toHaveLength(0);
  expect(hydrationErrors).toHaveLength(0);
}
