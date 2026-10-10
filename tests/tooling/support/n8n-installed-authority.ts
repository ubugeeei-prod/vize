import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import type { VizePublicRegistryInstallCollectorReceipt } from "../../../tools/support/release/public_install/authority-schema.ts";
import { rejectOverrides } from "../../../tools/support/release/public_acceptance/installed.ts";

export const sha256 = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
export interface InstalledCampaignPlan {
  schema: "vize.n8n.installed-campaign-v1";
  source: { C: string; H: string; tag: string; R: string; sourcePr: string };
  installReceipt: { path: string; sha256: string };
  collectorSha256: string;
}
interface PayloadPackage {
  name: string;
  version: string;
  resolved: string;
  integrity: string;
  packageDirectory: string;
  fileCount: number;
  allInstalledFileBytesEqualPublicArchive: true;
  files: Array<{ path: string; sha256: string; bytes: number }>;
}
interface PayloadManifest {
  schema: "vize-public-registry-payload-files-v1";
  installRoot: string;
  source: { C: string; H: string; tag: string; R: string };
  packages: PayloadPackage[];
}
const collectorPaths = [
  "tools/commands/release/npm/collect-public-install.rs",
  ...[
    "collect.py",
    "identity.py",
    "registry.py",
    "archive.py",
    "probe.py",
    "native-custody.cjs",
    "authority-schema.ts",
  ].map((name) => "tools/support/release/public_install/" + name),
].sort();

export function exactPath(value: string, owner?: string, directory = false): string {
  assert.equal(typeof value, "string");
  assert.ok(path.isAbsolute(value), "absolute canonical path required");
  assert.equal(fs.realpathSync(value), value, "path cannot redirect through a symlink");
  assert.equal(directory ? fs.statSync(value).isDirectory() : fs.statSync(value).isFile(), true);
  if (owner) assert.ok(value.startsWith(owner + path.sep), "path escaped its owner");
  return value;
}

export function validateCampaignPlan(value: InstalledCampaignPlan): void {
  assert.equal(value.schema, "vize.n8n.installed-campaign-v1");
  for (const commit of [value.source.C, value.source.H]) assert.match(commit, /^[0-9a-f]{40}$/u);
  assert.notEqual(value.source.C, value.source.H);
  assert.match(value.source.tag, /^v0\.[1-9][0-9]*\.0$/u);
  for (const id of [value.source.R, value.source.sourcePr]) assert.match(id, /^[1-9][0-9]*$/u);
  for (const digest of [value.installReceipt.sha256, value.collectorSha256])
    assert.match(digest, /^[0-9a-f]{64}$/u);
}

function fileSnapshot(directory: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const filename = path.join(directory, entry.name);
    assert.equal(entry.isSymbolicLink(), false, "installed payload cannot contain a symlink");
    if (entry.isDirectory()) files.push(...fileSnapshot(filename));
    else {
      assert.equal(entry.isFile(), true, "installed payload cannot contain a special file");
      files.push(filename);
    }
  }
  return files.sort();
}

/** Reuse a separately reviewed, SHA-pinned official collector receipt; never assign a workspace build identity. */
export function installedAuthority(plan: InstalledCampaignPlan, sourceRoot: string) {
  rejectOverrides();
  sourceRoot = exactPath(path.resolve(sourceRoot), undefined, true);
  validateCampaignPlan(plan);
  const receiptPath = exactPath(plan.installReceipt.path);
  const receiptBytes = fs.readFileSync(receiptPath);
  assert.equal(
    sha256(receiptBytes),
    plan.installReceipt.sha256,
    "reviewed install receipt changed",
  );
  const authority = JSON.parse(
    receiptBytes.toString("utf8"),
  ) as VizePublicRegistryInstallCollectorReceipt;
  assert.equal(authority.schema, "vize-public-registry-install-v1");
  assert.equal(authority.success, true);
  assert.equal(authority.version, plan.source.tag.slice(1));
  const source = { C: plan.source.C, H: plan.source.H, tag: plan.source.tag, R: plan.source.R };
  assert.deepEqual(authority.source, source);
  assert.equal(authority.collectorAuthority.schema, "vize-public-install-collector-v1");
  assert.equal(authority.collectorAuthority.sha256, plan.collectorSha256);
  assert.deepEqual(
    authority.collectorAuthority.files.map(({ path }) => path),
    collectorPaths,
  );
  const reviewedCollector = authority.collectorAuthority.files
    .map((file) => {
      assert.equal(
        sha256(fs.readFileSync(exactPath(path.join(sourceRoot, file.path), sourceRoot))),
        file.sha256,
      );
      return file.path + "\0" + file.sha256 + "\n";
    })
    .join("");
  assert.equal(sha256(reviewedCollector), plan.collectorSha256);
  const installRoot = exactPath(authority.installRoot, undefined, true);
  assert.notEqual(installRoot, sourceRoot);
  assert.equal(installRoot.startsWith(sourceRoot + path.sep), false);
  assert.equal(fs.existsSync(path.join(installRoot, ".git")), false);
  const lockPath = exactPath(authority.packageLockPath, installRoot);
  assert.equal(lockPath, path.join(installRoot, "package-lock.json"));
  const lockBytes = fs.readFileSync(lockPath);
  assert.equal(sha256(lockBytes), authority.packageLockSha256);
  const lock = JSON.parse(lockBytes.toString("utf8"));
  const consumer = JSON.parse(fs.readFileSync(path.join(installRoot, "package.json"), "utf8"));
  assert.equal(consumer.private, true);
  assert.equal(lock.lockfileVersion, 3);
  assert.deepEqual(lock.packages[""].dependencies, consumer.dependencies);
  const payloadPath = exactPath(authority.payloadManifestPath);
  const payloadBytes = fs.readFileSync(payloadPath);
  assert.equal(sha256(payloadBytes), authority.payloadManifestSha256);
  const payload = JSON.parse(payloadBytes.toString("utf8")) as PayloadManifest;
  assert.equal(payload.schema, "vize-public-registry-payload-files-v1");
  assert.equal(payload.installRoot, installRoot);
  assert.deepEqual(payload.source, source);
  assert.equal(new Set(payload.packages.map(({ name }) => name)).size, payload.packages.length);
  for (const item of authority.registry) {
    assert.equal(item.version, authority.version);
    assert.equal(item.provenanceSourceH, plan.source.H);
    assert.equal(item.provenanceR, plan.source.R);
    assert.equal(consumer.dependencies[item.name], authority.version);
    const packagePayload = payload.packages.find(({ name }) => name === item.name);
    assert.ok(packagePayload, "every public Vize package needs full payload custody");
    for (const key of ["name", "version", "resolved", "integrity"] as const)
      assert.equal(packagePayload[key], item[key]);
  }
  for (const name of ["vize", "@vizejs/native", authority.native.packageName])
    assert.ok(
      authority.registry.some((item) => item.name === name),
      name,
    );
  assert.equal(authority.native.version, authority.version);
  assert.equal(authority.native.successfulReturnObserved, true);
  assert.equal(authority.native.actualLoadedPath, authority.native.path);
  assert.equal(authority.cli.versionExitCode, 0);
  assert.equal(authority.cli.versionStdout, `vize ${authority.version}\n`);
  assert.equal(authority.cli.versionStderr, "");
  assert.equal(authority.custodyHook.configurationEnvironment, "VIZE_PUBLIC_NATIVE_CUSTODY");
  assert.equal(
    authority.custodyHook.sha256,
    sha256(
      fs.readFileSync(
        path.join(sourceRoot, "tools/support/release/public_install/native-custody.cjs"),
      ),
    ),
  );
  for (const value of Object.values(authority.sourceOverrides)) assert.equal(value, "");
  const expectedProvider = `@vizejs/native-${process.platform}-${process.arch}`;
  // The official collector currently owns the Darwin ARM64 campaign only.
  assert.equal(expectedProvider, "@vizejs/native-darwin-arm64");
  assert.equal(authority.native.packageName, expectedProvider);

  const recheck = () => {
    rejectOverrides();
    assert.equal(sha256(fs.readFileSync(receiptPath)), plan.installReceipt.sha256);
    assert.equal(sha256(fs.readFileSync(lockPath)), authority.packageLockSha256);
    assert.equal(sha256(fs.readFileSync(payloadPath)), authority.payloadManifestSha256);
    for (const item of payload.packages) {
      assert.equal(new URL(item.resolved).origin, "https://registry.npmjs.org");
      assert.match(item.integrity, /^sha512-[A-Za-z0-9+/]{86}==$/u);
      assert.equal(item.allInstalledFileBytesEqualPublicArchive, true);
      const directory = exactPath(item.packageDirectory, installRoot, true);
      assert.equal(directory, path.join(installRoot, "node_modules", item.name));
      assert.equal(item.fileCount, item.files.length);
      assert.ok(item.fileCount > 0);
      assert.deepEqual(fileSnapshot(directory), item.files.map(({ path }) => path).sort());
      const entry = lock.packages["node_modules/" + item.name];
      for (const key of ["version", "resolved", "integrity"] as const)
        assert.equal(entry[key], item[key]);
      assert.equal(entry.link, undefined);
      for (const file of item.files) {
        const bytes = fs.readFileSync(exactPath(file.path, directory));
        assert.equal(bytes.length, file.bytes);
        assert.equal(sha256(bytes), file.sha256);
      }
      const manifest = JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8"));
      assert.equal(manifest.name, item.name);
      assert.equal(manifest.version, item.version);
    }
    for (const [filename, digest, owner] of [
      [authority.cli.binPath, authority.cli.binSha256, path.join(installRoot, "node_modules/vize")],
      [
        authority.cli.distCliPath,
        authority.cli.distCliSha256,
        path.join(installRoot, "node_modules/vize"),
      ],
      [
        authority.native.path,
        authority.native.sha256,
        path.join(installRoot, "node_modules", authority.native.packageName),
      ],
      [authority.node.path, authority.node.sha256, undefined],
      [authority.custodyHook.path, authority.custodyHook.sha256, undefined],
      [authority.bundledCorsa.path, authority.bundledCorsa.sha256, installRoot],
    ] as const)
      assert.equal(sha256(fs.readFileSync(exactPath(filename, owner))), digest);
    assert.equal(path.extname(authority.native.path), ".node");
    assert.equal(
      sha256(fs.readFileSync(exactPath(authority.native.journalPath))),
      authority.native.journalSha256,
    );
    const cliManifest = JSON.parse(
      fs.readFileSync(path.join(installRoot, "node_modules/vize/package.json"), "utf8"),
    );
    assert.equal(
      authority.cli.binPath,
      path.join(installRoot, "node_modules/vize", cliManifest.bin.vize),
    );
    assert.equal(
      authority.cli.distCliPath,
      path.join(installRoot, "node_modules/vize/dist/cli.mjs"),
    );
  };
  recheck();
  return { authority, payload, recheck, receiptPath, receiptSha256: plan.installReceipt.sha256 };
}
