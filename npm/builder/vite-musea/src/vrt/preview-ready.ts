import type { Page } from "playwright";

/** Attach before navigation, because async setup may finish before goto returns. */
export async function observePreviewReady(page: Page): Promise<void> {
  await page.addInitScript(() => {
    window.addEventListener("message", (event) => {
      if (
        event.source === window &&
        event.origin === window.location.origin &&
        event.data?.type === "musea:ready"
      )
        Reflect.set(window, "__museaVrtReady", true);
    });
  });
}

export async function waitForMountedPreview(page: Page): Promise<void> {
  await page.waitForFunction(() => Reflect.get(window, "__museaVrtReady") === true, undefined, {
    timeout: 10000,
  });
}
