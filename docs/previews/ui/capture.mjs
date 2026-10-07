import assert from "node:assert/strict";
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

import { featuredExamples } from "../../../npm/ui/scripts/reference-docs/examples.ts";
import { previewExamples } from "./build-config.ts";

/** Browser evidence is generated from the same SFC source shown in the docs. */
export async function captureUiPreviews(browser, baseUrl, outputRoot) {
  mkdirSync(outputRoot, { recursive: true });
  const page = await browser.newPage({
    viewport: { width: 720, height: 440 },
    deviceScaleFactor: 1,
    reducedMotion: "reduce",
  });
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error" || message.type() === "warning") errors.push(message.text());
  });
  const receipts = [];
  try {
    for (const family of featuredExamples) {
      await page.goto(`${baseUrl}index.html?family=${family}`, { waitUntil: "networkidle" });
      await page.waitForSelector(`html[data-preview-ready="${family}"]`);
      const sourceSha256 = await page.locator("html").getAttribute("data-preview-source-sha256");
      assert.equal(
        sourceSha256,
        previewExamples.find(({ entry }) => entry.canonicalName === family).sourceSha256,
      );
      if (family === "button") {
        await page.getByRole("button", { name: "Save draft" }).click();
        assert.equal(await page.locator("output").textContent(), "Saved 1 times");
        assert.equal(await page.getByRole("button", { name: "Publish" }).isDisabled(), true);
      } else if (family === "input") {
        await page.getByRole("textbox", { name: "Display name" }).fill("Ada Lovelace");
        assert.equal(await page.locator("output").textContent(), "Hello, Ada Lovelace");
      } else if (family === "checkbox") {
        await page.getByRole("checkbox", { name: "I agree to the terms of service" }).check();
        assert.equal(await page.getByRole("button", { name: "Create account" }).isEnabled(), true);
      } else if (family === "switch") {
        await page.getByRole("switch", { name: "Email notifications" }).click();
        assert.equal(await page.locator("output").textContent(), "on");
      } else if (family === "tabs") {
        await page.getByRole("tab", { name: "Overview" }).focus();
        await page.keyboard.press("ArrowRight");
        await page.keyboard.press("Enter");
        assert.equal(
          await page.getByRole("tab", { name: "Specifications" }).getAttribute("aria-selected"),
          "true",
        );
        assert.equal(
          await page.getByRole("tabpanel").textContent(),
          "Water resistant, 320 g, machine washable.",
        );
      } else if (family === "dialog") {
        await page.getByRole("button", { name: "Edit profile" }).click();
        await page.getByRole("dialog", { name: "Edit profile" }).waitFor();
        await page.getByRole("textbox", { name: "Display name" }).fill("Ada Lovelace");
      } else if (family === "alert") {
        await page.getByRole("button", { name: "Dismiss" }).click();
        assert.equal(await page.getByRole("status").count(), 0);
        await page.getByRole("button", { name: "Show message again" }).click();
        await page.getByRole("status").waitFor();
      } else {
        assert.equal(await page.getByRole("article", { name: "Release 2.4" }).count(), 1);
      }
      assert.deepEqual(errors, [], `${family}: browser diagnostics`);
      await page.screenshot({ path: path.join(outputRoot, `${family}.png`), fullPage: true });
      if (family === "dialog") {
        await page.keyboard.press("Escape");
        assert.equal(await page.getByRole("dialog").count(), 0);
        assert.equal(
          await page
            .getByRole("button", { name: "Edit profile" })
            .evaluate((element) => element === document.activeElement),
          true,
        );
      }
      await page.setViewportSize({ width: 360, height: 440 });
      assert.equal(
        await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
        true,
        `${family}: narrow layout`,
      );
      await page.setViewportSize({ width: 720, height: 440 });
      receipts.push({
        family,
        sourceSha256,
        screenshot: `${family}.png`,
        interactions: "passed",
        narrowViewport: 360,
        browserDiagnostics: [],
      });
    }
    await page.goto(`${baseUrl}index.html?family=button`, { waitUntil: "networkidle" });
    const palette = () =>
      page.getByRole("button", { name: "Save draft" }).evaluate((element) => {
        const style = getComputedStyle(element);
        return [
          style.backgroundColor,
          style.color,
          style.borderRadius,
          style.paddingInlineStart,
          style.fontFamily,
        ];
      });
    const paper = await palette();
    await page.getByRole("combobox", { name: "Style preset" }).selectOption("signal");
    assert.equal(await page.locator("html").getAttribute("data-vize-theme"), "signal");
    assert.notDeepEqual(await palette(), paper, "preset changes the actual component styles");
    assert.deepEqual(errors, []);
    writeFileSync(
      path.join(outputRoot, "browser-evidence.json"),
      `${JSON.stringify({ schema: "vize.ui-docs-browser-evidence", browserVersion: browser.version(), availableBasicExamples: previewExamples.length, examples: receipts, source: "workspace catalog entry sources", screenshots: "actual Playwright Chromium rendering" }, null, 2)}\n`,
    );
    process.stdout.write(
      `Verified and captured ${receipts.length} component examples in Chromium\n`,
    );
  } finally {
    await page.close();
  }
}
