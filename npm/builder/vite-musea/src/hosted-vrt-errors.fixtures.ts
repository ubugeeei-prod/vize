import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright";
import { storedCaptureBytes } from "./hosted-vrt-controls.fixtures.ts";
import { sha256 } from "./hosted-vrt-build.fixtures.ts";
import { createLifecycleRelay, renderFrames } from "./hosted-vrt-lifecycle.fixtures.ts";

/** Preserve this exact browser request's actual owned error bytes at the relay. */
export async function proveOwnedHostedError(
  page: Page,
  session: { endpoint: string; token: string },
  local: string,
  output: string,
  galleryUrl: string,
) {
  const relay = await createLifecycleRelay(session.endpoint);
  const report = path.join(local, "reports/vrt-Button-report.json");
  const original = await readFile(report);
  const ambiguous = Buffer.from('{"reportOwner":{"version":1,"artIdentity":"foreign.art.vue"}}');
  const hold = relay.holdNext({ pathname: "/capture", status: 400 });
  async function connect(endpoint: string) {
    await page.goto(galleryUrl);
    await page.locator(".art-item").filter({ hasText: "Left" }).click();
    await page.getByRole("button", { name: "VRT", exact: true }).click();
    await page.getByRole("textbox", { name: "VRT endpoint", exact: true }).fill(endpoint);
    await page.getByLabel("Session token", { exact: true }).fill(session.token);
    await page.getByRole("button", { name: "Connect VRT", exact: true }).click();
    await page.getByText("Connected to your local VRT session.", { exact: true }).waitFor();
  }
  try {
    await connect(relay.endpoint);
    await writeFile(report, ambiguous);
    const before = await storedCaptureBytes(local);
    const delivered = page.waitForResponse(
      (item) => item.url() === `${relay.endpoint}/capture` && item.request().method() === "POST",
    );
    await page.getByRole("button", { name: "Run VRT", exact: true }).click();
    const actual = await hold.buffered;
    const data = JSON.parse(actual.body.toString()) as { error: string; success?: boolean };
    await writeFile(path.join(output, "actual-owned-error.json"), actual.body);
    assert.equal(actual.receipt.status, 400);
    assert.notEqual(data.success, true);
    assert.match(data.error, /Ambiguous VRT report ownership:/);
    assert.match(data.error, /Archive or move the existing reports before retrying\./);
    hold.release();
    await hold.released;
    assert.equal((await delivered).status(), actual.receipt.status);
    await renderFrames(page);
    const displayed = await page.locator(".vrt-error").textContent();
    await writeFile(
      path.join(output, "owned-error-pane.json"),
      JSON.stringify({ displayed, actual: actual.receipt }, null, 2),
    );
    await page.screenshot({ path: path.join(output, "owned-error-pane.png"), fullPage: true });
    assert.ok(displayed?.includes(data.error), `Hosted pane discarded actual error: ${displayed}`);
    assert.equal(await page.locator(".vrt-error .vrt-hint").count(), 0);
    assert.deepEqual(await storedCaptureBytes(local), before);
    assert.equal(sha256(await readFile(report)), sha256(ambiguous));
    return { ...actual.receipt, error: data.error, displayed };
  } finally {
    hold.release();
    await writeFile(report, original);
    await relay.close();
    if (!page.isClosed()) await connect(session.endpoint);
  }
}
