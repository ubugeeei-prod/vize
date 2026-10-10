import assert from "node:assert/strict";
import type { Page } from "playwright";

/** Exercise the actual generated site's tab controls and copyable commands. */
export async function verifyCommandTabs(page: Page, route: string) {
  if (!route.endsWith("/getting-started")) return;
  const initialization = /^\/(?:zh-CN|pt-BR|fr)\//.test(route);
  const browser = page.context().browser();
  assert(browser, "fallback browser exists");
  const fallback = await browser.newContext({ javaScriptEnabled: false });
  try {
    const plain = await fallback.newPage();
    const response = await plain.goto(page.url(), { waitUntil: "domcontentloaded" });
    assert.equal(response?.status(), 200);
    assert.equal(await plain.locator(".vize-command-tabs").count(), 0);
    const original = initialization ? "vpx vize init" : "vp install -D @vizejs/vite-plugin";
    assert.ok(
      (await plain.locator("pre code").allTextContents()).some((text) => text.trim() === original),
      "original copyable command survives without JavaScript",
    );
  } finally {
    await fallback.close();
  }
  const widget = page.locator(".vize-command-tabs").first();
  const tabs = widget.getByRole("tab");
  assert.deepEqual(await tabs.allTextContents(), [
    "vp",
    "npm",
    "pnpm",
    "yarn",
    "bun",
    "aube",
    "jsr",
  ]);
  const expected = initialization
    ? [
        "vpx vize init",
        "npx vize init",
        "pnpm dlx vize init",
        "yarn dlx vize init",
        "bun x vize init",
        "aube dlx vize init",
      ]
    : [
        "vp install -D @vizejs/vite-plugin",
        "npm install -D @vizejs/vite-plugin",
        "pnpm add -D @vizejs/vite-plugin",
        "yarn add -D @vizejs/vite-plugin",
        "bun add -D @vizejs/vite-plugin",
        "aube add -D @vizejs/vite-plugin",
      ];
  for (let index = 0; index < expected.length; index++) {
    await tabs.nth(index).click();
    const panel = widget.getByRole("tabpanel");
    assert.equal(await panel.count(), 1, `${route}: one visible panel`);
    assert.equal((await panel.locator("pre code").textContent())?.trim(), expected[index]);
    assert.equal(await tabs.nth(index).getAttribute("aria-selected"), "true");
  }
  await tabs.first().focus();
  await tabs.first().press("ArrowLeft");
  assert.equal(await tabs.last().getAttribute("aria-selected"), "true", "arrow wraps to JSR");
  assert.equal(
    await widget.getByRole("tabpanel").locator('[data-command-gap="registry"]').count(),
    1,
  );
  assert.equal(
    await widget.getByRole("tabpanel").locator("pre").count(),
    0,
    "no invented JSR command",
  );
  await tabs.last().press("Home");
  assert.equal(await tabs.first().getAttribute("aria-selected"), "true");
  await tabs.first().press("End");
  assert.equal(await tabs.last().getAttribute("aria-selected"), "true");
  await tabs.nth(1).click();
  await page.context().grantPermissions(["clipboard-read", "clipboard-write"]);
  await widget.getByRole("tabpanel").getByRole("button").click();
  await page.waitForFunction(
    (command) => navigator.clipboard.readText().then((value) => value.trim() === command),
    expected[1],
  );
  await page.reload({ waitUntil: "networkidle" });
  assert.equal(
    await page
      .locator(".vize-command-tabs")
      .first()
      .getByRole("tab", { name: "npm", exact: true })
      .getAttribute("aria-selected"),
    "true",
    "preference survives navigation",
  );
}
