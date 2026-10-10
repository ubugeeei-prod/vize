import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import type { Page } from "playwright";
import { PNG } from "pngjs";
import { parseArgs } from "./cli/index.ts";
import { startHostedVrtSession } from "./cli/serve.ts";
import { buildServiceGallery, repository, sha256 } from "./hosted-vrt-build.fixtures.ts";
import {
  createSecureHost,
  launchPublicBrowser,
  setLoopbackPermission,
  trustSecureHost,
  createAddressSpaceObserver,
} from "./hosted-vrt-network.fixtures.ts";
import {
  createLifecycleRelay,
  observeObjectUrls,
  nextArtifactSettlement,
  renderFrames,
} from "./hosted-vrt-lifecycle.fixtures.ts";

interface Capture {
  success: boolean;
  results: Array<{ images: Record<string, string> }>;
  summary: { total: number; errors: number; new: number };
}

async function submitConnection(page: Page, endpoint: string, token: string) {
  await page.getByRole("textbox", { name: "VRT endpoint", exact: true }).fill(endpoint);
  await page.getByLabel("Session token", { exact: true }).fill(token);
  await page.getByRole("button", { name: "Connect VRT", exact: true }).click();
}

void test(
  "hosted VRT keeps latest connection authority and releases images after actual panel unmount",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1", timeout: 180000 },
  async (t) => {
    const root = await mkdtemp(path.join(os.tmpdir(), "musea-native-hosted-lifecycle-"));
    const output = path.join(repository, "artifacts/musea-native-hosted-vrt-lifecycle");
    await mkdir(output, { recursive: true });
    const built = await buildServiceGallery(root, output);
    const host = await createSecureHost(built.directory);
    const restoreTrust = await trustSecureHost(host);
    const options = parseArgs([
      "serve",
      "--gallery-url",
      `${host.origin}/built/gallery/`,
      "--output",
      path.join(root, "local-vrt"),
    ]);
    options.vrt = {
      viewports: [{ name: "compact", width: 320, height: 180 }],
      capture: { settleTime: 0 },
    };
    const session = await startHostedVrtSession(options, host.spki);
    const relay = await createLifecycleRelay(session.endpoint);
    assert.notEqual(relay.endpoint, session.endpoint);
    const browser = await launchPublicBrowser(host);
    const observations: unknown[] = [];
    t.after(async () => {
      await browser.close();
      await relay.close();
      await session.close();
      restoreTrust();
      await host.close();
      await rm(root, { recursive: true, force: true });
    });

    async function openGallery() {
      const context = await browser.newContext();
      const page = await context.newPage();
      const objectUrls = await observeObjectUrls(page);
      const spaces = await createAddressSpaceObserver(page);
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(String(error)));
      await page.goto(`${host.origin}/built/gallery/`);
      await page.locator(".art-item").filter({ hasText: "Left" }).click();
      await page
        .frameLocator(".variant-card iframe")
        .getByRole("button", { name: "Left", exact: true })
        .waitFor();
      assert.equal(
        await page.frameLocator(".variant-card iframe").locator("museacomponent").count(),
        0,
      );
      await page.getByRole("button", { name: "VRT", exact: true }).click();
      const clearPermission = await setLoopbackPermission(page, "granted");
      return {
        page,
        objectUrls,
        errors,
        spaces,
        async close() {
          await objectUrls.restore();
          await clearPermission();
          await spaces.close();
          await context.close();
        },
      };
    }

    try {
      await t.test(
        "a delayed original 401 cannot invalidate the newer real 200 session",
        async () => {
          const gallery = await openGallery();
          const { page } = gallery;
          const hold = relay.holdNext({ pathname: "/session", status: 401 });
          const wrongToken = `${session.token[0] === "A" ? "B" : "A"}${session.token.slice(1)}`;
          const receipt: Record<string, unknown> = { phase: "latest-connection" };
          observations.push(receipt);
          try {
            await submitConnection(page, relay.endpoint, wrongToken);
            const original = await hold.buffered;
            assert.equal(original.receipt.status, 401);
            receipt.heldOriginal401 = original.receipt;
            await writeFile(path.join(output, "original-delayed-401.body"), original.body);
            const directResponse = page.waitForResponse(
              (response) =>
                response.url() === `${session.endpoint}/session` &&
                response.request().method() === "GET",
            );
            await submitConnection(page, session.endpoint, session.token);
            const direct = await directResponse;
            const directBytes = await direct.body();
            assert.equal(direct.status(), 200);
            assert.equal(
              JSON.parse(directBytes.toString()).galleryUrl,
              `${host.origin}/built/gallery/`,
            );
            receipt.newSession = {
              status: direct.status(),
              bytes: directBytes.length,
              sha256: sha256(directBytes),
            };
            await page.getByText("Connected to your local VRT session.", { exact: true }).waitFor();
            const lateResponse = page.waitForResponse(
              (response) =>
                response.url() === `${relay.endpoint}/session` && response.status() === 401,
            );
            hold.release();
            await hold.released;
            assert.equal((await lateResponse).status(), original.receipt.status);
            await renderFrames(page);
            const [capture] = await Promise.all([
              page.waitForResponse(
                (response) =>
                  new URL(response.url()).pathname === "/capture" &&
                  response.request().method() === "POST",
              ),
              page.getByRole("button", { name: "Run VRT", exact: true }).click(),
            ]);
            const captureBytes = await capture.body();
            receipt.postReleaseCapture = {
              origin: new URL(capture.url()).origin,
              pathname: new URL(capture.url()).pathname,
              status: capture.status(),
              bytes: captureBytes.length,
              sha256: sha256(captureBytes),
            };
            await writeFile(path.join(output, "post-delayed-401-capture.body"), captureBytes);
            assert.equal(capture.url(), `${session.endpoint}/capture`);
            assert.equal(capture.status(), 200);
            assert.equal((JSON.parse(captureBytes.toString()) as Capture).success, true);
            assert.equal(await page.locator(".hosted-vrt-connect [role=alert]").count(), 0);
            await page.waitForFunction(() => {
              const images = [...document.querySelectorAll<HTMLImageElement>(".vrt-results img")];
              return (
                images.length > 0 &&
                images.every((image) => image.complete && image.naturalWidth === 320)
              );
            });
            assert.deepEqual(gallery.errors, []);
          } finally {
            hold.release();
            receipt.addressSpaces = gallery.spaces.records;
            receipt.errors = gallery.errors;
            await gallery.close();
          }
        },
      );

      await t.test(
        "releasing an actual PNG after Variants unmount leaves no live object URL",
        async () => {
          const gallery = await openGallery();
          const { page } = gallery;
          const hold = relay.holdNext({
            pathname: /^\/artifacts\//,
            status: 200,
            contentType: "image/png",
          });
          const receipt: Record<string, unknown> = { phase: "unmount-with-pending-png" };
          observations.push(receipt);
          try {
            await submitConnection(page, relay.endpoint, session.token);
            await page.getByText("Connected to your local VRT session.", { exact: true }).waitFor();
            const settled = nextArtifactSettlement(page, relay.endpoint);
            const [capture] = await Promise.all([
              page.waitForResponse(
                (response) =>
                  response.url() === `${relay.endpoint}/capture` &&
                  response.request().method() === "POST",
              ),
              page.getByRole("button", { name: "Run VRT", exact: true }).click(),
            ]);
            assert.equal(capture.status(), 200);
            const captureBytes = await capture.body();
            const data = JSON.parse(captureBytes.toString()) as Capture;
            assert.equal(data.success, true);
            assert.equal(data.summary.total, 1);
            assert.equal(data.summary.errors, 0);
            receipt.realCapture = {
              status: capture.status(),
              bytes: captureBytes.length,
              sha256: sha256(captureBytes),
              data,
            };
            const held = await hold.buffered;
            assert.ok(Object.values(data.results[0].images).includes(held.receipt.pathname));
            const png = PNG.sync.read(held.body);
            assert.deepEqual([png.width, png.height], [320, 180]);
            const actual = await fetch(`${session.endpoint}${held.receipt.pathname}`, {
              headers: { Origin: host.origin, Authorization: `Bearer ${session.token}` },
            });
            assert.equal(actual.status, 200);
            assert.equal(sha256(Buffer.from(await actual.arrayBuffer())), held.receipt.sha256);
            await writeFile(path.join(output, "original-pending-artifact.png"), held.body);
            receipt.heldOriginalPng = held.receipt;
            receipt.beforeUnmount = await gallery.objectUrls.counts();
            assert.equal((await gallery.objectUrls.counts()).live, 0);
            await page.getByRole("button", { name: "Variants", exact: true }).click();
            await page.locator(".vrt-panel").waitFor({ state: "detached" });
            await gallery.objectUrls.unmounted();
            receipt.afterUnmount = await gallery.objectUrls.counts();
            hold.release();
            await hold.released;
            receipt.actualRequestSettlement = await settled;
            await renderFrames(page);
            const counts = await gallery.objectUrls.counts();
            receipt.afterRelease = counts;
            await page.screenshot({
              path: path.join(output, "variants-after-release.png"),
              fullPage: true,
            });
            assert.equal(await page.locator(".vrt-panel").count(), 0);
            assert.equal(counts.live, 0, "An unmounted panel retained a late image object URL");
            assert.deepEqual(gallery.errors, []);
          } finally {
            hold.release();
            receipt.addressSpaces = gallery.spaces.records;
            receipt.errors = gallery.errors;
            await gallery.close();
          }
        },
      );
    } finally {
      const receipt = JSON.stringify(
        {
          chromium: browser.version(),
          inputSha256: built.inputSha256,
          observations,
          relayResponses: relay.receipts,
          hostRequests: host.requests,
        },
        null,
        2,
      );
      assert.equal(receipt.includes(session.token), false);
      await writeFile(path.join(output, "observations.json"), receipt);
      await writeFile(
        path.join(output, "native-gallery-trust.pem"),
        await readFile(host.certificatePath),
      );
    }
  },
);
