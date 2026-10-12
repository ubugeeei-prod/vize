import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFile, realpath, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import test from "node:test";
import { chromium, type Page } from "playwright";
import { build, preview } from "vite";
import {
  fixture,
  previews,
  repository,
  unusedPort,
  variants,
} from "./literal-variant-browser.fixture.ts";
import { digest, inventory, mutantSite } from "./variant-navigation-browser.mutant.ts";
import { push } from "./variant-navigation-browser.routes.ts";
import {
  geometry,
  instrumentObserver,
  observerCount,
  observerEvidence,
  replayFormerNativeDelivery,
  settle,
  waitForNewNativeDelivery,
} from "./variant-navigation-browser.observer.ts";

async function assertTarget(page: Page, name: string, observations: unknown[], phase: string) {
  const state = await geometry(page, name);
  observations.push({ phase, ...state });
  assert.equal(state.aria, "true", JSON.stringify(state));
  assert.deepEqual(state.selected, [name]);
  assert.ok(
    Object.values(variants).some(
      (group) =>
        JSON.stringify(group.map(([name]) => name)) ===
        JSON.stringify(state.variantVector.map((row) => row.name)),
    ),
  );
  for (const row of state.variantVector) {
    assert.equal(row.sectionName, row.name);
    assert.equal(row.id, row.ariaControls);
    assert.ok(row.rect);
    assert.equal(row.aria, row.name === name ? "true" : null);
  }
  assert.ok(state.rect.top < state.viewport.height && state.rect.bottom > 0, JSON.stringify(state));
  const scroller = state.ancestors.find((row) => ["auto", "scroll"].includes(row.overflow));
  assert.ok(scroller, "actual scrolling ancestor must be captured");
  assert.ok(
    state.rect.bottom > scroller.top && state.rect.top < scroller.top + scroller.clientHeight,
  );
}

async function rapid(
  page: Page,
  first: string,
  last: string,
  observations: unknown[],
  phase: string,
) {
  const count = await observerCount(page);
  const items = page.locator(".variant-toc-item");
  await items.last().click();
  await items.first().click();
  await waitForNewNativeDelivery(page, count);
  await assertTarget(page, first, observations, `${phase}:last-first`);
  const next = await observerCount(page);
  await items.first().click();
  await items.last().click();
  await waitForNewNativeDelivery(page, next);
  await assertTarget(page, last, observations, `${phase}:first-last`);
}

void test(
  "actual native gallery keeps physical variant navigation and observer ownership",
  {
    skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1",
    timeout: 120_000,
  },
  async () => {
    const f = await fixture("navigation-owner");
    const browser = await chromium.launch();
    const page = await browser.newPage();
    await instrumentObserver(page);
    const errors: string[] = [];
    const observations: unknown[] = [];
    const receipts: unknown[] = [];
    page.on("pageerror", (error) => errors.push(String(error)));
    page.on("console", (message) => {
      if (message.type() === "error") errors.push(message.text());
    });
    const outDir = path.join(f.root, "dist");
    const servers: Array<Awaited<ReturnType<typeof preview>>> = [];
    async function serve(directory: string) {
      const server = await preview({
        root: f.root,
        configFile: false,
        build: { outDir: directory },
        preview: { host: "127.0.0.1", port: await unusedPort(), strictPort: true },
      });
      servers.push(server);
      return new URL("/__musea__/", server.resolvedUrls!.local[0]).href;
    }
    try {
      await build({ ...f.config, build: { outDir, emptyOutDir: true } });
      const loaded = Object.keys(createRequire(import.meta.url).cache).filter(
        (file) => file.endsWith(".node") && file.startsWith(path.join(repository, "npm/native/")),
      );
      assert.equal(
        loaded.length,
        1,
        `exact source native provider required: ${JSON.stringify(loaded)}`,
      );
      receipts.push({
        phase: "provider",
        source: execFileSync("git", ["--no-replace-objects", "rev-parse", "HEAD"], {
          cwd: repository,
          encoding: "utf8",
        }).trim(),
        native: await realpath(loaded[0]),
        nativeSha256: digest(await readFile(loaded[0])),
        instrumentationSha256: digest(
          await readFile(
            new URL("./variant-navigation-browser.instrumentation.js", import.meta.url),
          ),
        ),
        navigationProducerSha256: digest(
          await readFile(
            path.join(repository, "npm/builder/vite-musea/gallery/utils/variantNavigation.ts"),
          ),
        ),
        sources: f.sources,
        galleryInputs: await inventory(path.join(repository, "npm/builder/vite-musea/gallery")),
        wholeBuiltSite: await inventory(outDir),
      });
      const url = await serve(outDir);
      await page.goto(url);
      for (const title of ["Controls", "Keys"] as const) {
        const count = await observerCount(page);
        observations.push({ phase: `${title}:all-previews`, frames: await previews(page, title) });
        await waitForNewNativeDelivery(page, count);
        const names = variants[title].map(([name]) => name);
        await rapid(page, names[0], names.at(-1)!, observations, title);
        const hashCount = await observerCount(page);
        const target = await geometry(page, names[0]);
        await push(page, { hash: `#${target.id}` });
        await waitForNewNativeDelivery(page, hashCount);
        await assertTarget(page, names[0], observations, `${title}:hash`);
        const keyboardCount = await observerCount(page);
        await page.locator(".variant-toc-item").last().focus();
        await page.keyboard.press("Enter");
        await waitForNewNativeDelivery(page, keyboardCount);
        await assertTarget(page, names.at(-1)!, observations, `${title}:keyboard`);
      }
      // Keep the original Chromium viewport; real recorded post-scroll entries already
      // intersect its unchanged IO band (y129..346 at1280x720).
      const firstCount = await observerCount(page);
      await page.locator(".variant-toc-item").first().click();
      await waitForNewNativeDelivery(page, firstCount);
      const manualCount = await observerCount(page);
      await page
        .locator(".variant-section")
        .last()
        .evaluate((section) => section.scrollIntoView({ behavior: "instant", block: "start" }));
      await page.waitForFunction(
        () =>
          document.querySelector(".variant-toc-item:last-child")?.getAttribute("aria-current") ===
          "true",
        undefined,
        { timeout: 8_000 },
      );
      await settle(page);
      assert.equal(
        await observerCount(page),
        manualCount,
        "manual scroll must use the existing observer",
      );
      await assertTarget(page, "hasOwnProperty", observations, "manual-scroll");
      const retainedFirstCount = await observerCount(page);
      await page.locator(".variant-toc-item").first().click();
      await waitForNewNativeDelivery(page, retainedFirstCount);
      await assertTarget(page, "__proto__", observations, "retained:before");
      const retainedCount = await observerCount(page);
      await page.locator(".variant-toc-item").last().click();
      await waitForNewNativeDelivery(page, retainedCount);
      observations.push(await replayFormerNativeDelivery(page, "__proto__"));
      await settle(page);
      await assertTarget(page, "hasOwnProperty", observations, "retained:refused");

      const tabCount = await observerCount(page);
      await page.getByRole("button", { name: "Props", exact: true }).click();
      await settle(page);
      const tabEvidence = await observerEvidence(page);
      assert.ok(tabEvidence.records.slice(0, tabCount).every((record) => record.disconnected));
      observations.push(await replayFormerNativeDelivery(page, "__proto__"));
      await settle(page);
      assert.equal(await page.locator(".variant-section").count(), 0);
      assert.equal(
        await page.getByRole("button", { name: "Props", exact: true }).getAttribute("aria-pressed"),
        "true",
      );
      observations.push({
        phase: "tab:former-callback",
        html: await page.content(),
        evidence: await observerEvidence(page),
      });
      await page.getByRole("button", { name: "Variants", exact: true }).click();
      await waitForNewNativeDelivery(page, tabCount);
      await assertTarget(page, "__proto__", observations, "tab:hash-restored");
      await push(page, { hash: "#variant-constructor" }, { tab: "Props" });
      await settle(page);
      assert.equal(await page.locator(".variant-section").count(), 0);
      observations.push({
        phase: "rapid-hash-tab",
        html: await page.content(),
        evidence: await observerEvidence(page),
      });
      const restoreCount = await observerCount(page);
      await page.getByRole("button", { name: "Variants", exact: true }).click();
      await waitForNewNativeDelivery(page, restoreCount);
      await assertTarget(page, "constructor", observations, "rapid-hash-tab:restored");
      const routeCount = await observerCount(page);
      await previews(page, "Controls");
      await waitForNewNativeDelivery(page, routeCount);
      await assertTarget(page, "Default", observations, "route:before-former-callback");
      observations.push(await replayFormerNativeDelivery(page, "__proto__"));
      await settle(page);
      await assertTarget(page, "Default", observations, "route:former-callback-refused");
      await push(page, { hash: "#variant-custom-theme" }, { replace: { name: "home" } });
      await settle(page);
      assert.equal(await page.locator(".component-view").count(), 0);
      observations.push(await replayFormerNativeDelivery(page, "__proto__"));
      const disposed = await observerEvidence(page);
      assert.ok(disposed.records.every((record) => record.disconnected));
      observations.push({ phase: "unmount", evidence: disposed, html: await page.content() });

      // Genuine one-change compiled producers must fail the additive observation for its reason.
      for (const kind of ["callback", "smooth"] as const) {
        const mutant = await mutantSite(f.root, outDir, kind);
        receipts.push(mutant.receipt);
        const mutantUrl = await serve(mutant.site);
        await page.goto(mutantUrl);
        await previews(page, "Keys");
        const initialCount = await observerCount(page);
        await page.locator(".variant-toc-item").first().click();
        await waitForNewNativeDelivery(page, initialCount);
        await assertTarget(page, "__proto__", observations, `${kind}:initial`);
        if (kind === "callback") {
          const count = await observerCount(page);
          await page.locator(".variant-toc-item").last().click();
          await waitForNewNativeDelivery(page, count);
          await assertTarget(page, "hasOwnProperty", observations, "callback:latest-target");
          observations.push(await replayFormerNativeDelivery(page, "__proto__"));
          await settle(page);
          const reversed = await geometry(page, "hasOwnProperty");
          observations.push({
            phase: "callback:counterexample",
            ...reversed,
            evidence: await observerEvidence(page),
          });
          assert.equal(reversed.aria, null);
          assert.deepEqual(reversed.selected, ["__proto__"]);
          assert.ok(reversed.rect.top < reversed.viewport.height && reversed.rect.bottom > 0);
        } else {
          // Start at a real distant last target; record actual newly attached IO delivery during return.
          await page
            .locator(".variant-section")
            .last()
            .evaluate((section) => section.scrollIntoView({ behavior: "instant", block: "start" }));
          await settle(page);
          const count = await observerCount(page);
          await page.locator(".variant-toc-item").first().click();
          await waitForNewNativeDelivery(page, count);
          const moving = await geometry(page, "__proto__");
          observations.push({
            phase: "smooth:counterexample",
            ...moving,
            evidence: await observerEvidence(page),
            failedClauses: {
              physicalTarget: moving.rect.bottom <= 0,
              selectedTarget: moving.aria !== "true",
            },
          });
          assert.ok(moving.rect.bottom <= 0 || moving.aria !== "true", JSON.stringify(moving));
        }
      }
      assert.deepEqual(errors, []);
    } finally {
      try {
        observations.push({
          phase: "final",
          html: await page.content(),
          evidence: await observerEvidence(page),
        });
        await page.screenshot({ path: path.join(f.output, "final.png"), fullPage: true });
      } catch (error) {
        observations.push({ captureError: String(error) });
      }
      await writeFile(
        path.join(f.output, "observations.json"),
        JSON.stringify({ observations, receipts, errors }, null, 2),
      );
      await browser.close();
      for (const server of servers)
        await new Promise<void>((resolve, reject) =>
          server.httpServer.close((error) => (error ? reject(error) : resolve())),
        );
      await f.retain();
    }
  },
);
