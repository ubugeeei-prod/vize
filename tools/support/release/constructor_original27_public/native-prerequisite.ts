import assert from "node:assert/strict";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { asRecord, asString } from "../../json-value.ts";
import type { Recorder } from "../../config/windows_consumer/common.ts";
import {
  assertCustody,
  fileImage,
  inside,
  sha256,
  type FileImage,
} from "../computed_event_custody.ts";
import { readTarGz } from "../../archive/tar.ts";
import { packedManifest } from "../slot_instance_public/packed-manifest.ts";
import { readBytes } from "../public_install/registry.ts";
import {
  requireNativeLoadPrerequisite,
  recheckNativeLoadPrerequisite,
} from "../human_reports_11252_11253_public/native-loader-prerequisite.ts";
import type { PackageBase } from "../human_reports_11252_11253_public/model.ts";
import { assertLockedPackage, type authenticate } from "./authority.ts";

/** Match every shipped regular file; npm-owned extras are inspected by the
 * existing non-executing module-resolution prerequisite before any require. */
export function shippedImages(root: string, payload: Buffer, integrity: unknown) {
  root = realpathSync(root);
  const archive = packedManifest(payload, integrity),
    images: FileImage[] = [];
  for (const entry of readTarGz(payload)) {
    if (entry.kind !== "file") continue;
    const path = join(root, entry.name.slice("package/".length));
    assert.ok(inside(root, path));
    assert.equal(lstatSync(path).isSymbolicLink(), false);
    const image = fileImage(path);
    assert.ok(inside(root, image.path));
    assert.equal(image.bytes, entry.data.length);
    assert.equal(image.sha256, sha256(entry.data));
    images.push(image);
  }
  assert.equal(images.length, archive.files.length);
  return { archive, images };
}
export async function nativePrerequisite(
  root: string,
  lock: Record<string, unknown>,
  authority: Awaited<ReturnType<typeof authenticate>>,
  sdk: FileImage[],
  record: Recorder,
  reader = readBytes,
) {
  const packages: PackageBase[] = [],
    images: FileImage[] = [];
  for (const name of ["vize", "@vizejs/native", "@vizejs/native-linux-x64-gnu"]) {
    assertLockedPackage(lock, authority.capsule, name, authority.ownership.version);
    const entry = asRecord(asRecord(lock.packages)["node_modules/" + name]);
    const directory = realpathSync(join(root, "node_modules", name));
    assert.ok(inside(realpathSync(join(root, "node_modules")), directory));
    const payload = await reader(asString(entry.resolved));
    const label = name.replaceAll("/", "_").replaceAll("@", "");
    record.json(label + ".archive-download.json", {
      url: entry.resolved,
      integrity: entry.integrity,
      bytes: payload.length,
      sha256: sha256(payload),
      verified: false,
    });
    const checked = shippedImages(directory, payload, entry.integrity);
    assert.deepEqual(readFileSync(join(directory, "package.json")), checked.archive.manifest);
    const manifest = asRecord(JSON.parse(checked.archive.manifest.toString("utf8")));
    assert.equal(manifest.name, name);
    assert.equal(manifest.version, authority.ownership.version);
    images.push(...checked.images);
    packages.push({
      name,
      version: asString(manifest.version),
      directory,
      resolved: asString(entry.resolved),
      integrity: asString(entry.integrity),
      archiveSha512: checked.archive.tarballSHA512,
      files: checked.images,
    });
    record.json(label + ".authenticated-shipped-files.json", {
      files: checked.images,
      archiveSha512: checked.archive.tarballSHA512,
      allShippedBytesEqualInstalled: true,
      completeNodeModulesFileSetQualified: false,
    });
  }
  const release = {
    version: authority.ownership.version,
    candidate: authority.candidate,
    source: authority.ownership.producerSource,
    workflowHead: authority.ownership.workflowHead,
    run: authority.ownership.runId,
    capsule: authority.capsule,
    sdk,
    observations: [],
    produced: authority.ownership.produced,
  };
  let phase = 0;
  function inspect<T>(body: (sink: (kind: string, value: unknown) => void) => T) {
    const events: unknown[] = [];
    try {
      return body((kind, value) => events.push({ kind, value }));
    } finally {
      record.json(`native-prerequisite-${phase++}.json`, events);
    }
  }
  const nativeLoad = inspect((sink) =>
    requireNativeLoadPrerequisite(root, packages, sdk, release, process.env, sink),
  );
  const verify = () => {
    assertCustody(images);
    inspect((sink) =>
      recheckNativeLoadPrerequisite({ root, packages, sdk, nativeLoad }, process.env, sink),
    );
  };
  verify();
  return { images, verify };
}
