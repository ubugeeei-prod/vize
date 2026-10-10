import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Page, Request } from "playwright";
import { PNG } from "pngjs";
import { startHostedVrtSession } from "./cli/serve.ts";
import type { CliOptions } from "./cli/index.ts";
import { sha256 } from "./hosted-vrt-build.fixtures.ts";
import {
  createLifecycleRelay,
  observeObjectUrls,
  renderFrames,
} from "./hosted-vrt-lifecycle.fixtures.ts";

interface Capture {
  success: boolean;
  results: Array<{ images: Record<string, string> }>;
  summary: { total: number; errors: number };
  reports: { html: string; json: string };
}

interface AuthorityControl {
  page: Page;
  objectUrls: Awaited<ReturnType<typeof observeObjectUrls>>;
  relay: Awaited<ReturnType<typeof createLifecycleRelay>>;
  session: { endpoint: string; token: string };
  options: CliOptions;
  certificateSpki: string;
  origin: string;
  output: string;
}

async function capture(page: Page, endpoint: string) {
  const [response] = await Promise.all([
    page.waitForResponse(
      (item) => item.url() === `${endpoint}/capture` && item.request().method() === "POST",
    ),
    page.getByRole("button", { name: "Run VRT", exact: true }).click(),
  ]);
  const body = await response.body();
  assert.equal(response.status(), 200);
  const data = JSON.parse(body.toString()) as Capture;
  assert.equal(data.success, true);
  assert.equal(data.summary.total, 1);
  assert.equal(data.summary.errors, 0);
  return { data, receipt: { status: response.status(), bytes: body.length, sha256: sha256(body) } };
}

function artifactSettlement(page: Page, url: string): Promise<string> {
  return new Promise((resolve) => {
    const settle = (request: Request, outcome: string) => {
      if (request.method() !== "GET" || request.url() !== url) return;
      page.off("requestfinished", finished);
      page.off("requestfailed", failed);
      resolve(outcome);
    };
    const finished = (request: Request) => settle(request, "finished");
    const failed = (request: Request) => settle(request, "failed");
    page.on("requestfinished", finished);
    page.on("requestfailed", failed);
  });
}

/** Real A-authorized bytes pending while a different real B service refuses A's bearer. */
export async function proveSessionAuthorityInvalidation(control: AuthorityControl) {
  const { page, relay, session, options, certificateSpki, origin, objectUrls } = control;
  const output = path.join(control.output, "authority-invalidation");
  await mkdir(output, { recursive: true });
  const receipt: Record<string, unknown> = { phase: "same-session-401-with-pending-png" };
  let second: Awaited<ReturnType<typeof startHostedVrtSession>> | undefined;
  let pending: ReturnType<typeof relay.holdNext> | undefined;
  let refusalHold: ReturnType<typeof relay.holdNext> | undefined;
  const artifact = async (route: string) => {
    const actual = await fetch(`${session.endpoint}${route}`, {
      headers: { Origin: origin, Authorization: `Bearer ${session.token}` },
    });
    assert.equal(actual.status, 200);
    return Buffer.from(await actual.arrayBuffer());
  };
  try {
    relay.selectUpstream(session.endpoint);
    await page.getByRole("textbox", { name: "VRT endpoint", exact: true }).fill(relay.endpoint);
    await page.getByLabel("Session token", { exact: true }).fill(session.token);
    await page.getByRole("button", { name: "Connect VRT", exact: true }).click();
    await page.getByText("Connected to your local VRT session.", { exact: true }).waitFor();
    const first = await capture(page, relay.endpoint);
    await page.waitForFunction(() => {
      const images = [...document.querySelectorAll<HTMLImageElement>(".vrt-results img")];
      return (
        images.length > 0 && images.every((image) => image.complete && image.naturalWidth === 320)
      );
    });
    const firstImages = [];
    for (const [kind, route] of Object.entries(first.data.results[0].images)) {
      const bytes = await artifact(route);
      const png = PNG.sync.read(bytes);
      assert.deepEqual([png.width, png.height], [320, 180]);
      await writeFile(path.join(output, `first-${kind}.png`), bytes);
      firstImages.push({ kind, bytes: bytes.length, sha256: sha256(bytes) });
    }
    const html = await artifact(first.data.reports.html);
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.getByRole("button", { name: "Download HTML report", exact: true }).click(),
    ]);
    const downloaded = path.join(output, "first-report.html");
    await download.saveAs(downloaded);
    assert.equal(sha256(await readFile(downloaded)), sha256(html));
    receipt.firstCapture = { ...first.receipt, images: firstImages, htmlSha256: sha256(html) };
    receipt.firstObjectUrls = await objectUrls.counts();
    assert.ok((await objectUrls.counts()).live > 0);

    pending = relay.holdNext({ pathname: /^\/artifacts\//, status: 200, contentType: "image/png" });
    const next = await capture(page, relay.endpoint);
    const held = await pending.buffered;
    assert.ok(Object.values(next.data.results[0].images).includes(held.receipt.pathname));
    const original = await artifact(held.receipt.pathname);
    assert.equal(sha256(original), held.receipt.sha256);
    await writeFile(path.join(output, "original-held-a.png"), held.body);
    receipt.secondCapture = next.receipt;
    receipt.heldA = held.receipt;
    receipt.beforeInvalidation = await objectUrls.counts();
    const settled = artifactSettlement(page, `${relay.endpoint}${held.receipt.pathname}`);

    second = await startHostedVrtSession(options, certificateSpki);
    assert.notEqual(second.token, session.token);
    assert.notEqual(second.endpoint, session.endpoint);
    relay.selectUpstream(second.endpoint);
    receipt.upstreams = { first: session.endpoint, second: second.endpoint, distinctToken: true };
    refusalHold = relay.holdNext({ pathname: first.data.reports.html, status: 401 });
    const refusalResponse = page.waitForResponse(
      (item) =>
        item.url() === `${relay.endpoint}${first.data.reports.html}` &&
        item.request().method() === "GET",
    );
    await page.getByRole("button", { name: "Download HTML report", exact: true }).click();
    const buffered401 = await refusalHold.buffered;
    assert.equal(buffered401.receipt.status, 401);
    assert.equal(buffered401.body.length, 0);
    await writeFile(path.join(output, "original-b-401.body"), buffered401.body);
    receipt.realB401 = buffered401.receipt;
    refusalHold.release();
    await refusalHold.released;
    const refusal = await refusalResponse;
    assert.equal(refusal.status(), 401);
    await renderFrames(page);
    const after401 = await objectUrls.counts();
    receipt.after401 = after401;
    receipt.reconnectVisibleAfter401 = await page
      .getByLabel("Session token", { exact: true })
      .count();
    receipt.reportButtonsAfter401 = await page
      .getByRole("button", { name: "Download HTML report", exact: true })
      .count();
    pending.release();
    await pending.released;
    receipt.pendingASettlement = await settled;
    await renderFrames(page);
    const afterRelease = await objectUrls.counts();
    receipt.afterRelease = afterRelease;
    receipt.reconnectVisibleAfterRelease = await page
      .getByLabel("Session token", { exact: true })
      .count();
    receipt.reportButtonsAfterRelease = await page
      .getByRole("button", { name: "Download HTML report", exact: true })
      .count();
    assert.equal(
      receipt.reconnectVisibleAfter401,
      1,
      "Actual 401 retained invalid session authority",
    );
    assert.equal(receipt.reconnectVisibleAfterRelease, 1);
    assert.equal(receipt.reportButtonsAfter401, 0);
    assert.equal(
      receipt.reportButtonsAfterRelease,
      0,
      "A late response revived invalidated reports",
    );
    assert.equal(after401.live, 0);
    assert.equal(afterRelease.live, 0, "A late A PNG retained an invalidated object URL");
    assert.equal(
      afterRelease.created,
      after401.created,
      "A late A PNG created an invalidated object URL",
    );
    return receipt;
  } finally {
    pending?.release();
    refusalHold?.release();
    relay.selectUpstream(session.endpoint);
    const serialized = JSON.stringify({ ...receipt, relayResponses: relay.receipts }, null, 2);
    assert.equal(serialized.includes(session.token), false);
    if (second) assert.equal(serialized.includes(second.token), false);
    await writeFile(path.join(output, "observations.json"), serialized);
    await second?.close();
  }
}
