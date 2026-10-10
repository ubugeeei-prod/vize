import assert from "node:assert/strict";
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { PNG } from "pngjs";
import type { Page } from "playwright";
import { parseArgs } from "./cli/index.ts";
import { startHostedVrtSession } from "./cli/serve.ts";
import { buildServiceGallery, repository, sha256 } from "./hosted-vrt-build.fixtures.ts";
import { proveOwnedHostedError } from "./hosted-vrt-errors.fixtures.ts";
import { proveCompiledSession } from "./hosted-vrt-cli.fixtures.ts";
import {
  createSecureHost,
  launchPublicBrowser,
  setLoopbackPermission,
  trustSecureHost,
  createAddressSpaceObserver,
} from "./hosted-vrt-network.fixtures.ts";
import {
  refuseSessionInputs,
  refuseHostedNavigation,
  storedCaptureBytes,
  proveSerialCapture,
} from "./hosted-vrt-controls.fixtures.ts";

interface Capture {
  success: boolean;
  results: Array<{
    viewport: string;
    isNew?: boolean;
    passed: boolean;
    diffPercentage?: number;
    images: Record<string, string>;
  }>;
  summary: { total: number; passed: number; failed: number; new: number; errors: number };
  reports: { json: string; html: string };
}

async function capture(page: Page, endpoint: string): Promise<Capture> {
  const [response] = await Promise.all([
    page.waitForResponse(
      (item) => item.url() === `${endpoint}/capture` && item.request().method() === "POST",
    ),
    page.getByRole("button", { name: "Run VRT", exact: true }).click(),
  ]);
  const actual = (await response.json()) as Capture;
  assert.equal(response.status(), 200, JSON.stringify(actual));
  assert.equal(actual.success, true);
  assert.equal(actual.summary.total, 1);
  assert.equal(actual.summary.errors, 0);
  assert.equal(actual.results[0].viewport, "compact");
  await page.waitForFunction(() =>
    [...document.querySelectorAll<HTMLImageElement>(".vrt-results img")].every(
      (image) => image.complete && image.naturalWidth === 320,
    ),
  );
  return actual;
}

void test(
  "public HTTPS native gallery connects through real loopback permission and reviews local VRT bytes",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1", timeout: 180000 },
  async (t) => {
    const root = await mkdtemp(path.join(os.tmpdir(), "musea-native-hosted-vrt-"));
    const output = path.join(repository, "artifacts/musea-native-hosted-vrt");
    await mkdir(output, { recursive: true });
    const built = await buildServiceGallery(root, output);
    const host = await createSecureHost(built.directory);
    const restoreTrust = await trustSecureHost(host);
    const local = path.join(root, "local-vrt");
    const options = parseArgs([
      "serve",
      "--gallery-url",
      `${host.origin}/built/gallery/`,
      "--output",
      local,
    ]);
    const history = JSON.parse(
      await readFile(
        path.join(repository, "tests/_fixtures/differential/musea/hosted-vrt-session.json"),
        "utf8",
      ),
    ) as { viewport: { name: string; width: number; height: number } };
    options.vrt = { viewports: [history.viewport], threshold: 0.1, capture: { settleTime: 0 } };
    const session = await startHostedVrtSession(options, host.spki);
    assert.equal(new URL(session.endpoint).hostname, "127.0.0.1");
    assert.notEqual(new URL(session.endpoint).origin, host.origin);
    const browser = await launchPublicBrowser(host);
    const page = await browser.newPage();
    const addressSpaces = await createAddressSpaceObserver(page);
    const permissionCleanup: Array<() => Promise<void>> = [];
    const errors: string[] = [];
    const observations: unknown[] = [];
    page.on("pageerror", (error) => errors.push(String(error)));
    t.after(async () => {
      for (const cleanup of permissionCleanup.reverse()) await cleanup();
      await addressSpaces.close();
      await browser.close();
      await session.close();
      restoreTrust();
      await host.close();
      await rm(root, { recursive: true, force: true });
    });
    const artifact = async (route: string) => {
      const response = await fetch(`${session.endpoint}${route}`, {
        headers: { Origin: host.origin, Authorization: `Bearer ${session.token}` },
      });
      assert.equal(response.status, 200);
      return Buffer.from(await response.arrayBuffer());
    };
    try {
      observations.push({
        phase: "compiled-cli-lifecycle",
        receipt: await proveCompiledSession(host, options.galleryUrl!, root, output),
      });
      observations.push({
        phase: "input-refusals",
        receipts: await refuseSessionInputs(session, host.origin, built.artPath, local),
      });
      assert.deepEqual(await storedCaptureBytes(local), {});
      await page.goto(`${host.origin}/built/gallery/`);
      assert.equal(await page.evaluate(() => window.isSecureContext), true);
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
      assert.equal(await page.getByRole("button", { name: "Run VRT", exact: true }).count(), 0);
      permissionCleanup.push(await setLoopbackPermission(page, "denied"));
      const denied = await page.evaluate(
        async () =>
          (await navigator.permissions.query({ name: "loopback-network" as PermissionName })).state,
      );
      assert.equal(denied, "denied");
      await page.getByRole("textbox", { name: "VRT endpoint", exact: true }).fill(session.endpoint);
      await page.getByLabel("Session token", { exact: true }).fill(session.token);
      await page.getByRole("button", { name: "Connect VRT", exact: true }).click();
      await page.getByRole("alert").filter({ hasText: "Loopback access is blocked" }).waitFor();
      const deniedFetch = await page.evaluate(async (endpoint) => {
        try {
          await fetch(`${endpoint}/session`, { targetAddressSpace: "loopback" } as RequestInit);
          return "unexpected-success";
        } catch (error) {
          return error instanceof Error ? error.name : String(error);
        }
      }, session.endpoint);
      assert.equal(deniedFetch, "TypeError");
      assert.deepEqual(await storedCaptureBytes(local), {});
      observations.push({
        phase: "real-public-loopback-denied",
        permission: denied,
        fetch: deniedFetch,
      });
      permissionCleanup.push(await setLoopbackPermission(page, "granted"));
      const allowed = await page.evaluate(
        async () =>
          (await navigator.permissions.query({ name: "loopback-network" as PermissionName })).state,
      );
      assert.equal(allowed, "granted");
      await page.getByLabel("Session token", { exact: true }).fill(session.token);
      await page.getByRole("button", { name: "Connect VRT", exact: true }).click();
      await page.getByText("Connected to your local VRT session.", { exact: true }).waitFor();
      assert.ok(
        addressSpaces.records.some(
          (item) => item.origin === host.origin && item.resourceIPAddressSpace === "Public",
        ),
      );
      assert.ok(
        addressSpaces.records.some(
          (item) =>
            item.origin === session.endpoint &&
            item.initiatorIPAddressSpace === "Public" &&
            item.initiatorIsSecureContext === true &&
            item.localNetworkAccessRequestPolicy === "PermissionBlock",
        ),
      );
      assert.ok(
        addressSpaces.records.some(
          (item) => item.origin === session.endpoint && item.resourceIPAddressSpace === "Loopback",
        ),
      );
      assert.equal(await page.getByLabel("Session token", { exact: true }).count(), 0);
      observations.push({
        phase: "real-public-loopback-allowed",
        permission: allowed,
        chromium: browser.version(),
        secureContext: true,
        publicAddressSpaceOverride: new URL(host.origin).host,
        certificateSpki: host.spki,
        certificateSha256: sha256(await readFile(host.certificatePath)),
        originalInputSha256: built.inputSha256,
      });
      const first = await capture(page, session.endpoint);
      assert.equal(first.summary.new, 1);
      const baseline = await artifact(first.results[0].images.snapshot);
      const png = PNG.sync.read(baseline);
      assert.deepEqual([png.width, png.height], [320, 180]);
      let blue = 0;
      for (let i = 0; i < png.data.length; i += 4)
        if (png.data[i] === 0 && png.data[i + 1] === 0 && png.data[i + 2] === 255) blue++;
      assert.ok(blue > 2000);
      await writeFile(path.join(output, "baseline-blue.png"), baseline);
      observations.push({
        phase: "new",
        data: first,
        baselineSha256: sha256(baseline),
        bluePixels: blue,
      });
      const repeated = await capture(page, session.endpoint);
      assert.equal(repeated.results[0].diffPercentage, 0);
      assert.equal(repeated.summary.passed, 1);
      observations.push({ phase: "match", data: repeated });
      const rebuilt = await buildServiceGallery(root, output, true);
      assert.notEqual(rebuilt.inputSha256, built.inputSha256);
      const changed = await capture(page, session.endpoint);
      assert.equal(changed.summary.failed, 1);
      assert.ok(changed.results[0].diffPercentage! > 0);
      const changedBytes = await artifact(changed.results[0].images.current);
      const diff = await artifact(changed.results[0].images.diff);
      assert.notEqual(sha256(changedBytes), sha256(baseline));
      assert.equal(sha256(await artifact(changed.results[0].images.snapshot)), sha256(baseline));
      await writeFile(path.join(output, "current-green.png"), changedBytes);
      await writeFile(path.join(output, "actual-diff.png"), diff);
      observations.push({
        phase: "physical-rebuild-diff",
        data: changed,
        changedInputSha256: rebuilt.inputSha256,
      });
      await page.getByRole("checkbox", { name: "Update snapshots", exact: true }).check();
      observations.push({ phase: "update", data: await capture(page, session.endpoint) });
      await page.getByRole("checkbox", { name: "Update snapshots", exact: true }).uncheck();
      const clean = await capture(page, session.endpoint);
      assert.equal(clean.results[0].diffPercentage, 0);
      assert.equal(clean.summary.passed, 1);
      assert.equal(sha256(await artifact(clean.results[0].images.snapshot)), sha256(changedBytes));
      observations.push({ phase: "match-after-update", data: clean });
      for (const kind of ["json", "html"] as const) {
        const bytes = await artifact(clean.reports[kind]);
        assert.equal(
          sha256(bytes),
          sha256(await readFile(path.join(local, `reports/vrt-Button-report.${kind}`))),
        );
        await writeFile(path.join(output, `authenticated-report.${kind}`), bytes);
        const [download] = await Promise.all([
          page.waitForEvent("download"),
          page
            .getByRole("button", { name: `Download ${kind.toUpperCase()} report`, exact: true })
            .click(),
        ]);
        const file = path.join(output, `downloaded-report.${kind}`);
        await download.saveAs(file);
        assert.equal(sha256(await readFile(file)), sha256(bytes));
        const deniedArtifact = await fetch(`${session.endpoint}${clean.reports[kind]}`, {
          headers: { Origin: host.origin },
        });
        assert.equal(deniedArtifact.status, 401);
      }
      observations.push({
        phase: "navigation-refusals",
        receipts: await refuseHostedNavigation(
          session,
          host,
          rebuilt.directory,
          local,
          built.artPath,
        ),
      });
      observations.push({
        phase: "serial-capture",
        receipt: await proveSerialCapture(session, host.origin, built.artPath),
      });
      observations.push({
        phase: "owned-report-error",
        receipt: await proveOwnedHostedError(page, session, local, output, options.galleryUrl!),
      });
      assert.equal((await capture(page, session.endpoint)).summary.passed, 1);
      const beforeTrustRefusal = await storedCaptureBytes(local);
      const wrongPinSession = await startHostedVrtSession(
        options,
        Buffer.alloc(32, 7).toString("base64"),
      );
      try {
        const refusal = await fetch(`${wrongPinSession.endpoint}/capture`, {
          method: "POST",
          headers: {
            Origin: host.origin,
            Authorization: `Bearer ${wrongPinSession.token}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ artPath: built.artPath, update: true }),
        });
        assert.equal(refusal.status, 400);
        assert.deepEqual(await storedCaptureBytes(local), beforeTrustRefusal);
        observations.push({ phase: "mismatching-certificate-refusal", status: refusal.status });
      } finally {
        await wrongPinSession.close();
      }
      assert.deepEqual(errors, []);
      await page.screenshot({ path: path.join(output, "gallery.png"), fullPage: true });
      assert.equal(
        (await page.locator(".vrt-results img").first().getAttribute("src"))?.startsWith("blob:"),
        true,
      );
      assert.ok(
        host.requests.every(
          (item) => !item.pathname.includes("@vite") && !item.pathname.endsWith(".art.vue"),
        ),
      );
    } catch (error) {
      await writeFile(
        path.join(output, "failure.json"),
        JSON.stringify({ error: String(error), observations, errors }, null, 2),
      );
      throw error;
    } finally {
      const receipt = JSON.stringify(
        { observations, errors, requests: host.requests, addressSpaces: addressSpaces.records },
        null,
        2,
      );
      assert.equal(receipt.includes(session.token), false);
      await writeFile(path.join(output, "observations.json"), receipt);
      await cp(local, path.join(output, "local-vrt"), { recursive: true }).catch((error) => {
        if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
      });
    }
  },
);
