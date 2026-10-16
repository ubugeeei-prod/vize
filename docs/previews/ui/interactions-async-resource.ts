/** Real abortable producers, retained data and explicit execution outcomes. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

async function activate(page: Page, name: string): Promise<void> {
  await page.getByRole("button", { name, exact: true }).focus();
  await page.keyboard.press("Enter");
}

export async function exerciseAsyncResourceExample(page: Page): Promise<string[]> {
  const quote = page.getByRole("region", { name: "Current quote" });
  const output = page.locator("output");
  await activate(page, "Standard");
  assert.equal(await output.textContent(), "Getting a quote…");
  await activate(page, "Cancel request");
  await page.clock.runFor(1000);
  await page.getByText("0 responses · 1 cancelled request", { exact: true }).waitFor();
  assert.equal(await output.textContent(), "Quote request cancelled.");
  assert.equal(await page.getByRole("button", { name: "Cancel request" }).isDisabled(), true);
  await page.getByText("Last execution result: cancelled", { exact: true }).waitFor();
  assert.equal(await quote.count(), 0);

  await activate(page, "Standard");
  await activate(page, "Express");
  await page.getByText("Last execution result: superseded", { exact: true }).waitFor();
  await page.clock.runFor(300);
  await quote.getByText("Express delivery · €9.90", { exact: true }).waitFor();
  await page.clock.runFor(800);
  await page.getByText("1 response · 2 cancelled requests", { exact: true }).waitFor();
  assert.equal(await quote.getByText("Standard delivery", { exact: false }).count(), 0);

  await activate(page, "Unavailable area");
  assert.equal(await output.textContent(), "Getting a quote…");
  await quote.getByText("Express delivery · €9.90", { exact: true }).waitFor();
  await page.clock.runFor(300);
  assert.equal(await page.getByRole("alert").textContent(), "No courier serves this sample area.");
  await page.getByText("Last execution result: error", { exact: true }).waitFor();
  await quote.getByText("Express delivery · €9.90", { exact: true }).waitFor();

  await activate(page, "Standard");
  assert.equal(await page.getByRole("alert").count(), 0);
  await page.clock.runFor(800);
  await quote.getByText("Standard delivery · €4.90", { exact: true }).waitFor();
  await page.getByText("Last execution result: success", { exact: true }).waitFor();
  assert.equal(
    await page
      .getByRole("button", { name: "Standard" })
      .evaluate((button) => button === document.activeElement),
    true,
  );

  await activate(page, "Express");
  await activate(page, "Reset quote");
  await page.clock.runFor(1000);
  assert.equal(await output.textContent(), "Choose a delivery service.");
  assert.equal(await quote.count(), 0);
  await page.getByText("3 responses · 3 cancelled requests", { exact: true }).waitFor();
  assert.equal(
    await page
      .getByRole("button", { name: "Reset quote" })
      .evaluate((button) => button === document.activeElement),
    true,
  );
  await activate(page, "Express");
  await page.clock.runFor(300);
  await quote.getByText("Express delivery · €9.90", { exact: true }).waitFor();
  return [
    "native AbortSignal cancellation clears the producer timer without a late response",
    "replacement aborts the slow loader and exposes superseded before the newest result wins",
    "pending and failure retain the last successful quote; recovery clears the error",
    "execution outcomes resolve as cancelled/superseded/error/success without unhandled rejection",
    "reset aborts pending work, restores idle data and preserves keyboard focus",
  ];
}

/** Unmount the actual hydrated Vue app after evidence capture, then run past its pending work. */
export async function verifyAsyncResourceUnmount(page: Page): Promise<string[]> {
  await activate(page, "Standard");
  assert.equal(await page.locator("output").textContent(), "Getting a quote…");
  await page.locator("[data-hydration-root]").evaluate((root) => {
    const app = (root as HTMLElement & { __vue_app__?: { unmount(): void } }).__vue_app__;
    if (app == null) throw new Error("Expected the real hydrated Vue application");
    app.unmount();
  });
  await page.clock.runFor(2000);
  assert.equal(await page.locator("[data-hydration-root]").textContent(), "");
  return [
    "actual hydrated Vue app unmounted while pending",
    "no late DOM after the producer deadline",
  ];
}
