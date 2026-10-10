import assert from "node:assert/strict";
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import type { TestContext } from "node:test";
import { parseArgs } from "./cli/index.ts";
import { startHostedVrtSession } from "./cli/serve.ts";
import { buildServiceGallery, repository, sha256 } from "./hosted-vrt-build.fixtures.ts";
import { storedCaptureBytes } from "./hosted-vrt-controls.fixtures.ts";
import {
  createLifecycleRelay,
  observeObjectUrls,
  renderFrames,
} from "./hosted-vrt-lifecycle.fixtures.ts";
import {
  createSecureHost,
  launchPublicBrowser,
  setLoopbackPermission,
  trustSecureHost,
  createAddressSpaceObserver,
} from "./hosted-vrt-network.fixtures.ts";
import {
  captureReconnected,
  connectSession,
  emptyPane,
  noPersistedSessionToken,
  proveStaleReconnectedPane,
} from "./hosted-vrt-reconnect.fixtures.ts";

import { installOldRenderer } from "./hosted-vrt-old-renderer.fixtures.ts";

export async function proveReconnection(t: TestContext, renderer: "before" | "fixed") {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-native-vrt-reconnect-"));
  const output = path.join(repository, "artifacts/musea-native-hosted-vrt-reconnect", renderer);
  const local = path.join(root, "local-vrt");
  const observations: unknown[] = [];
  const errors: string[] = [];
  const consoleErrors: Array<{ text: string; url: string }> = [];
  const owned: {
    host?: Awaited<ReturnType<typeof createSecureHost>>;
    restoreTrust?: () => void;
    a?: Awaited<ReturnType<typeof startHostedVrtSession>>;
    b?: Awaited<ReturnType<typeof startHostedVrtSession>>;
    relay?: Awaited<ReturnType<typeof createLifecycleRelay>>;
    browser?: Awaited<ReturnType<typeof launchPublicBrowser>>;
    spaces?: Awaited<ReturnType<typeof createAddressSpaceObserver>>;
    detachPermission?: () => Promise<void>;
  } = {};
  let primaryFailure: unknown;
  t.after(async () => {
    const failures: unknown[] = [];
    const tokens = [owned.a?.token, owned.b?.token].filter((token): token is string => !!token);
    const redact = (error: unknown) =>
      tokens.reduce((text, token) => text.replaceAll(token, "[redacted]"), String(error));
    for (const cleanup of [
      () => owned.detachPermission?.(),
      () => owned.spaces?.close(),
      () => owned.browser?.close(),
      () => owned.relay?.close(),
      () => owned.b?.close(),
      () => owned.a?.close(),
      () => owned.restoreTrust?.(),
      () => owned.host?.close(),
      async () => {
        await mkdir(output, { recursive: true });
        const receipt = JSON.stringify(
          {
            observations,
            errors,
            consoleErrors,
            requests: owned.host?.requests,
            addressSpaces: owned.spaces?.records,
            failure: primaryFailure === undefined ? null : redact(primaryFailure),
            cleanupFailures: failures.map(redact),
          },
          null,
          2,
        );
        for (const token of tokens) assert.equal(receipt.includes(token), false);
        await writeFile(path.join(output, "observations.json"), receipt);
      },
      () =>
        cp(local, path.join(output, "local-vrt"), { recursive: true }).catch((error) => {
          if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
        }),
      () =>
        cp(path.join(root, "dist"), path.join(output, "built"), { recursive: true }).catch(
          (error) => {
            if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
          },
        ),
      () => noPersistedSessionToken(output, tokens),
      () => rm(root, { recursive: true, force: true }),
    ]) {
      try {
        await cleanup();
      } catch (error) {
        failures.push(error);
      }
    }
    if (failures.length) {
      if (primaryFailure !== undefined)
        throw new AggregateError(
          [primaryFailure, ...failures],
          "VRT reconnect failed and cleanup also failed",
        );
      throw new AggregateError(failures, "VRT reconnect cleanup failed");
    }
  });
  try {
    await mkdir(output, { recursive: true });
    const built = await buildServiceGallery(root, output);
    if (renderer === "before")
      observations.push({
        phase: "isolated-old-renderer",
        receipt: await installOldRenderer(root, built.directory, output),
      });
    const host = (owned.host = await createSecureHost(built.directory));
    owned.restoreTrust = await trustSecureHost(host);
    const url = `${host.origin}/built/gallery/`;
    const options = parseArgs(["serve", "--gallery-url", url, "--output", local]);
    options.vrt = {
      viewports: [{ name: "compact", width: 320, height: 180 }],
      threshold: 0,
      capture: { settleTime: 0 },
    };
    const a = (owned.a = await startHostedVrtSession(options, host.spki));
    const relay = (owned.relay = await createLifecycleRelay(a.endpoint));
    const browser = (owned.browser = await launchPublicBrowser(host));
    const page = await browser.newPage();
    const spaces = (owned.spaces = await createAddressSpaceObserver(page));
    const urls = await observeObjectUrls(page);
    page.on("pageerror", (error) => errors.push(String(error)));
    page.on("console", (message) => {
      if (message.type() === "error")
        consoleErrors.push({ text: message.text(), url: message.location().url });
    });
    await page.goto(url);
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
    owned.detachPermission = await setLoopbackPermission(page, "granted");
    const permission = await page.evaluate(
      async () =>
        (await navigator.permissions.query({ name: "loopback-network" as PermissionName })).state,
    );
    assert.equal(permission, "granted");
    await connectSession(page, relay.endpoint, a.token);
    const input = {
      page,
      relayEndpoint: relay.endpoint,
      origin: host.origin,
      artPath: built.artPath,
      local,
      output,
    };
    const first = await captureReconnected({ ...input, session: a, phase: "first" });
    observations.push({ ...first.receipt, urls: await urls.counts() });
    assert.ok((await urls.counts()).live > 0);
    const report = path.join(local, "reports/vrt-Button-report.json");
    const original = await readFile(report);
    const ambiguous = Buffer.from('{"reportOwner":{"version":1,"artIdentity":"foreign.art.vue"}}');
    await writeFile(report, ambiguous);
    const beforeError = await storedCaptureBytes(local);
    const errorHold = relay.holdNext({ pathname: "/capture", status: 400 });
    try {
      const delivered = page.waitForResponse(
        (item) => item.url() === `${relay.endpoint}/capture` && item.request().method() === "POST",
      );
      await page.getByRole("button", { name: "Run VRT", exact: true }).click();
      const actual = await errorHold.buffered;
      await writeFile(path.join(output, "actual-owned-error.json"), actual.body);
      const data = JSON.parse(actual.body.toString()) as { error: string; success?: boolean };
      assert.equal(actual.receipt.status, 400);
      assert.notEqual(data.success, true);
      assert.match(data.error, /Ambiguous VRT report ownership:/);
      assert.match(data.error, /Archive or move the existing reports before retrying\./);
      errorHold.release();
      await errorHold.released;
      assert.equal((await delivered).status(), 400);
      await page.locator(".vrt-error").waitFor();
      await renderFrames(page);
      const displayed = await page.locator(".vrt-error").textContent();
      assert.ok(displayed?.includes(data.error));
      assert.equal(await page.locator(".vrt-error .vrt-hint").count(), 0);
      assert.equal(
        await page.getByText("Connected to your local VRT session.", { exact: true }).count(),
        1,
      );
      assert.deepEqual(await storedCaptureBytes(local), beforeError);
      assert.deepEqual(await readFile(report), ambiguous);
      observations.push({
        phase: "owned-error-connected",
        receipt: actual.receipt,
        displayed,
        urls: await urls.counts(),
      });
      await page.screenshot({ path: path.join(output, "owned-error.png"), fullPage: true });
    } finally {
      errorHold.release();
      await writeFile(report, original);
    }
    const b = (owned.b = await startHostedVrtSession(options, host.spki));
    assert.notEqual(b.endpoint, a.endpoint);
    assert.notEqual(b.token, a.token);
    relay.selectUpstream(b.endpoint);
    const beforeUnauthorized = await storedCaptureBytes(local);
    const unauthorized = relay.holdNext({ pathname: "/capture", status: 401 });
    try {
      const delivered = page.waitForResponse(
        (item) => item.url() === `${relay.endpoint}/capture` && item.request().method() === "POST",
      );
      await page.getByRole("button", { name: "Run VRT", exact: true }).click();
      const actual = await unauthorized.buffered;
      assert.equal(actual.receipt.status, 401);
      assert.equal(actual.body.length, 0);
      await writeFile(path.join(output, "actual-unauthorized-body.bin"), actual.body);
      unauthorized.release();
      await unauthorized.released;
      assert.equal((await delivered).status(), 401);
      await page
        .getByRole("alert")
        .filter({ hasText: "Reconnect with the endpoint and token" })
        .waitFor();
      await page.getByLabel("Session token", { exact: true }).waitFor();
      const disconnected = await emptyPane(page);
      assert.equal(await page.getByRole("button", { name: "Run VRT", exact: true }).count(), 0);
      assert.equal(
        await page
          .getByText("Capture this hosted gallery with Node and Playwright:", { exact: true })
          .count(),
        1,
      );
      assert.equal((await urls.counts()).live, 0);
      assert.deepEqual(await storedCaptureBytes(local), beforeUnauthorized);
      observations.push({
        phase: "actual-authority-denied",
        receipt: actual.receipt,
        disconnected,
        urls: await urls.counts(),
      });
    } finally {
      unauthorized.release();
    }
    await connectSession(page, relay.endpoint, b.token);
    if (renderer === "before") {
      observations.push(await proveStaleReconnectedPane(page, output));
      assert.equal((await urls.counts()).live, 0);
    } else {
      await page
        .getByText('Click "Run VRT" to capture and compare screenshots.', { exact: true })
        .waitFor();
      const reconnected = await emptyPane(page);
      assert.equal(await page.getByRole("alert").count(), 0);
      assert.equal((await urls.counts()).live, 0);
      observations.push({
        phase: "reconnected-empty",
        pane: reconnected,
        urls: await urls.counts(),
      });
      await page.screenshot({ path: path.join(output, "reconnected-empty.png"), fullPage: true });
      const matched = await captureReconnected({
        ...input,
        session: b,
        phase: "reconnected",
        first: first.baseline,
      });
      observations.push({ ...matched.receipt, urls: await urls.counts() });
    }
    assert.deepEqual(errors, []);
    // These two genuine rejected HTTP responses may emit Chromium resource diagnostics.
    assert.ok(consoleErrors.length <= 2);
    for (const message of consoleErrors) {
      assert.equal(message.url, `${relay.endpoint}/capture`);
      assert.match(
        message.text,
        /^Failed to load resource: the server responded with a status of (400 \(Bad Request\)|401 \(Unauthorized\))$/,
      );
    }
    assert.ok(
      host.requests.every(
        (item) => !item.pathname.includes("@vite") && !item.pathname.endsWith(".art.vue"),
      ),
    );
    assert.ok(
      spaces.records.some(
        (item) => item.origin === host.origin && item.resourceIPAddressSpace === "Public",
      ),
    );
    assert.ok(
      spaces.records.some(
        (item) =>
          item.origin === relay.endpoint &&
          item.initiatorIPAddressSpace === "Public" &&
          item.initiatorIsSecureContext === true,
      ),
    );
    assert.ok(
      spaces.records.some(
        (item) => item.origin === relay.endpoint && item.resourceIPAddressSpace === "Loopback",
      ),
    );
    observations.push({
      phase: "secure-public-real-loopback",
      permission,
      chromium: browser.version(),
      publicAddressSpaceOverride: new URL(host.origin).host,
      certificateSpki: host.spki,
      certificateSha256: sha256(await readFile(host.certificatePath)),
      originalInputSha256: built.inputSha256,
      relay: relay.receipts,
    });
    await urls.restore();
  } catch (error) {
    primaryFailure = error;
    throw error;
  }
}
