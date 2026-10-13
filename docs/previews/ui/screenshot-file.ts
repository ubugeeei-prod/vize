/** Keep captured PNG bytes intact and give each body an immutable public identity. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const previewsRoot = path.resolve(import.meta.dirname, "../../public/component-previews");
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

export function saveScreenshot(outputRoot: string, stem: string, bytes: Uint8Array) {
  assert.match(stem, /^[a-z0-9-]+$/);
  const sha256 = digest(bytes);
  const file = `${stem}-${sha256}.png`;
  writeFileSync(path.join(outputRoot, file), bytes);
  // Preserve existing external links; newly generated documentation uses the immutable file.
  writeFileSync(path.join(outputRoot, `${stem}.png`), bytes);
  return { file, sha256 };
}

/** Source-only catalogue rendering works before captures exist; the site gate requires hashes. */
export function previewImageUrl(relativeFile: string, root = previewsRoot): string {
  assert.match(relativeFile, /^(?:composables\/)?[a-z0-9-]+\.png$/);
  const source = path.join(root, relativeFile);
  if (!existsSync(source)) return `/component-previews/${relativeFile}`;
  const file = relativeFile.replace(/\.png$/, `-${digest(readFileSync(source))}.png`);
  if (!existsSync(path.join(root, file))) return `/component-previews/${relativeFile}`;
  assert.equal(digest(readFileSync(path.join(root, file))), digest(readFileSync(source)));
  return `/component-previews/${file}`;
}

export function verifyScreenshotResponse(pathname: string, bytes: Uint8Array): void {
  const match = pathname.match(/-([a-f0-9]{64})\.png$/);
  assert.ok(match, `${pathname}: native screenshot URL must contain its complete body SHA256`);
  assert.equal(digest(bytes), match[1], `${pathname}: actual PNG body matches canonical URL`);
  assert.deepEqual(
    Buffer.from(bytes.subarray(0, 8)),
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
  );
}
