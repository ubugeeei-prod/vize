import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { buildCaptureIdentity } from "./snapshot-identity.ts";
import { SnapshotIndex, SNAPSHOT_INDEX_FILE, SNAPSHOT_LOCK_FILE } from "./snapshot-index.ts";

const capture = {
  art: { path: "src/Button.art.vue" },
  variantName: "Default",
  viewport: { width: 200, height: 100 },
};
const identity = buildCaptureIdentity(capture);

async function directory(t: { after: (fn: () => Promise<unknown>) => unknown }): Promise<string> {
  const result = await fs.mkdtemp(path.join(os.tmpdir(), "musea-snapshot-index-validation-"));
  t.after(() => fs.rm(result, { recursive: true, force: true }));
  return result;
}

void test("invalid ownership files fail before any PNG change and release their own lock", async (t) => {
  const other = buildCaptureIdentity({ ...capture, variantName: "Other" });
  const malformed = [
    "not json",
    { version: 2, owners: {} },
    { version: 1, owners: [] },
    { version: 1, owners: { "../outside.png": identity } },
    { version: 1, owners: { "é.png": identity } },
    { version: 1, owners: { "image.png": "not json" } },
    { version: 1, owners: { "first.png": identity, "second.png": identity } },
    { version: 1, owners: { "Button.png": identity, "button.png": other } },
    { version: 1, owners: { [`snapshot-${"0".repeat(64)}.png`]: identity } },
  ];
  for (const data of malformed) {
    const dir = await directory(t);
    const file = path.join(dir, SNAPSHOT_INDEX_FILE);
    const content = typeof data === "string" ? data : JSON.stringify(data);
    await fs.writeFile(file, content);
    await fs.writeFile(path.join(dir, "existing.png"), "unchanged");
    await assert.rejects(SnapshotIndex.open(dir), /Invalid snapshot ownership index/);
    assert.equal(await fs.readFile(file, "utf8"), content);
    assert.equal(await fs.readFile(path.join(dir, "existing.png"), "utf8"), "unchanged");
    await assert.rejects(fs.access(path.join(dir, SNAPSHOT_LOCK_FILE)), { code: "ENOENT" });
  }
});

void test("locks exclude a second run, close is idempotent and cannot remove a replaced lock", async (t) => {
  const dir = await directory(t);
  const first = await SnapshotIndex.open(dir);
  const lockFile = path.join(dir, SNAPSHOT_LOCK_FILE);
  const lock = await fs.readFile(lockFile, "utf8");
  await assert.rejects(SnapshotIndex.open(dir), /directory is locked/);
  assert.equal(await fs.readFile(lockFile, "utf8"), lock);
  await fs.unlink(lockFile);
  await fs.writeFile(lockFile, "another owner");
  await first.close();
  await first.close();
  assert.equal(await fs.readFile(lockFile, "utf8"), "another owner");
  await assert.rejects(first.plan([capture]), /index is closed/);
  await fs.unlink(lockFile);
  const second = await SnapshotIndex.open(dir);
  await second.close();
  await assert.rejects(fs.access(lockFile), { code: "ENOENT" });
});

void test("invalid maps and duplicate capture jobs cannot reserve or overwrite a filename", async (t) => {
  const dir = await directory(t);
  await assert.rejects(
    SnapshotIndex.open(dir, {
      snapshotIdentities: { a: "left/Button.art.vue", b: "left/Button.art.vue" },
    }),
    /Duplicate Art snapshot identity/,
  );
  await assert.rejects(
    SnapshotIndex.open(dir, { snapshotIdentities: { a: "../outside" } }),
    /Invalid project-relative/,
  );
  assert.deepEqual(await fs.readdir(dir), []);
  const index = await SnapshotIndex.open(dir);
  try {
    await assert.rejects(index.plan([capture, capture]), /Duplicate snapshot capture identity/);
    await assert.rejects(fs.access(path.join(dir, SNAPSHOT_INDEX_FILE)), { code: "ENOENT" });
    assert.equal((await index.plan([capture])).size, 1);
  } finally {
    await index.close();
  }
});

void test("a failed mixed adoption leaves the complete old index unchanged", async (t) => {
  const dir = await directory(t);
  const index = await SnapshotIndex.open(dir);
  try {
    await index.plan([capture]);
    const before = await fs.readFile(path.join(dir, SNAPSHOT_INDEX_FILE), "utf8");
    const unowned = { ...capture, art: { path: "src/Badge.art.vue" } };
    await fs.writeFile(path.join(dir, "Badge--Default--200x100.png"), "unowned");
    await assert.rejects(
      index.plan([unowned, { ...capture, art: { path: "src/Alert.art.vue" } }]),
      /Explicit adoptLegacySnapshots/,
    );
    assert.equal(await fs.readFile(path.join(dir, SNAPSHOT_INDEX_FILE), "utf8"), before);
    const fresh = { ...capture, art: { path: "src/Alert.art.vue" } };
    assert.equal(
      (await index.plan([fresh])).get(buildCaptureIdentity(fresh)),
      "Alert--Default--200x100.png",
    );
  } finally {
    await index.close();
  }
});

void test("adoption and owned reuse refuse symlinks before baseline IO", async (t) => {
  const dir = await directory(t);
  const outside = path.join(dir, "outside.txt");
  const filename = "Button--Default--200x100.png";
  await fs.writeFile(outside, "external content");
  await fs.symlink(outside, path.join(dir, filename));
  const adopter = await SnapshotIndex.open(dir, { adoptLegacySnapshots: true });
  try {
    await assert.rejects(adopter.plan([capture]), /regular PNG file/);
  } finally {
    await adopter.close();
  }
  await fs.unlink(path.join(dir, filename));
  const owner = await SnapshotIndex.open(dir);
  try {
    await owner.plan([capture]);
    await fs.symlink(outside, path.join(dir, filename));
    await assert.rejects(owner.plan([capture]), /regular PNG file/);
    assert.equal(await fs.readFile(outside, "utf8"), "external content");
  } finally {
    await owner.close();
  }
});
