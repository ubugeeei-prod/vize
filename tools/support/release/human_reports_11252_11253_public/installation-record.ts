import assert from "node:assert/strict";
import path from "node:path";
import type { Fixture, InstallReceipt, Recipe, ReleaseAuthority } from "./model.ts";
import { manifestBytes } from "./installation.ts";
import { array, bytes, hash, object, sha256, text, wholeJson } from "./value.ts";
import {
  checkHostFacts,
  checkManagerFiles,
  validateOptionalAbsence,
} from "./optional-residency.ts";
const under = (root: string, file: string) =>
  file.startsWith(root + "/") && path.posix.normalize(file) === file;
export function validateRecordedInstall(
  row: InstallReceipt,
  fixture: Fixture,
  recipe: Recipe,
  authority: ReleaseAuthority,
): void {
  assert.equal(row.schema, "vize.humanReports11252_11253.install.v1");
  assert(path.posix.isAbsolute(row.root));
  assert.equal(path.posix.normalize(row.root), row.root);
  assert.equal(row.platform, recipe.platform);
  assert.equal(row.arch, recipe.arch);
  assert.equal(row.nodeVersion, recipe.node);
  assert.equal(row.pnpmVersion, recipe.pnpm);
  assert(row.kernel);
  assert.equal(
    row.native.packageName,
    recipe.platform === "darwin" ? "@vizejs/native-darwin-arm64" : "@vizejs/native-linux-x64-gnu",
  );
  const manifest = manifestBytes(fixture, authority.version, row.kind === "tsrs-supplement");
  assert(bytes(row.packageRawBase64).equals(manifest));
  assert.deepEqual(row.packageManifest, wholeJson(manifest));
  checkHostFacts(row.host, recipe.platform, recipe.arch);
  checkManagerFiles(row.managerFiles, bytes(row.lockRawBase64), manifest);
  assert.equal(sha256(bytes(row.lockRawBase64)), row.lockSha256);
  for (const image of [row.node, row.pnpm, row.cli, row.native, ...row.sdk]) {
    hash(image.sha256);
    assert(Number.isSafeInteger(image.bytes) && image.bytes > 0);
    assert(path.posix.isAbsolute(image.path));
  }
  const direct = new Map([
    ["vize", authority.version],
    ["@vizejs/native", authority.version],
    [row.native.packageName, authority.version],
    ["vue", recipe.vue],
    ["typescript", recipe.typescript],
  ]);
  if (recipe.vueTsc) direct.set("vue-tsc", recipe.vueTsc);
  if (row.kind === "tsrs-supplement") {
    assert.equal(recipe.issue, 11252);
    direct.set("tsrs", "0.20.1-ts7.1.0-beta");
  }
  const seen = new Set<string>(),
    sri = object(authority.capsule.npmSRI);
  for (const pkg of row.packages) {
    assert(under(row.root, pkg.directory));
    assert(!seen.has(pkg.directory));
    seen.add(pkg.directory);
    assert(pkg.files.length > 0);
    assert.equal(pkg.allInstalledBytesEqualArchive, true);
    const names = new Set<string>();
    for (const image of pkg.files) {
      assert(under(pkg.directory, image.path));
      assert(!names.has(image.path));
      names.add(image.path);
      hash(image.sha256);
      assert(Number.isSafeInteger(image.bytes) && image.bytes >= 0);
    }
    assert.equal(new URL(pkg.resolved).origin, "https://registry.npmjs.org");
    assert.match(pkg.archiveSha512, /^[a-f0-9]{128}$/u);
    assert.equal(Buffer.from(pkg.archiveSha512, "hex").toString("base64"), pkg.integrity.slice(7));
    assert(pkg.integrity.startsWith("sha512-"));
    if (pkg.name === "vize" || pkg.name.startsWith("@vizejs/native")) {
      const expected = object(sri[pkg.name]);
      assert.equal(expected.integrity, pkg.integrity);
      assert.equal(expected.tarball, pkg.resolved);
    }
  }
  for (const [name, version] of direct)
    assert(
      row.packages.some((p) => p.name === name && p.version === version),
      `missing exact recorded provider ${name}`,
    );
  assert(Array.isArray(row.declaredOptionalNotInstalled));
  for (const absent of row.declaredOptionalNotInstalled) {
    assert(!direct.has(absent.name), "a required provider cannot be recorded absent");
    validateOptionalAbsence(absent, row.root, bytes(row.lockRawBase64), row.host, row.packages);
  }
  const cli = row.packages.filter((p) => p.name === "vize" && p.version === authority.version);
  assert.equal(cli.length, 1);
  assert(under(cli[0]!.directory, row.cli.path));
  assert.deepEqual(row.mapperExec, ["node", "./bin/vize", "content-mapper"]);
  assert.equal(row.sdk.length, 10);
  assert.equal(new Set(row.sdk.map((f) => f.path)).size, 10);
  for (const sdk of authority.sdk.filter((v) => v.path !== "package.json")) {
    const found = row.sdk.filter((f) => f.path === cli[0]!.directory + "/" + sdk.path);
    assert.equal(found.length, 1);
    assert.equal(found[0]?.sha256, sdk.sha256);
    assert.equal(found[0]?.bytes, sdk.bytes);
  }
  const platform = row.packages.filter(
    (p) => p.name === row.native.packageName && p.version === authority.version,
  );
  assert.equal(platform.length, 1);
  assert(
    platform[0]?.files.some(
      (f) =>
        f.path === row.native.path &&
        f.sha256 === row.native.sha256 &&
        f.bytes === row.native.bytes,
    ),
  );
  assert.equal(row.native.identityProbeOnly, true);
  assert.match(row.native.token, /^[a-f0-9]{32}$/u);
  let matches = 0;
  for (const value of array(authority.capsule.receipts)) {
    const item = object(value),
      raw = bytes(text(item.rawBase64));
    assert.equal(sha256(raw), item.sha256);
    assert.equal(String(object(item.origin).runId), authority.run);
    const r = object(wholeJson(raw));
    assert.equal(r.schema, "vize.platformVsix.v1");
    assert.equal(r.source, authority.candidate);
    assert.equal(r.version, authority.version);
    const p = object(r.provider);
    assert.equal(p.source, authority.candidate);
    assert.equal(p.releaseVersion, authority.version);
    if (p.targetPlatform !== `${recipe.platform}-${recipe.arch}`) continue;
    assert.equal(p.nativeBuildToken, row.native.token);
    const native = array(p.files)
      .map(object)
      .filter((f) => f.path === p.nodeFile);
    assert.equal(native.length, 1);
    assert.equal(native[0]?.sha256, row.native.sha256);
    assert.equal(native[0]?.bytes, row.native.bytes);
    matches++;
  }
  assert.equal(matches, 1);
}
export function sameRelease(recorded: ReleaseAuthority, actual: ReleaseAuthority): void {
  for (const k of [
    "version",
    "candidate",
    "source",
    "workflowHead",
    "run",
    "capsule",
    "sdk",
  ] as const)
    assert.deepEqual(recorded[k], actual[k]);
  const a = object(recorded.produced),
    b = object(actual.produced);
  for (const k of [
    "record",
    "artifactId",
    "artifactName",
    "archiveBytes",
    "archiveSha256",
    "memberSha256",
    "memberBytes",
    "memberRawBase64",
  ])
    assert.deepEqual(a[k], b[k]);
}
