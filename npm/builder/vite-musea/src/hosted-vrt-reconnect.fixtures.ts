import assert from "node:assert/strict";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { PNG } from "pngjs";
import type { Page } from "playwright";
import { sha256 } from "./hosted-vrt-build.fixtures.ts";
import { renderFrames } from "./hosted-vrt-lifecycle.fixtures.ts";

interface Capture {
  success: boolean;
  results: Array<{
    artPath: string;
    variantName: string;
    viewport: string;
    isNew?: boolean;
    passed: boolean;
    diffPercentage?: number;
    error?: string;
    images: Record<string, string>;
  }>;
  summary: {
    total: number;
    passed: number;
    failed: number;
    new: number;
    errors: number;
    skipped: number;
    duration: number;
  };
  reports: { json: string; html: string };
}

export async function connectSession(page: Page, endpoint: string, token: string) {
  await page.getByRole("textbox", { name: "VRT endpoint", exact: true }).fill(endpoint);
  await page.getByLabel("Session token", { exact: true }).fill(token);
  await page.getByRole("button", { name: "Connect VRT", exact: true }).click();
  await page.getByText("Connected to your local VRT session.", { exact: true }).waitFor();
  assert.equal(await page.getByLabel("Session token", { exact: true }).count(), 0);
}

/** Passive DOM checks; the observer never changes the pane or its real image bytes. */
export async function emptyPane(page: Page) {
  await renderFrames(page);
  for (const selector of [".vrt-error", ".vrt-results", ".vrt-summary", ".vrt-artifacts"])
    assert.equal(await page.locator(selector).count(), 0, selector);
  for (const kind of ["JSON", "HTML"])
    assert.equal(
      await page.getByRole("button", { name: `Download ${kind} report`, exact: true }).count(),
      0,
    );
  return { html: await page.locator(".vrt-panel").innerHTML(), empty: true };
}

/** Require the authentic old renderer's precise failure of the same empty-pane assertion. */
export async function proveStaleReconnectedPane(page: Page, output: string) {
  await renderFrames(page);
  let failure: unknown;
  try {
    await emptyPane(page);
  } catch (error) {
    failure = error;
  }
  assert.ok(failure instanceof assert.AssertionError);
  assert.match(failure.message, /^\.vrt-error(?:\n|$)/);
  assert.equal(failure.actual, 1);
  assert.equal(failure.expected, 0);
  assert.equal(failure.operator, "strictEqual");
  const displayed = (await page.locator(".vrt-error p").textContent())?.trim();
  assert.equal(
    displayed,
    "Local VRT session: HTTP 401. Reconnect with the endpoint and token printed by musea-vrt serve.",
  );
  assert.equal(
    await page.getByText("Connected to your local VRT session.", { exact: true }).count(),
    1,
  );
  assert.equal(await page.locator(".vrt-results").count(), 0);
  assert.equal(await page.locator(".vrt-summary").count(), 0);
  await page.screenshot({ path: path.join(output, "actual-old-stale-pane.png"), fullPage: true });
  const html = await page.locator(".vrt-panel").innerHTML();
  await writeFile(path.join(output, "actual-old-stale-pane.html"), html);
  return {
    phase: "actual-before-empty-pane-red",
    selector: ".vrt-error",
    actual: failure.actual,
    expected: failure.expected,
    operator: failure.operator,
    assertionMessage: failure.message,
    displayed,
    html,
  };
}

/** Authenticate the actual service response, physical PNG/report files, and native UI downloads. */
export async function captureReconnected(input: {
  page: Page;
  session: { endpoint: string; token: string };
  relayEndpoint: string;
  origin: string;
  artPath: string;
  local: string;
  output: string;
  phase: "first" | "reconnected";
  first?: Buffer;
}) {
  const { page, session, phase } = input;
  const [response] = await Promise.all([
    page.waitForResponse(
      (item) =>
        item.url() === `${input.relayEndpoint}/capture` && item.request().method() === "POST",
    ),
    page.getByRole("button", { name: "Run VRT", exact: true }).click(),
  ]);
  assert.deepEqual(response.request().postDataJSON(), { artPath: input.artPath, update: false });
  const raw = await response.body();
  const destination = path.join(input.output, phase);
  await mkdir(destination, { recursive: true });
  await writeFile(path.join(destination, "response.json"), raw);
  assert.equal(response.status(), 200, raw.toString());
  const data = JSON.parse(raw.toString()) as Capture;
  assert.equal(data.success, true);
  assert.ok(Number.isFinite(data.summary.duration) && data.summary.duration >= 0);
  assert.deepEqual(
    { ...data.summary, duration: 0 },
    {
      total: 1,
      passed: phase === "first" ? 0 : 1,
      failed: 0,
      new: phase === "first" ? 1 : 0,
      errors: 0,
      skipped: 0,
      duration: 0,
    },
  );
  assert.equal(data.results.length, 1);
  const item = data.results[0];
  assert.equal(item.artPath, input.artPath);
  assert.equal(item.variantName, "Default");
  assert.equal(item.viewport, "compact");
  assert.equal(item.passed, true);
  assert.equal(item.error, undefined);
  if (phase === "first") assert.equal(item.isNew, true);
  else assert.ok(item.isNew === undefined || item.isNew === false);
  if (phase === "reconnected") assert.equal(item.diffPercentage, 0);
  assert.deepEqual(Object.keys(item.images), ["snapshot", "current"]);
  const artifact = async (route: string) => {
    assert.match(route, /^\/artifacts\/[0-9a-f-]{36}$/);
    const actual = await fetch(`${session.endpoint}${route}`, {
      headers: { Origin: input.origin, Authorization: `Bearer ${session.token}` },
    });
    assert.equal(actual.status, 200);
    return Buffer.from(await actual.arrayBuffer());
  };
  const physicalJson = await readFile(path.join(input.local, "reports/vrt-Button-report.json"));
  const report = JSON.parse(physicalJson.toString());
  assert.deepEqual(report.reportOwner, { version: 1, artIdentity: "src/Button.art.vue" });
  assert.deepEqual(report.summary, data.summary);
  assert.equal(report.results.length, data.results.length);
  const written = report.results[0];
  assert.equal(written.art, "Button");
  for (const key of ["artPath", "variantName", "viewport", "diffPercentage", "error"] as const)
    assert.equal(written[key], item[key]);
  assert.equal(written.variant, item.variantName);
  assert.equal(written.status, item.isNew ? "new" : "passed");
  const images: unknown[] = [];
  let baseline: Buffer | undefined;
  for (const [kind, route] of Object.entries(item.images)) {
    const bytes = await artifact(route);
    assert.deepEqual(bytes, await readFile(report.results[0][`${kind}Path`]));
    const png = PNG.sync.read(bytes);
    assert.deepEqual([png.width, png.height], [320, 180]);
    if (input.first) assert.deepEqual(bytes, input.first);
    if (kind === "snapshot") baseline = bytes;
    await writeFile(path.join(destination, `${kind}.png`), bytes);
    images.push({ kind, bytes: bytes.length, sha256: sha256(bytes) });
  }
  assert.ok(baseline);
  await page.locator(".vrt-results img").first().waitFor();
  await page.waitForFunction(() =>
    [...document.querySelectorAll<HTMLImageElement>(".vrt-results img")].every(
      (image) => image.complete && image.naturalWidth === 320 && image.src.startsWith("blob:"),
    ),
  );
  assert.deepEqual(await page.locator(".vrt-variant-name").allTextContents(), ["Default"]);
  const reports: unknown[] = [];
  for (const kind of ["json", "html"] as const) {
    const bytes = await artifact(data.reports[kind]);
    assert.deepEqual(
      bytes,
      await readFile(path.join(input.local, `reports/vrt-Button-report.${kind}`)),
    );
    await writeFile(path.join(destination, `authenticated-report.${kind}`), bytes);
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page
        .getByRole("button", { name: `Download ${kind.toUpperCase()} report`, exact: true })
        .click(),
    ]);
    const downloaded = path.join(destination, `downloaded-report.${kind}`);
    await download.saveAs(downloaded);
    assert.deepEqual(await readFile(downloaded), bytes);
    reports.push({ kind, bytes: bytes.length, sha256: sha256(bytes) });
  }
  assert.equal(await page.locator(".vrt-error").count(), 0);
  await page.screenshot({ path: path.join(destination, "pane.png"), fullPage: true });
  return { baseline, receipt: { phase, data, images, reports } };
}

export async function noPersistedSessionToken(directory: string, tokens: string[]) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) await noPersistedSessionToken(filename, tokens);
    else if (entry.isFile()) {
      const bytes = await readFile(filename);
      for (const token of tokens) assert.equal(bytes.includes(token), false, filename);
    }
  }
}
