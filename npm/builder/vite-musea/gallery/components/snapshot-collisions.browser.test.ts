import assert from "node:assert/strict";
import { mkdir, readFile, writeFile, rm, readdir, copyFile } from "node:fs/promises";
import { createServer } from "node:http";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { MuseaVrtRunner } from "../../src/vrt/runner.ts";
import type { ArtFileInfo } from "../../src/types/index.ts";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const output = path.join(repository, "artifacts/musea-snapshot-collisions");
const fixtureRoot = path.join(repository, "tests/_fixtures/differential/musea/snapshot-collision");

await test(
  "colliding Art baselines are rejected before any real Chromium worker writes",
  { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1" },
  async () => {
    await mkdir(output, { recursive: true });
    const arts: ArtFileInfo[] = ["Left", "Right"].map((title) => ({
      path: path.join(fixtureRoot, title.toLowerCase(), "Button.art.vue"),
      metadata: { title, tags: [], status: "ready" },
      variants: [{ name: "Default", template: "", isDefault: true, skipVrt: false }],
      hasScriptSetup: false,
      hasScript: false,
      styleCount: 0,
    }));
    const sources = await Promise.all(arts.map((art) => readFile(art.path, "utf8")));
    assert.ok(sources[0].includes("#0000ff") && sources[1].includes("#ff0000"));
    const requests: string[] = [];
    // Native parsing of the complete persisted Arts is covered in package tests.
    // This browser boundary isolates real navigation, PNG comparison and writes.
    const server = createServer((request, response) => {
      const url = new URL(request.url || "/", "http://fixture.invalid");
      requests.push(url.href);
      const index = arts.findIndex((art) => art.path === url.searchParams.get("art"));
      if (index < 0) {
        response.statusCode = 404;
        response.end();
        return;
      }
      const color = index === 0 ? "#0000ff" : "#ff0000";
      response.setHeader("Content-Type", "text/html");
      response.end(
        `<html><body style="margin:0"><main class="musea-variant" style="width:100px;height:60px;background:${color}">${arts[index].metadata.title}</main></body></html>`,
      );
    });
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    const address = server.address();
    assert.ok(address && typeof address !== "string");
    const origin = `http://127.0.0.1:${address.port}`;
    const snapshotDir = path.join(output, "snapshots");
    await rm(snapshotDir, { recursive: true, force: true });
    const viewport = { width: 200, height: 100, name: "small" };
    const runner = new MuseaVrtRunner({
      snapshotDir,
      viewports: [viewport],
      workers: 2,
      capture: { waitForNetwork: false, settleTime: 0 },
    });
    await runner.init();
    try {
      // Retain the legacy collision: the right Art is compared to the left PNG.
      const left = await runner.captureAndCompare(arts[0], "Default", viewport, origin);
      const right = await runner.captureAndCompare(arts[1], "Default", viewport, origin);
      assert.equal(left.isNew, true);
      assert.equal(right.passed, false);
      assert.equal(left.snapshotPath, right.snapshotPath);
      assert.ok((right.diffPercentage ?? 0) > 20);
      await writeFile(
        path.join(output, "legacy.json"),
        JSON.stringify({ arts, sources, left, right }, null, 2),
      );
      await rm(snapshotDir, { recursive: true, force: true });
      const priorRequests = requests.length;
      await assert.rejects(
        runner.runAllTests(arts, origin),
        /Snapshot name collision.*Button--Default--small.png/,
      );
      assert.equal(requests.length, priorRequests);
      await assert.rejects(readdir(snapshotDir), /ENOENT/);
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify(
          { sources, requests, rejectedBeforeNavigation: true, rejectedBeforeWrites: true },
          null,
          2,
        ),
      );
    } finally {
      await runner.close();
      await new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      );
    }
  },
);
