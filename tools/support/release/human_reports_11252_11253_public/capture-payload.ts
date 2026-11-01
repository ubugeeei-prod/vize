import assert from "node:assert/strict";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { publicFiles } from "./dependency-bin-inputs.ts";
import type { PublicPackageFacts } from "./dependency-bin-types.ts";
import type { PublicPayloadReceipt } from "./model.ts";
import type { Sink } from "./evidence.ts";
import { observePackageInventory } from "./package-inventory.ts";
import { bytes, sha256 } from "./value.ts";

/** Every shipped byte is a mandatory prerequisite. Extra installed paths are
 * observed here, and remain explicitly pending the unchanged final set guard. */
export function capturePublicPayload(pkg: PublicPackageFacts, sink: Sink): PublicPayloadReceipt {
  assert(pkg.archive);
  observePackageInventory(pkg.directory, pkg.archive, sink);
  const observations: unknown[] = [];
  const observe = (complete: boolean, error: string | null) =>
    sink("public-payload-prerequisites", {
      package: pkg.name,
      version: pkg.version,
      complete,
      observations,
      error,
      bodyBytesExcluded: true,
      completeInstalledFileSetQualified: false,
    });
  let files: PublicPayloadReceipt["files"];
  try {
    files = [...publicFiles(pkg)]
      .map(([relative, expected]) => {
        const path = join(pkg.directory, relative);
        const stat = lstatSync(path);
        assert(stat.isFile() && !stat.isSymbolicLink(), "public payload must be a regular file");
        assert.equal(realpathSync(path), path, "public payload redirect refuses execution");
        const actual = readFileSync(path);
        observations.push({
          path,
          expectedBytes: expected.length,
          expectedSha256: sha256(expected),
          actualBytes: actual.length,
          actualSha256: sha256(actual),
        });
        assert(actual.equals(expected), "public payload mismatch refuses execution");
        return { path, bytes: actual.length, sha256: sha256(actual) };
      })
      .sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  } catch (error) {
    observe(false, String(error));
    throw error;
  }
  observe(true, null);
  assert(files.length > 0);
  return {
    name: pkg.name,
    version: pkg.version,
    directory: pkg.directory,
    integrity: pkg.integrity,
    resolved: pkg.resolved,
    archiveSha512: bytes(pkg.integrity.slice(7)).toString("hex"),
    files,
    manifestRawBase64: pkg.manifest.toString("base64"),
    publicPayloadBytesEqualArchive: true,
    completeInstalledFileSetQualified: false,
  };
}

/** Recheck mandatory payload bytes without converting pending extras into
 * qualified files, or claiming that the complete physical inventory is equal. */
export function recheckCapturePayload(packages: readonly PublicPayloadReceipt[]): void {
  for (const pkg of packages) {
    assert.equal(pkg.publicPayloadBytesEqualArchive, true);
    assert.equal(pkg.completeInstalledFileSetQualified, false);
    for (const image of pkg.files) {
      assert(image.path.startsWith(pkg.directory + "/"));
      assert.equal(realpathSync(image.path), image.path);
      const stat = lstatSync(image.path);
      assert(stat.isFile() && !stat.isSymbolicLink());
      const raw = readFileSync(image.path);
      assert.equal(raw.length, image.bytes);
      assert.equal(sha256(raw), image.sha256, "public payload changed; execute no further command");
    }
  }
}
