import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  previewImageUrl,
  saveScreenshot,
  verifyScreenshotResponse,
} from "../../docs/previews/ui/screenshot-file.ts";

const fixture = path.resolve(import.meta.dirname, "../fixtures/docs-native-screenshot-cache");
const oldBody = readFileSync(path.join(fixture, "old.png"));
const newBody = readFileSync(path.join(fixture, "new.png"));
const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

void test("a new native PNG body bypasses a stale CDN entry without changing either body", () => {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-native-preview-cache-"));
  const stem = "use-cycle-list-720-interacted";
  try {
    const oldCapture = saveScreenshot(directory, stem, oldBody);
    const oldUrl = previewImageUrl(`${stem}.png`, directory);
    const staleCache = new Map([
      [oldUrl, oldBody],
      [`/component-previews/${stem}.png`, oldBody],
    ]);
    const newCapture = saveScreenshot(directory, stem, newBody);
    const newUrl = previewImageUrl(`${stem}.png`, directory);
    assert.notEqual(oldUrl, newUrl);
    assert.notEqual(oldCapture.sha256, newCapture.sha256);
    assert.ok(newUrl.endsWith(`-${sha256(newBody)}.png`));
    assert.deepEqual(readFileSync(path.join(directory, oldCapture.file)), oldBody);
    assert.deepEqual(readFileSync(path.join(directory, newCapture.file)), newBody);
    assert.deepEqual(readFileSync(path.join(directory, `${stem}.png`)), newBody);
    assert.equal(staleCache.has(newUrl), false);
    const response = staleCache.get(newUrl) ?? readFileSync(path.join(directory, newCapture.file));
    verifyScreenshotResponse(newUrl, response);
    assert.throws(
      () => verifyScreenshotResponse(newUrl, oldBody),
      /actual PNG body matches canonical URL/,
    );
    assert.throws(
      () => verifyScreenshotResponse(`/component-previews/${stem}.png`, newBody),
      /complete body SHA256/,
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
