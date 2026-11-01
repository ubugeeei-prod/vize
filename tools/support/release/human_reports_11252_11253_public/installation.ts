import assert from "node:assert/strict";
import { readFileSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { archiveComparison } from "../public_install/archive.ts";
import { captureDeclarationGraph, fileImage, inside } from "../computed_event_custody.ts";
import type { Fixture, InstallReceipt, Recipe, ReleaseAuthority } from "./model.ts";
import { array, bytes, object, sha256, text, wholeJson } from "./value.ts";
export function manifestBytes(fixture: Fixture, version: string, supplement: boolean): Buffer {
  const source = fixture.files.find((f) => f.path === "package.json");
  assert(source);
  const original = bytes(source.base64).toString("utf8");
  assert.equal(original.split('"vize": "0.627.0"').length, 2);
  const replaced = original.replace('"vize": "0.627.0"', `"vize": "${version}"`);
  if (!supplement) return Buffer.from(replaced);
  assert.equal(fixture.issue, 11252);
  const manifest = object(JSON.parse(replaced));
  object(manifest.devDependencies).tsrs = "0.20.1-ts7.1.0-beta";
  return Buffer.from(JSON.stringify(manifest, null, 2) + "\n");
}
export function validateInstallation(
  receipt: InstallReceipt,
  fixture: Fixture,
  recipe: Recipe,
  authority: ReleaseAuthority,
  archiveBodies: ReadonlyMap<string, Buffer>,
): void {
  assert.equal(receipt.schema, "vize.humanReports11252_11253.install.v1");
  assert.equal(receipt.platform, recipe.platform);
  assert.equal(receipt.arch, recipe.arch);
  assert.equal(receipt.nodeVersion, recipe.node);
  assert.equal(receipt.pnpmVersion, recipe.pnpm);
  assert(receipt.kernel);
  assert.equal(realpathSync(receipt.root), receipt.root);
  assert.equal(receipt.node.sha256, fileImage(receipt.node.path).sha256);
  assert.equal(receipt.pnpm.sha256, fileImage(receipt.pnpm.path).sha256);
  const manifest = manifestBytes(fixture, authority.version, receipt.kind === "tsrs-supplement");
  assert(
    bytes(receipt.packageRawBase64).equals(manifest),
    "only explicit Vize version substitution is allowed in original manifest",
  );
  assert(readFileSync(join(receipt.root, "package.json")).equals(manifest));
  assert.deepEqual(receipt.packageManifest, wholeJson(manifest));
  const lockBytes = readFileSync(join(receipt.root, "pnpm-lock.yaml"));
  assert.equal(sha256(lockBytes), receipt.lockSha256);
  assert(
    lockBytes.equals(bytes(receipt.lockRawBase64)),
    "retain entire generated lock; no reporter lock is invented",
  );
  const expected = new Map([
    ["vize", authority.version],
    ["@vizejs/native", authority.version],
    [
      recipe.platform === "darwin" ? "@vizejs/native-darwin-arm64" : "@vizejs/native-linux-x64-gnu",
      authority.version,
    ],
    ["vue", recipe.vue],
    ["typescript", recipe.typescript],
  ]);
  if (recipe.vueTsc) expected.set("vue-tsc", recipe.vueTsc);
  if (receipt.kind === "tsrs-supplement") expected.set("tsrs", "0.20.1-ts7.1.0-beta");
  const seen = new Set<string>(),
    seenExpected = new Set<string>(),
    sri = object(authority.capsule.npmSRI);
  for (const pkg of receipt.packages) {
    assert(inside(receipt.root, pkg.directory));
    assert.equal(realpathSync(pkg.directory), pkg.directory);
    assert(!seen.has(pkg.directory));
    seen.add(pkg.directory);
    if (expected.get(pkg.name) === pkg.version) seenExpected.add(pkg.name);
    assert.equal(new URL(pkg.resolved).origin, "https://registry.npmjs.org");
    assert.match(pkg.integrity, /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
    if (pkg.name === "vize" || pkg.name.startsWith("@vizejs/native")) {
      const row = object(sri[pkg.name]);
      assert.equal(row.integrity, pkg.integrity);
      assert.equal(row.tarball, pkg.resolved);
    }
    assert(pkg.files.length > 0);
    const archive = archiveBodies.get(pkg.name + "@" + pkg.version);
    assert(archive, `whole public archive required for ${pkg.name}`);
    const comparison = archiveComparison(pkg.directory, archive, pkg.integrity);
    assert.equal(comparison.tarballSHA512, pkg.archiveSha512);
    assert.deepEqual(comparison.files, pkg.files);
    assert.equal(pkg.allInstalledBytesEqualArchive, true);
  }
  for (const name of expected.keys())
    assert(seenExpected.has(name), `missing original provider ${name}`);
  assert.deepEqual(receipt.mapperExec, ["node", "./bin/vize", "content-mapper"]);
  const cliRoot = realpathSync(join(receipt.root, "node_modules/vize"));
  const installed = object(wholeJson(readFileSync(join(cliRoot, "package.json"))));
  assert.deepEqual(object(object(installed.typescript).contentMapper).exec, receipt.mapperExec);
  const graph = captureDeclarationGraph(cliRoot);
  assert.equal(graph.files.length, 10);
  assert.deepEqual(graph.files, receipt.sdk);
  for (const source of authority.sdk.filter((v) => v.path !== "package.json")) {
    const image = fileImage(join(cliRoot, source.path));
    assert.equal(image.bytes, source.bytes);
    assert.equal(image.sha256, source.sha256);
  }
  assert.equal(fileImage(receipt.cli.path).sha256, receipt.cli.sha256);
  assert.equal(fileImage(receipt.native.path).sha256, receipt.native.sha256);
  assert.equal(
    receipt.native.identityProbeOnly,
    true,
    "image probe is not a product child load witness",
  );
  assert.match(receipt.native.token, /^[a-f0-9]{32}$/u);
  let matched = 0;
  for (const item of array(authority.capsule.receipts)) {
    const row = object(item),
      raw = bytes(text(row.rawBase64));
    assert.equal(sha256(raw), row.sha256);
    assert.equal(String(object(row.origin).runId), authority.run);
    const providerReceipt = object(wholeJson(raw));
    assert.equal(providerReceipt.schema, "vize.platformVsix.v1");
    assert.equal(providerReceipt.source, authority.candidate);
    assert.equal(providerReceipt.version, authority.version);
    const provider = object(providerReceipt.provider);
    if (provider.targetPlatform !== `${recipe.platform}-${recipe.arch}`) continue;
    assert.equal(provider.source, authority.candidate);
    assert.equal(provider.releaseVersion, authority.version);
    assert.equal(provider.nativeBuildToken, receipt.native.token);
    const files = array(provider.files)
      .map(object)
      .filter((f) => f.path === provider.nodeFile);
    assert.equal(files.length, 1);
    assert.equal(files[0]?.sha256, receipt.native.sha256);
    assert.equal(files[0]?.bytes, receipt.native.bytes);
    matched++;
  }
  assert.equal(matched, 1, "exactly one genuine physical producer native receipt required");
}
