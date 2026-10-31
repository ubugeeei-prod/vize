import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { readTarGz } from "../../archive/tar.ts";
import { asRecord, asString, parseJson, type JsonRecord } from "../../json-value.ts";
import type { Recorder } from "../../config/windows_consumer/common.ts";
import { fileImage, inside, sha256 } from "../computed_event_custody.ts";
import { publicUrl, readBytes, sriSha512 } from "../public_install/registry.ts";

/** These are exactly the sections transformed by release/npm/npm_publish.rs. */
const dependencySections = new Set([
  "dependencies",
  "devDependencies",
  "peerDependencies",
  "optionalDependencies",
]);
export function packedManifest(payload: Buffer, integrity: unknown) {
  const digest = createHash("sha512").update(payload).digest();
  assert.deepEqual(
    digest,
    sriSha512(integrity),
    "whole registry tarball must match the genuine capsule SRI",
  );
  const names = new Set<string>(),
    files = [];
  let manifest: Buffer | undefined;
  for (const member of readTarGz(payload)) {
    const parts = member.name.split("/");
    assert.equal(parts[0], "package");
    assert.ok(parts.every((part) => part !== "" && part !== "." && part !== ".."));
    assert.ok(!member.name.includes("\\"));
    assert.ok(
      member.kind === "file" || member.kind === "directory",
      "links and special package members refuse",
    );
    assert.ok(!names.has(member.name), "duplicate packed member refuses");
    names.add(member.name);
    if (member.kind === "directory") continue;
    assert.ok(parts.length > 1);
    files.push({ path: member.name, bytes: member.data.length, sha256: sha256(member.data) });
    if (member.name === "package/package.json") manifest = member.data;
  }
  assert.ok(manifest, "one real packed package.json required");
  return { manifest, files, tarballSHA512: digest.toString("hex") };
}
export function assertPackedManifest(installed: Buffer, packed: Buffer, source: JsonRecord) {
  assert.deepEqual(
    installed,
    packed,
    "entire installed manifest bytes must equal the SRI-authenticated packed member",
  );
  const manifest = asRecord(parseJson(new TextDecoder("utf-8", { fatal: true }).decode(packed)));
  const unchanged = (value: JsonRecord) =>
    Object.fromEntries(Object.entries(value).filter(([key]) => !dependencySections.has(key)));
  // No dependency is ignored: every dependency byte is authenticated above.
  // Source equality applies to all remaining fields, including every public mapping.
  assert.deepEqual(
    unchanged(manifest),
    unchanged(source),
    "all source fields outside the four proven publication transforms remain exact",
  );
  return manifest;
}
export function captureInstalledManifests(modules: string, record: Recorder) {
  const rows = new Map<
    string,
    { file: string; raw: Buffer; image: ReturnType<typeof fileImage> }
  >();
  for (const [name, label] of [
    ["vize", "cli"],
    ["@vizejs/native", "native"],
    ["@vizejs/native-linux-x64-gnu", "platform"],
  ]) {
    record.json("installation-phase.json", { phase: "capture-installed-manifests", package: name });
    const file = join(modules, name, "package.json"),
      raw = readFileSync(file),
      image = fileImage(file);
    assert.ok(inside(modules, image.path));
    assert.equal(image.bytes, raw.length);
    assert.equal(image.sha256, sha256(raw));
    writeFileSync(join(record.root, "installed-" + label + "-package.raw.json"), raw, {
      flag: "wx",
    });
    record.json("installed-" + label + "-manifest-image.json", { package: name, ...image });
    rows.set(name, { file, raw, image });
  }
  return rows;
}
export async function authenticatePackedManifest(
  capsule: JsonRecord,
  source: JsonRecord,
  installed: Buffer,
  record: Recorder,
) {
  record.json("installation-phase.json", { phase: "authenticate-packed-cli-manifest" });
  const entry = asRecord(asRecord(capsule.npmSRI).vize),
    url = publicUrl(asString(entry.tarball));
  const payload = await readBytes(url);
  writeFileSync(join(record.root, "packed-vize.tgz"), payload, { flag: "wx" });
  record.json("packed-vize-download.json", {
    url,
    integrity: entry.integrity,
    bytes: payload.length,
    sha256: sha256(payload),
    verified: false,
  });
  const archive = packedManifest(payload, entry.integrity);
  writeFileSync(join(record.root, "packed-cli-package.raw.json"), archive.manifest, { flag: "wx" });
  const manifest = assertPackedManifest(installed, archive.manifest, source);
  record.json("packed-vize-authority.json", {
    url,
    integrity: entry.integrity,
    bytes: payload.length,
    sha256: sha256(payload),
    tarballSHA512: archive.tarballSHA512,
    files: archive.files,
    manifest: { bytes: archive.manifest.length, sha256: sha256(archive.manifest) },
    allInstalledManifestBytesEqualPacked: true,
    sourceNonDependencyFieldsEqual: true,
    scope:
      "whole public SRI archive owns every packed field; immutable tag source owns the complete nondependency remainder",
  });
  return manifest;
}
