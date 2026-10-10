import assert from "node:assert/strict";
import { cp, readFile, stat } from "node:fs/promises";
import path from "node:path";
import type { Browser } from "playwright";
import { type VrtFixture } from "./vrt_fixtures.ts";
import {
  assertSummary,
  cleanup,
  ownershipIndex,
  pngDetails,
  type Evidence,
} from "./vrt_artifacts.ts";
import { buildHostedVrt, staticVrtHost } from "./vrt_host.ts";
import { captureHostedCli, installedVrtBin, runVrtCli } from "./vrt_cli.ts";
import { observeHostedAudit } from "./vrt_audit.ts";

export async function observeHostedVrt(browser: Browser, f: VrtFixture, e: Evidence) {
  const firstBuild = await buildHostedVrt(f, e, "hosted-original", false);
  const host = await staticVrtHost(firstBuild.directory, e);
  const page = await browser.newPage().catch(async (error) => {
    await host.close().catch((closeError) => {
      throw new AggregateError([error, closeError], "Hosted page launch cleanup failed");
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
  try {
    const manifestResponse = await fetch(new URL("api/static.json", host.url));
    assert.equal(manifestResponse.status, 200);
    assert.deepEqual(Buffer.from(await manifestResponse.arrayBuffer()), firstBuild.raw);
    const bin = await installedVrtBin();
    const baselineDir = path.join(f.root, "cli-baselines");
    const first = await captureHostedCli(f, e, bin, host.url, "cli-hosted-new");
    assertSummary(first.data, 2, { new: 2, passed: 0, failed: 0 });
    const byArt = new Map(first.data.results.map((result) => [result.artPath, result]));
    const left = byArt.get(f.arts.left)!,
      right = byArt.get(f.arts.right)!;
    assert.ok(left && right);
    assert.notEqual(left.snapshotPath, right.snapshotPath);
    const leftPng = await readFile(left.snapshotPath),
      rightPng = await readFile(right.snapshotPath);
    assert.ok(pngDetails(leftPng).colors.blue > 2000);
    assert.ok(pngDetails(rightPng).colors.red > 2000);
    const repeated = await captureHostedCli(f, e, bin, host.url, "cli-hosted-match");
    assertSummary(repeated.data, 2, { passed: 2, failed: 0, new: 0 });
    assert.ok(repeated.data.results.every((result) => result.diffPercentage === 0));
    assert.deepEqual(await readFile(left.snapshotPath), leftPng);
    assert.deepEqual(await readFile(right.snapshotPath), rightPng);
    const originalIndex = await ownershipIndex(baselineDir);
    await e.save("cli-index-before-diff.json", originalIndex.bytes);
    await buildHostedVrt(f, e, "hosted-physical-green-rebuild", true);
    const changed = await captureHostedCli(
      f,
      e,
      bin,
      host.url,
      "cli-hosted-physical-diff",
      false,
      1,
    );
    assertSummary(changed.data, 2, { failed: 1, passed: 1, new: 0 });
    const changedLeft = changed.data.results.find((result) => result.artPath === f.arts.left)!;
    assert.ok(typeof changedLeft.diffPercentage === "number" && changedLeft.diffPercentage > 0);
    assert.ok(pngDetails(await readFile(changedLeft.currentPath!)).colors.green > 2000);
    assert.deepEqual(await readFile(left.snapshotPath), leftPng);
    assert.deepEqual(await readFile(right.snapshotPath), rightPng);
    const updated = await captureHostedCli(f, e, bin, host.url, "cli-hosted-update", true);
    assertSummary(updated.data, 2, { failed: 1, passed: 1, new: 0 });
    const clean = await captureHostedCli(f, e, bin, host.url, "cli-hosted-updated-match");
    assertSummary(clean.data, 2, { passed: 2, failed: 0, new: 0 });
    assert.ok(clean.data.results.every((result) => result.diffPercentage === 0));
    assert.ok(pngDetails(await readFile(left.snapshotPath)).colors.green > 2000);
    assert.deepEqual(await readFile(right.snapshotPath), rightPng);
    assert.deepEqual((await ownershipIndex(baselineDir)).bytes, originalIndex.bytes);
    await buildHostedVrt(f, e, "hosted-right-removed", true, false);
    await runVrtCli(f, e, bin, "cli-hosted-clean-tombstone", [
      "clean",
      "--gallery-url",
      host.url,
      "--config",
      f.config,
      "--output",
      path.join(f.root, "cli-output"),
    ]);
    await assert.rejects(stat(right.snapshotPath), { code: "ENOENT" });
    assert.deepEqual((await ownershipIndex(baselineDir)).bytes, originalIndex.bytes);
    await buildHostedVrt(f, e, "hosted-right-restored", true);
    const restored = await captureHostedCli(f, e, bin, host.url, "cli-hosted-restored-owner");
    assertSummary(restored.data, 2, { passed: 1, new: 1, failed: 0 });
    const restoredRight = restored.data.results.find((result) => result.artPath === f.arts.right)!;
    assert.equal(restoredRight.status, "new");
    assert.equal(
      restored.data.results.find((result) => result.artPath === f.arts.left)!.status,
      "passed",
    );
    assert.equal(restoredRight.snapshotPath, right.snapshotPath);
    assert.deepEqual(await readFile(restoredRight.snapshotPath), rightPng);
    assert.deepEqual((await ownershipIndex(baselineDir)).bytes, originalIndex.bytes);
    const final = await captureHostedCli(f, e, bin, host.url, "cli-hosted-restored-match");
    assertSummary(final.data, 2, { passed: 2, new: 0, failed: 0 });
    assert.ok(final.data.results.every((result) => result.diffPercentage === 0));
    await runVrtCli(f, e, bin, "cli-hosted-html", [
      "--gallery-url",
      host.url,
      "--config",
      f.config,
      "--output",
      path.join(f.root, "cli-output"),
      "--ci",
    ]);
    await e.save(
      "cli-hosted-html/report.html",
      await readFile(path.join(f.root, "cli-output/vrt-report.html")),
    );
    assert.deepEqual(await readFile(path.join(f.root, "cli-output/vrt-report.json")), final.json);
    await observeHostedAudit(page, f, e, host, () => {
      assert.deepEqual(errors, []);
      assert.deepEqual(consoleErrors, []);
    });
    assert.deepEqual(errors, []);
    assert.ok(
      host.requests.every(
        (item) =>
          !/(@vite|preview-module|\.art\.vue$)/.test((item as { pathname: string }).pathname),
      ),
    );
  } catch (error) {
    e.records.push({
      phase: "hosted-failure",
      error: String(error),
      stack: error instanceof Error ? error.stack : undefined,
    });
    await cleanup(e, "hosted-failure-evidence", [
      { name: "document", run: async () => e.save("hosted-failure.html", await page.content()) },
      {
        name: "screenshot",
        run: () =>
          page.screenshot({ path: path.join(e.output, "hosted-failure.png"), fullPage: true }),
      },
    ]).catch((evidenceError) =>
      e.records.push({ phase: "hosted-failure-evidence", error: String(evidenceError) }),
    );
    throw error;
  } finally {
    e.records.push({ phase: "hosted-browser-errors", errors, consoleErrors });
    const baselineDir = path.join(f.root, "cli-baselines");
    await cleanup(e, "hosted-cleanup", [
      { name: "page", run: () => page.close() },
      { name: "http", run: () => host.close() },
      { name: "requests", run: () => e.json("hosted-http-requests.json", host.requests) },
      {
        name: "baselines",
        run: () =>
          cp(baselineDir, path.join(e.output, "final-cli-baselines"), { recursive: true }).catch(
            (error: NodeJS.ErrnoException) => {
              if (error.code !== "ENOENT") throw error;
              e.records.push({ phase: "final-cli-baselines", absent: true });
            },
          ),
      },
    ]);
  }
}
