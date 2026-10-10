import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import { buildHostedGallery } from "./hosted-tests.browser-fixtures.ts";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const output = path.join(repository, "artifacts/musea-hosted");

await test(
  "built hosted gallery runs real audits after async setup and rejects audit errors",
  {
    skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1",
  },
  async () => {
    await mkdir(output, { recursive: true });
    const host = await buildHostedGallery(output);
    const browser = await chromium.launch();
    const observations: unknown[] = [];
    const errors: string[] = [];
    try {
      const page = await browser.newPage();
      page.on("pageerror", (error) => errors.push(String(error)));
      await page.goto(`${host.url}/tests`);
      await page.getByRole("button", { name: "Run All A11y Tests", exact: true }).waitFor();
      for (let iteration = 0; iteration < 2; iteration++) {
        await page.getByRole("button", { name: "Run All A11y Tests", exact: true }).click();
        await page
          .getByRole("button", { name: "Run All A11y Tests", exact: true })
          .waitFor({ timeout: 15000 });
        const counts = await page.locator(".summary-stats .stat-value").allTextContents();
        assert.deepEqual(counts.slice(0, 4), ["6", "3", "3", "0"]);
        assert.equal(
          await page
            .locator(".test-item.failed .violation-tag")
            .filter({ hasText: "button-name" })
            .count(),
          3,
        );
        assert.equal(await page.locator(".test-item.passed .count.passes").count(), 3);
        observations.push({
          iteration,
          counts,
          tests: await page.locator(".test-list").innerText(),
        });
      }
      await page.screenshot({ path: path.join(output, "completed-audits.png"), fullPage: true });
      // Each preview demonstrably mounts after its HTML load, the old dispatch boundary.
      await page.locator(".test-item").first().click();
      const frame = page.frameLocator(".variant-card iframe").first();
      await frame.getByRole("button", { name: "Accessible action" }).waitFor();
      assert.equal(
        await frame.locator("body").evaluate(() => Reflect.get(window, "__hostedSetupAfterLoad")),
        true,
      );
      await page.locator(".tab-btn").filter({ hasText: "A11y" }).click();
      await page.getByRole("button", { name: /^Run (Test|Again)$/ }).click();
      await page.locator(".a11y-panel .a11y-success").waitFor({ timeout: 10000 });
      await page.getByRole("button", { name: "VRT", exact: true }).click();
      await page.getByText("Capture this hosted gallery with Node and Playwright:").waitFor();
      assert.ok((await page.locator(".vrt-command").innerText()).includes(`${host.url}/`));
      assert.equal(await page.getByRole("button", { name: "Run VRT", exact: true }).count(), 0);
      await page.goto(`${host.url}/tests`);
      host.refuseAxe();
      await page.getByRole("button", { name: "Run All A11y Tests", exact: true }).click();
      await page
        .getByRole("button", { name: "Run All A11y Tests", exact: true })
        .waitFor({ timeout: 15000 });
      const failedCounts = await page.locator(".summary-stats .stat-value").allTextContents();
      assert.deepEqual(failedCounts.slice(0, 4), ["6", "0", "6", "0"]);
      assert.equal(await page.locator(".test-item.failed .test-error").count(), 6);
      observations.push({
        failedCounts,
        tests: await page.locator(".test-list").innerText(),
        requests: host.requests,
      });
      assert.ok(host.requests.some((url) => url.endsWith("/vendor/axe-core.min.js")));
      assert.ok(
        host.requests.every((url) => !url.includes("preview-module") && !url.includes("@vite")),
      );
      assert.deepEqual(errors, []);
      await page.screenshot({ path: path.join(output, "failed-audits.png"), fullPage: true });
    } finally {
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify({ observations, errors }, null, 2),
      );
      await browser.close();
      await host.close();
    }
  },
);
