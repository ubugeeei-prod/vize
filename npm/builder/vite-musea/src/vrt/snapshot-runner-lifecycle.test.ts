import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import type { ArtFileInfo } from "../types/index.ts";
import { MuseaVrtRunner } from "./runner.ts";

const viewport = { width: 200, height: 100, name: "small" };
const art = (root: string, name: string) =>
  ({ path: path.join(root, `${name}.art.vue`) }) as ArtFileInfo;

void test("one runner shares first concurrent ownership initialization without sharing distinct owners", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-concurrent-initialize-"));
  const runner = new MuseaVrtRunner({ projectRoot: root, snapshotDir: root });
  const other = new MuseaVrtRunner({ projectRoot: root, snapshotDir: root });
  try {
    const pending = [
      runner.getSnapshotName(art(root, "Button"), "Default", viewport),
      runner.getSnapshotName(art(root, "Badge"), "Default", viewport),
    ];
    await Promise.allSettled(pending);
    const names = await Promise.all(pending);
    assert.equal(new Set(names).size, 2);
    const index = JSON.parse(await readFile(path.join(root, "identities.json"), "utf8"));
    assert.equal(Object.keys(index.owners).length, 2);
    assert.equal(new Set(Object.values(index.owners)).size, 2);
    await assert.rejects(other.getSnapshotName(art(root, "Other"), "Default", viewport), /locked/);
    await runner.close();
    assert.ok(await other.getSnapshotName(art(root, "Other"), "Default", viewport));
  } finally {
    await runner.close();
    await other.close();
    await rm(root, { recursive: true, force: true });
  }
});

void test("close waits for requested initialization and leaves no late ownership lock", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-close-initialize-"));
  const runner = new MuseaVrtRunner({ projectRoot: root, snapshotDir: root });
  try {
    const capture = runner.getSnapshotName(art(root, "Button"), "Default", viewport);
    const closing = runner.close();
    const [name] = await Promise.all([capture, closing]);
    assert.equal(name, "Button--Default--small.png");
    await assert.rejects(stat(path.join(root, "identities.lock")), /ENOENT/);
    const index = JSON.parse(await readFile(path.join(root, "identities.json"), "utf8"));
    assert.equal(Object.keys(index.owners).length, 1);
  } finally {
    await runner.close();
    await rm(root, { recursive: true, force: true });
  }
});

void test("failed ownership initialization is retryable and unused close creates no directory", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-retry-initialize-"));
  const unused = new MuseaVrtRunner({ snapshotDir: path.join(root, "unused") });
  const runner = new MuseaVrtRunner({ projectRoot: root, snapshotDir: root });
  try {
    await unused.close();
    await assert.rejects(stat(path.join(root, "unused")), /ENOENT/);
    await writeFile(path.join(root, "identities.json"), "broken JSON");
    await assert.rejects(
      runner.getSnapshotName(art(root, "Button"), "Default", viewport),
      /Invalid snapshot ownership index/,
    );
    await assert.rejects(stat(path.join(root, "identities.lock")), /ENOENT/);
    await rm(path.join(root, "identities.json"));
    assert.equal(
      await runner.getSnapshotName(art(root, "Button"), "Default", viewport),
      "Button--Default--small.png",
    );
  } finally {
    await runner.close();
    await rm(root, { recursive: true, force: true });
  }
});
