import assert from "node:assert/strict";
import { readFile, rm, writeFile } from "node:fs/promises";
import { createServer as createTcpServer } from "node:net";
import path from "node:path";
import type { Browser, Page } from "playwright";
import { galleryPath, sides, type Side, type VrtFixture, sha256 } from "./vrt_fixtures.ts";
import {
  assertSummary,
  assertApiReport,
  cleanup,
  inventory,
  ownershipIndex,
  pngDetails,
  retainReport,
  type ApiCapture,
  type Evidence,
  type RetainedCapture,
  type VrtCounts,
} from "./vrt_artifacts.ts";

async function unusedPort() {
  const server = createTcpServer();
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  await new Promise<void>((resolve, reject) =>
    server.close((error) => (error ? reject(error) : resolve())),
  );
  return address.port;
}

async function openServer(f: VrtFixture) {
  const { createServer } = await import("vite");
  const server = await createServer({
    root: f.root,
    configFile: f.config,
    server: { host: "127.0.0.1", port: await unusedPort(), strictPort: true },
  });
  try {
    await server.listen();
    const address = server.httpServer!.address();
    assert.ok(address && typeof address !== "string");
    return { server, url: `http://127.0.0.1:${address.port}${galleryPath}` };
  } catch (error) {
    await server.close().catch((closeError) => {
      throw new AggregateError([error, closeError], "Vite launch cleanup failed");
    });
    throw error;
  }
}

export async function selectVrt(page: Page, url: string, side: Side) {
  const title = side === "left" ? "Left" : "Right";
  await page.goto(url);
  await page.locator(".art-item").filter({ hasText: title }).click();
  const preview = page.frameLocator(".variant-card iframe");
  await preview.getByRole("button", { name: title, exact: true }).waitFor();
  assert.equal(await preview.locator("museacomponent").count(), 0);
  await page.getByRole("button", { name: "VRT", exact: true }).click();
}

async function requestCapture(page: Page, e: Evidence, url: string, phase: string, status = 200) {
  const expected = new URL("api/run-vrt", url).href;
  const [response] = await Promise.all([
    page.waitForResponse((item) => item.url() === expected && item.request().method() === "POST", {
      timeout: 120_000,
    }),
    page.getByRole("button", { name: "Run VRT", exact: true }).click(),
  ]);
  const bytes = await response.body();
  await e.save(`${phase}/response.json`, bytes);
  await e.json(`${phase}/http.json`, {
    status: response.status(),
    url: response.url(),
    request: response.request().postData(),
    headers: response.headers(),
  });
  assert.equal(response.status(), status, bytes.toString());
  return JSON.parse(bytes.toString());
}

async function capture(
  page: Page,
  f: VrtFixture,
  e: Evidence,
  url: string,
  side: Side,
  phase: string,
  counts: VrtCounts,
): Promise<RetainedCapture> {
  const data = (await requestCapture(page, e, url, phase)) as ApiCapture;
  assert.equal(data.success, true);
  assertSummary(data, 1, counts);
  assert.equal(data.results[0].artPath, f.arts[side]);
  assert.equal(data.artifacts.snapshotDir, path.join(f.root, f.settings.snapshotDir));
  const identity = `src/${side}/Button.art.vue`;
  const expected = `vrt-art-${sha256(JSON.stringify([1, identity]))}-report`;
  for (const [kind, filename] of [
    ["json", data.artifacts.jsonReportPath],
    ["html", data.artifacts.htmlReportPath],
  ]) {
    assert.equal(filename, path.join(f.root, ".vize/reports", `${expected}.${kind}`));
  }
  const retained = await retainReport(
    e,
    phase,
    data.artifacts.jsonReportPath,
    data.artifacts.htmlReportPath,
  );
  const viewport = f.settings.viewports[0];
  assertApiReport(retained.data, data, viewport.width, viewport.height);
  assert.deepEqual(retained.data.reportOwner, { version: 1, artIdentity: identity });
  assert.deepEqual(
    retained.data.results.map((item) => item.artPath),
    [f.arts[side]],
  );
  const html = await readFile(data.artifacts.htmlReportPath);
  const png = await readFile(data.results[0].snapshotPath);
  assert.ok(
    pngDetails(png).colors[side === "right" ? "red" : "blue"] > 2000 ||
      (side === "left" && pngDetails(png).colors.green > 2000),
  );
  await page.locator(".vrt-variant-name").filter({ hasText: "Default" }).waitFor();
  assert.deepEqual(await page.locator(".vrt-variant-name").allTextContents(), ["Default"]);
  await e.save(`${phase}/gallery.html`, await page.content());
  await page.screenshot({ path: path.join(e.output, phase, "gallery.png"), fullPage: true });
  e.records.push({ phase, data });
  return { data, json: retained.json, html, png };
}

async function unchanged(other: RetainedCapture) {
  assert.deepEqual(await readFile(other.data.artifacts.jsonReportPath), other.json);
  assert.deepEqual(await readFile(other.data.artifacts.htmlReportPath), other.html);
  assert.deepEqual(await readFile(other.data.results[0].snapshotPath), other.png);
}

export async function observeVrtApi(browser: Browser, f: VrtFixture, e: Evidence) {
  let active = await openServer(f);
  const page = await browser.newPage().catch(async (error) => {
    await active.server.close().catch((closeError) => {
      throw new AggregateError([error, closeError], "Vite page launch cleanup failed");
    });
    throw error;
  });
  page.setDefaultTimeout(15_000);
  const errors: string[] = [],
    consoleErrors: unknown[] = [];
  page.on("pageerror", (error) => errors.push(String(error)));
  page.on("console", (message) => {
    if (message.type() === "error")
      consoleErrors.push({ text: message.text(), location: message.location() });
  });
  const latest = {} as Record<Side, RetainedCapture>;
  try {
    for (const side of sides) {
      await selectVrt(page, active.url, side);
      const token = await page.evaluate(() => {
        const token = crypto.randomUUID();
        Reflect.set(window, "__installedVrtDocument", token);
        return token;
      });
      const first = await capture(page, f, e, active.url, side, `api-${side}-new`, {
        new: 1,
        passed: 0,
        failed: 0,
      });
      if (side === "right") await unchanged(latest.left);
      const repeat = await capture(page, f, e, active.url, side, `api-${side}-match`, {
        passed: 1,
        new: 0,
        failed: 0,
      });
      assert.equal(repeat.data.results[0].diffPercentage, 0);
      assert.deepEqual(repeat.png, first.png);
      assert.deepEqual(repeat.data.artifacts, first.data.artifacts);
      assert.equal(await page.evaluate(() => Reflect.get(window, "__installedVrtDocument")), token);
      latest[side] = repeat;
    }
    assert.notEqual(
      latest.left.data.results[0].snapshotPath,
      latest.right.data.results[0].snapshotPath,
    );
    assert.notDeepEqual(latest.left.png, latest.right.png);
    await selectVrt(page, active.url, "left");
    const reloaded = page.waitForEvent("framenavigated", {
      predicate: (frame) => frame === page.mainFrame(),
      timeout: 15_000,
    });
    const changed = f.authored.left.toString().replace("#0000ff", "#00ff00");
    assert.notEqual(changed, f.authored.left.toString());
    await writeFile(f.arts.left, changed);
    await reloaded;
    await page.waitForFunction(
      async (url) => {
        const art = await (await fetch(url)).json();
        return art.variants[0].template.includes("#00ff00");
      },
      new URL(`api/arts/${encodeURIComponent(f.arts.left)}`, active.url).href,
    );
    await page.getByRole("button", { name: "VRT", exact: true }).click();
    const diff = await capture(
      page,
      f,
      e,
      active.url,
      "left",
      "api-physical-diff-configured-threshold",
      { passed: 1, new: 0, failed: 0 },
    );
    assert.ok(
      typeof diff.data.results[0].diffPercentage === "number" &&
        diff.data.results[0].diffPercentage > 0 &&
        diff.data.results[0].diffPercentage < f.settings.threshold,
    );
    assert.deepEqual(diff.png, latest.left.png);
    assert.ok(pngDetails(await readFile(diff.data.results[0].currentPath!)).colors.green > 2000);
    await unchanged(latest.right);
    await page.getByRole("checkbox", { name: "Update snapshots", exact: true }).check();
    await capture(page, f, e, active.url, "left", "api-update", { passed: 1, new: 0, failed: 0 });
    await page.getByRole("checkbox", { name: "Update snapshots", exact: true }).uncheck();
    latest.left = await capture(page, f, e, active.url, "left", "api-updated-match", {
      passed: 1,
      new: 0,
      failed: 0,
    });
    assert.equal(latest.left.data.results[0].diffPercentage, 0);
    assert.ok(pngDetails(latest.left.png).colors.green > 2000);
    await unchanged(latest.right);
    const snapshotDir = path.join(f.root, f.settings.snapshotDir);
    const index = await ownershipIndex(snapshotDir);
    await e.save("api-index-before-restart.json", index.bytes);
    await active.server.close();
    active = await openServer(f);
    for (const side of sides) {
      await selectVrt(page, active.url, side);
      const previous = latest[side];
      latest[side] = await capture(page, f, e, active.url, side, `api-${side}-restart-match`, {
        passed: 1,
        new: 0,
        failed: 0,
      });
      assert.equal(latest[side].data.results[0].diffPercentage, 0);
      assert.deepEqual(latest[side].data.artifacts, previous.data.artifacts);
      assert.deepEqual(latest[side].png, previous.png);
      await unchanged(latest[side === "left" ? "right" : "left"]);
    }
    assert.deepEqual((await ownershipIndex(snapshotDir)).bytes, index.bytes);
    assert.deepEqual(errors, []);
    assert.deepEqual(consoleErrors, []);
    const reportDir = path.dirname(latest.right.data.artifacts.jsonReportPath);
    const { reportOwner: _owner, ...legacy } = JSON.parse(latest.right.json.toString());
    const legacyJson = JSON.stringify(legacy, null, 2);
    await writeFile(path.join(reportDir, "vrt-Button-report.json"), legacyJson);
    await writeFile(path.join(reportDir, "vrt-Button-report.html"), latest.right.html);
    await rm(f.arts.right);
    await page.waitForFunction(
      async ({ url, removed }) => {
        const arts = await (await fetch(url)).json();
        return arts.length === 1 && arts.every((art: { path: string }) => art.path !== removed);
      },
      { url: new URL("api/arts", active.url).href, removed: f.arts.right },
    );
    await selectVrt(page, active.url, "left");
    const before = { snapshots: await inventory(snapshotDir), reports: await inventory(reportDir) };
    const refusal = await requestCapture(page, e, active.url, "api-removed-owner-refusal", 500);
    assert.match(refusal.error, /Ambiguous VRT report ownership.*Archive or move/);
    await page.locator(".vrt-error").filter({ hasText: "Archive or move" }).waitFor();
    assert.deepEqual(await inventory(snapshotDir), before.snapshots);
    assert.deepEqual(await inventory(reportDir), before.reports);
    await unchanged(latest.right);
    await e.json("api-refusal-inventories.json", before);
    await e.save("api-removed-owner-refusal/gallery.html", await page.content());
    assert.deepEqual(errors, []);
  } catch (error) {
    e.records.push({
      phase: "api-failure",
      error: String(error),
      stack: error instanceof Error ? error.stack : undefined,
    });
    await cleanup(e, "api-failure-evidence", [
      { name: "document", run: async () => e.save("api-failure.html", await page.content()) },
      {
        name: "screenshot",
        run: () =>
          page.screenshot({ path: path.join(e.output, "api-failure.png"), fullPage: true }),
      },
    ]).catch((evidenceError) =>
      e.records.push({ phase: "api-failure-evidence", error: String(evidenceError) }),
    );
    throw error;
  } finally {
    e.records.push({ phase: "api-browser-errors", errors, consoleErrors });
    await cleanup(e, "api-cleanup", [
      { name: "page", run: () => page.close() },
      { name: "vite", run: () => active.server.close() },
    ]);
  }
}
