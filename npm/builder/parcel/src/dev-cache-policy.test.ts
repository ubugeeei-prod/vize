import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  advanceDevBundleMtime,
  type DevCacheRecord,
  parcelRespondsNotModified,
} from "./dev-cache-policy.ts";

const sameSecond = new Date("2026-11-04T03:04:05.250Z");

async function writeAt(file: string, text: string, when: Date): Promise<void> {
  await fs.writeFile(file, text);
  await fs.utimes(file, when, when);
}

void test("a same-second mtime is a Parcel 304 even when the bytes differ", () => {
  const later = sameSecond.getTime() + 700;
  assert.equal(new Date(later).toUTCString(), sameSecond.toUTCString());
  assert.equal(parcelRespondsNotModified(sameSecond.toUTCString(), later), true);
});

void test("changed bytes must not reuse a same-second Parcel 304", async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), "vize-parcel-304-"));
  try {
    const file = path.join(directory, "bundle.js");
    await writeAt(file, "Recovered", sameSecond);
    const recovered = await advanceDevBundleMtime(fs, file, undefined);
    assert.equal(parcelRespondsNotModified(sameSecond.toUTCString(), recovered.servedMtimeMs), true);

    await writeAt(file, "Discarded", sameSecond);
    const discarded = await advanceDevBundleMtime(fs, file, recovered);
    await writeAt(file, "RapidRecovered", sameSecond);
    const rapid = await advanceDevBundleMtime(fs, file, discarded);
    const stat = await fs.stat(file);
    assert.equal(await fs.readFile(file, "utf8"), "RapidRecovered");
    assert.equal(parcelRespondsNotModified(sameSecond.toUTCString(), stat.mtimeMs), false);
    assert.notEqual(stat.mtime.toUTCString(), sameSecond.toUTCString());
    assert.equal(parcelRespondsNotModified(sameSecond.toUTCString(), rapid.servedMtimeMs), false);
  } finally {
    await fs.rm(directory, { recursive: true, force: true });
  }
});

void test("unchanged bytes keep a same-second 304", async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), "vize-parcel-304-"));
  try {
    const file = path.join(directory, "icon.svg");
    await writeAt(file, "<svg/>", sameSecond);
    const first: DevCacheRecord = await advanceDevBundleMtime(fs, file, undefined);
    await writeAt(file, "<svg/>", sameSecond);
    await advanceDevBundleMtime(fs, file, first);
    const stat = await fs.stat(file);
    assert.equal(stat.mtime.toUTCString(), sameSecond.toUTCString());
    assert.equal(parcelRespondsNotModified(sameSecond.toUTCString(), stat.mtimeMs), true);
  } finally {
    await fs.rm(directory, { recursive: true, force: true });
  }
});
