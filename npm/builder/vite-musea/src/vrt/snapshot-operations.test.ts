import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, writeFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import type { ArtFileInfo } from "../types/index.ts";
import { MuseaVrtRunner } from "./runner.ts";
import type { VrtResult } from "./types.ts";

function art(root: string, side: string): ArtFileInfo {
  return {
    path: path.join(root, side, "Button.art.vue"),
    metadata: { title: side, tags: [], status: "ready" },
    variants: [{ name: "Default", template: "", isDefault: true, skipVrt: false }],
    hasScript: false,
    hasScriptSetup: false,
    styleCount: 0,
  };
}

void test("clean retains variant viewport overrides and ownership reservations", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-clean-identities-"));
  const snapshotDir = path.join(root, "snapshots");
  const runner = new MuseaVrtRunner({
    projectRoot: root,
    snapshotDir,
    viewports: [{ width: 200, height: 100, name: "desktop" }],
  });
  const source = art(root, "left");
  const viewport = { width: 123, height: 45, name: "portrait" };
  source.variants[0].args = { viewport };
  try {
    const name = await runner.getSnapshotName(source, "Default", viewport);
    await writeFile(path.join(snapshotDir, name), "kept baseline");
    assert.equal(await runner.cleanOrphans([source]), 0);
    const owned = await readFile(path.join(snapshotDir, "identities.json"), "utf8");
    assert.equal(await runner.cleanOrphans([]), 1);
    assert.equal(await readFile(path.join(snapshotDir, "identities.json"), "utf8"), owned);
  } finally {
    await runner.close();
    await rm(root, { recursive: true, force: true });
  }
});

void test("approval refuses an ambiguous basename before copying and accepts relative ownership", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-approve-identities-"));
  const snapshotDir = path.join(root, "snapshots");
  const runner = new MuseaVrtRunner({ projectRoot: root, snapshotDir });
  const viewport = { width: 200, height: 100, name: "small" };
  try {
    const results: VrtResult[] = [];
    for (const side of ["left", "right"]) {
      const source = art(root, side);
      const name = await runner.getSnapshotName(source, "Default", viewport);
      const snapshotPath = path.join(snapshotDir, name);
      const currentPath = path.join(snapshotDir, "current", name);
      await mkdir(path.dirname(currentPath), { recursive: true });
      await writeFile(snapshotPath, `${side}-old`);
      await writeFile(currentPath, `${side}-new`);
      results.push({
        artPath: source.path,
        variantName: "Default",
        viewport,
        passed: false,
        snapshotPath,
        currentPath,
      });
    }
    await assert.rejects(
      runner.approveResults(results, "Button/*"),
      /Ambiguous approval pattern.*left\/Button.*right\/Button/,
    );
    assert.equal(await readFile(results[0].snapshotPath, "utf8"), "left-old");
    assert.equal(await readFile(results[1].snapshotPath, "utf8"), "right-old");
    assert.equal(await runner.approveResults(results, "right/Button/*"), 1);
    assert.equal(await readFile(results[0].snapshotPath, "utf8"), "left-old");
    assert.equal(await readFile(results[1].snapshotPath, "utf8"), "right-new");
  } finally {
    await runner.close();
    await rm(root, { recursive: true, force: true });
  }
});

void test("corrupt ownership prevents clean and baseline updates before PNG mutation", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-invalid-identities-"));
  const runner = new MuseaVrtRunner({ projectRoot: root, snapshotDir: root });
  const baseline = path.join(root, "Unowned.png");
  await writeFile(baseline, "unchanged");
  await writeFile(path.join(root, "identities.json"), "broken JSON");
  try {
    await assert.rejects(runner.cleanOrphans([]), /Invalid snapshot ownership index/);
    await assert.rejects(
      runner.updateBaselines([
        {
          artPath: path.join(root, "Button.art.vue"),
          variantName: "Default",
          viewport: { width: 200, height: 100 },
          passed: false,
          snapshotPath: baseline,
        },
      ]),
      /Invalid snapshot ownership index/,
    );
    assert.equal(await readFile(baseline, "utf8"), "unchanged");
  } finally {
    await runner.close();
    await rm(root, { recursive: true, force: true });
  }
});
