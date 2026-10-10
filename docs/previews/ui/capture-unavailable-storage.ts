/** Verify the writable fallback with Chromium's actual localStorage capability disabled. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { chromium } from "playwright";
import { resolvePuppeteerExecutablePath } from "../../browser-path.js";
import { previewComposableExamples } from "./build-config.ts";

export async function verifyUnavailableStorage(baseUrl: string, outputRoot: string): Promise<void> {
  const example = previewComposableExamples.find((item) => item.name === "use-storage");
  assert.ok(example, "storage fallback must use the same registered source");
  mkdirSync(outputRoot, { recursive: true });
  const args = ["--disable-local-storage"];
  const browser = await chromium.launch({
    executablePath: resolvePuppeteerExecutablePath(),
    args,
  });
  const page = await browser.newPage({ viewport: { width: 360, height: 540 } });
  const diagnostics: string[] = [];
  page.on("pageerror", (error) => diagnostics.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error" || message.type() === "warning")
      diagnostics.push(message.text());
  });
  const screenshots: { state: string; file: string; sha256: string }[] = [];
  const status = "Browser storage is unavailable; changes stay in this tab.";
  try {
    const response = await page.goto(`${baseUrl}index.html?composable=use-storage`, {
      waitUntil: "networkidle",
    });
    assert.equal(response?.status(), 200);
    await page.waitForSelector('html[data-preview-ready="use-storage"]');
    assert.equal(
      await page.locator("html").getAttribute("data-preview-source-sha256"),
      example.sourceSha256,
    );
    assert.equal(
      await page.locator("html").getAttribute("data-preview-hydration-retained"),
      "true",
    );
    assert.equal(
      await page.evaluate(() => {
        const storage: unknown = window.localStorage;
        return storage === null;
      }),
      true,
      "the native browser capability must actually be unavailable",
    );
    await page.getByText(status, { exact: true }).waitFor();
    assert.equal(await page.locator("output").textContent(), "Resume at: Getting started");
    for (const state of ["initial", "interacted"]) {
      if (state === "interacted") {
        await page.getByRole("combobox", { name: "Guide section" }).focus();
        await page.keyboard.press("p");
        await page.keyboard.press("Tab");
        assert.equal(
          await page.getByRole("combobox", { name: "Guide section" }).inputValue(),
          "publishing",
        );
        assert.equal(await page.locator("output").textContent(), "Resume at: Publishing");
        await page.getByText(status, { exact: true }).waitFor();
      }
      assert.equal(
        await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
        true,
        "unavailable-storage presentation must fit the mobile viewport",
      );
      assert.deepEqual(diagnostics, []);
      const file = `use-storage-unavailable-360-${state}.png`;
      await page.screenshot({ path: path.join(outputRoot, file), fullPage: true });
      screenshots.push({
        state,
        file,
        sha256: createHash("sha256")
          .update(readFileSync(path.join(outputRoot, file)))
          .digest("hex"),
      });
    }
    const reloaded = await page.reload({ waitUntil: "networkidle" });
    assert.equal(reloaded?.status(), 200);
    await page.waitForSelector('html[data-preview-ready="use-storage"]');
    assert.equal(
      await page.locator("html").getAttribute("data-preview-hydration-retained"),
      "true",
    );
    assert.equal(await page.locator("output").textContent(), "Resume at: Getting started");
    await page.getByText(status, { exact: true }).waitFor();
    assert.deepEqual(diagnostics, []);
    writeFileSync(
      path.join(outputRoot, "unavailable-storage-evidence.json"),
      `${JSON.stringify(
        {
          schema: "vize.composable-docs-unavailable-storage-evidence",
          browserVersion: browser.version(),
          baseUrl,
          args,
          sourceSha256: example.sourceSha256,
          screenshots,
          interactions: [
            "native localStorage is null",
            "SSR fallback hydrates without replacements",
            "keyboard selection updates in-tab state with accurate unavailable status",
            "reload restores the default because the choice could not persist",
          ],
          browserDiagnostics: diagnostics,
        },
        null,
        2,
      )}\n`,
    );
  } finally {
    await browser.close();
  }
  process.stdout.write("Verified actual unavailable-storage fallback and mobile keyboard state\n");
}
