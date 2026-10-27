import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const [directory, output] = process.argv.slice(2);
assert.ok(directory && output);
const version = "0.602.0";
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const root = fs.realpathSync(directory);
const rootRequire = createRequire(path.join(root, "package.json"));
const pluginManifest = fs.realpathSync(path.join(root, "node_modules/@vizejs/vite-plugin/package.json"));
const pluginRequire = createRequire(pluginManifest);
const lock = fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8");
const evidence = {
  schema: "vize.public-editor-registry-native",
  version, consumer: root, node: process.version,
  acceptanceHead: process.env.GITHUB_SHA,
  releaseTag: "v0.602.0",
  preparedTag: "92223735e9c41cefbfe584d7f364e4847569c92b",
  producerSource: "3fa403f8cf4edbd851f7ca91c0f0648200f5777f",
  editorMerge: "d756cde149df2ba447bad11440f26a8e949a6c66",
  lockSha256: sha256(lock), packages: [], physicalAddons: [],
};
function inside(file) {
  const real = fs.realpathSync(file);
  assert.ok(real.startsWith(root + path.sep), real);
  assert.ok(!real.includes("/Source/"), real);
  return real;
}
function manifestAbove(file, expected) {
  let directory = path.dirname(fs.realpathSync(file));
  for (;;) {
    const file = path.join(directory, "package.json");
    if (fs.existsSync(file)) {
      const bytes = fs.readFileSync(file);
      const pkg = JSON.parse(bytes);
      if (!expected || pkg.name === expected)
        return { file: inside(file), directory, pkg, bytes };
    }
    const parent = path.dirname(directory);
    assert.notEqual(parent, directory, "package manifest missing: " + expected);
    directory = parent;
  }
}
function extract(tarball, member) {
  const result = spawnSync("tar", ["-xOzf", "-", member], {
    input: tarball, maxBuffer: 128 * 1024 * 1024,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr.toString());
  return result.stdout;
}
try {
  const native = pluginRequire("@vizejs/native");
  evidence.nativeExports = Object.keys(native).sort();
  const addons = [];
  for (const file of Object.keys(pluginRequire.cache)) {
    if (!file.endsWith(".node")) continue;
    const manifest = manifestAbove(file);
    if (!manifest.pkg.name?.startsWith("@vizejs/native-")) continue;
    const real = inside(file);
    assert.equal(manifest.pkg.version, version);
    addons.push({ file: real, manifest });
  }
  assert.equal(addons.length, 1, "exactly one physical public native platform addon must be loaded");
  const packages = [
    manifestAbove(pluginManifest, "@vizejs/vite-plugin"),
    manifestAbove(rootRequire.resolve("vite-plus/package.json"), "vite-plus"),
    ...["vize", "@vizejs/native", "@vizejs/unplugin"].map((name) =>
      manifestAbove(pluginRequire.resolve(name), name)),
    addons[0].manifest,
  ];
  for (const manifest of packages) {
    const { name } = manifest.pkg;
    const expectedVersion = name === "vite-plus" ? "1.4.0" : version;
    assert.equal(manifest.pkg.version, expectedVersion, name);
    const metadataUrl = "https://registry.npmjs.org/" + encodeURIComponent(name) + "/" + expectedVersion;
    const response = await fetch(metadataUrl);
    assert.equal(response.status, 200, metadataUrl);
    const metadata = await response.json();
    assert.equal(metadata.name, name);
    assert.equal(metadata.version, expectedVersion);
    assert.ok(metadata.dist.integrity.startsWith("sha512-"));
    assert.ok(lock.includes(metadata.dist.integrity), name + ": registry SRI absent from fresh lockfile");
    const archiveResponse = await fetch(metadata.dist.tarball);
    assert.equal(archiveResponse.status, 200, metadata.dist.tarball);
    const archive = Buffer.from(await archiveResponse.arrayBuffer());
    const actualIntegrity = "sha512-" + createHash("sha512").update(archive).digest("base64");
    assert.equal(actualIntegrity, metadata.dist.integrity, name);
    const archiveManifest = extract(archive, "package/package.json");
    const archivePkg = JSON.parse(archiveManifest.toString());
    assert.equal(archivePkg.name, name);
    assert.equal(archivePkg.version, expectedVersion);
    const record = {
      name, version: expectedVersion, manifest: manifest.file, metadataUrl,
      registryPacket: metadata, observedTarballIntegrity: actualIntegrity,
      tarballSha256: sha256(archive), lockContainsRegistryIntegrity: true,
      installedManifestSha256: sha256(manifest.bytes),
      archiveManifestSha256: sha256(archiveManifest),
      manifestBytesEqual: manifest.bytes.equals(archiveManifest),
    };
    evidence.packages.push(record);
    for (const addon of addons.filter((entry) => entry.manifest.pkg.name === name)) {
      const installed = fs.readFileSync(addon.file);
      const member = "package/" + path.relative(manifest.directory, addon.file).split(path.sep).join("/");
      const publicBytes = extract(archive, member);
      assert.ok(installed.equals(publicBytes), "loaded addon differs from public SRI-verified tarball");
      evidence.physicalAddons.push({
        file: addon.file, package: name, version: expectedVersion,
        bytes: installed.length, sha256: sha256(installed), tarMember: member,
        publicTarMemberSha256: sha256(publicBytes), sameBytes: true,
      });
    }
  }
  assert.equal(evidence.physicalAddons.length, 1);
  evidence.status = "passed";
} catch (error) {
  evidence.status = "failed";
  evidence.error = { message: error.message, stack: error.stack };
  process.exitCode = 1;
} finally {
  evidence.finishedAt = new Date().toISOString();
  fs.writeFileSync(output, JSON.stringify(evidence, null, 2) + "\n");
  console.log(JSON.stringify({
    status: evidence.status, output, version,
    registryPackages: evidence.packages.length, physicalAddons: evidence.physicalAddons,
  }));
}
