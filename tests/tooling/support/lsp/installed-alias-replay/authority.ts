import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import type { VizePublicRegistryInstallAuthority } from "./authority-schema.ts";
import { recheckPublicPayload } from "./payload.ts";
import { RawGitRepository } from "./raw-git.ts";
import { digest, exactFile, sourceOverrideKeys, verifyNativeJournal } from "./custody.ts";

/** The caller supplies a reviewed concrete installation receipt and its exact digest. */
export function loadPublicAuthority(
  receiptPath: string,
  receiptSha256: string,
  repositoryRoot: string,
  signedSourceMerge: string,
): VizePublicRegistryInstallAuthority {
  const bytes = fs.readFileSync(exactFile(receiptPath, receiptSha256));
  const authority: VizePublicRegistryInstallAuthority = JSON.parse(bytes.toString("utf8"));
  assert.equal(authority.schema, "vize-public-registry-install-v1");
  assert.equal(authority.success, true);
  for (const key of sourceOverrideKeys) {
    assert.equal(authority.sourceOverrides[key], "");
    assert.equal(
      process.env[key] ?? "",
      "",
      "ambient source provider override is forbidden before any probe",
    );
  }
  assert.equal(
    process.env.VIZE_PUBLIC_NATIVE_CUSTODY ?? "",
    "",
    "outer custody configuration cannot be reused",
  );
  assert.match(authority.version, /^\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?$/u);
  for (const sha of [signedSourceMerge, authority.source.C, authority.source.H])
    assert.match(sha, /^[a-f0-9]{40}$/u);
  assert.equal(authority.source.tag, `v${authority.version}`);
  assert.match(authority.source.R, /^\d+$/u);
  const repository = new RawGitRepository(repositoryRoot);
  for (const destination of [authority.source.C, authority.source.H]) {
    assert.ok(
      repository.includes(signedSourceMerge, destination),
      "the actual signed source merge must be included in this immutable public cut",
    );
  }
  assert.ok(path.isAbsolute(authority.installRoot));
  assert.equal(fs.realpathSync(authority.installRoot), authority.installRoot);
  exactFile(authority.packageLockPath, authority.packageLockSha256, authority.installRoot);
  exactFile(authority.node.path, authority.node.sha256);
  const nodeProbe = spawnSync(authority.node.path, ["--version"], { encoding: "utf8" });
  assert.equal(nodeProbe.error, undefined);
  assert.equal(nodeProbe.status, 0);
  assert.equal(nodeProbe.signal, null);
  assert.equal(nodeProbe.stdout.trim(), authority.node.version);
  exactFile(authority.cli.binPath, authority.cli.binSha256, authority.installRoot);
  exactFile(authority.cli.distCliPath, authority.cli.distCliSha256, authority.installRoot);
  exactFile(authority.native.path, authority.native.sha256, authority.installRoot);
  exactFile(authority.custodyHook.path, authority.custodyHook.sha256);
  exactFile(authority.bundledCorsa.path, authority.bundledCorsa.sha256, authority.installRoot);
  assert.equal(authority.bundledCorsa.allInstalledFileBytesEqualPublicArchive, true);
  assert.equal(authority.bundledCorsa.declaredVersion, authority.bundledCorsa.version);
  recheckPublicPayload(authority);
  assert.equal(authority.custodyHook.configurationEnvironment, "VIZE_PUBLIC_NATIVE_CUSTODY");
  assert.equal(authority.native.actualLoadedPath, authority.native.path);
  assert.equal(authority.native.successfulReturnObserved, true);
  assert.equal(authority.native.version, authority.version);
  assert.equal(authority.cli.versionExitCode, 0);
  assert.equal(authority.cli.versionStdout.trim(), `vize ${authority.version}`);
  assert.equal(authority.cli.versionStderr, "");
  for (const key of sourceOverrideKeys) {
    assert.equal(authority.sourceOverrides[key], "");
    assert.equal(process.env[key] ?? "", "", "ambient source provider override is forbidden");
  }
  exactFile(authority.native.journalPath, authority.native.journalSha256);
  verifyNativeJournal(authority.native.journalPath, authority);
  const lock = JSON.parse(fs.readFileSync(authority.packageLockPath, "utf8"));
  assert.ok(Array.isArray(authority.registry) && authority.registry.length > 0);
  const names = new Set<string>();
  for (const record of authority.registry) {
    assert.ok(!names.has(record.name), "duplicate registry package authority");
    names.add(record.name);
    assert.equal(record.version, authority.version);
    assert.equal(record.provenanceSourceH, authority.source.H);
    assert.equal(record.provenanceR, authority.source.R);
    assert.ok(record.resolved.startsWith("https://registry.npmjs.org/"));
    assert.match(record.integrity, /^sha512-[A-Za-z0-9+/]+=*$/u);
    const installed = lock.packages[`node_modules/${record.name}`];
    assert.equal(installed.version, record.version);
    assert.equal(installed.resolved, record.resolved);
    assert.equal(installed.integrity, record.integrity);
    const manifest = JSON.parse(
      fs.readFileSync(
        path.join(authority.installRoot, "node_modules", record.name, "package.json"),
        "utf8",
      ),
    );
    assert.equal(manifest.name, record.name);
    assert.equal(manifest.version, record.version);
  }
  assert.ok(
    names.has("vize") && names.has("@vizejs/native") && names.has(authority.native.packageName),
  );
  const cliManifest = JSON.parse(
    fs.readFileSync(path.join(authority.installRoot, "node_modules/vize/package.json"), "utf8"),
  );
  assert.equal(
    cliManifest.optionalDependencies[authority.bundledCorsa.packageName],
    authority.bundledCorsa.version,
  );
  const corsa = lock.packages[`node_modules/${authority.bundledCorsa.packageName}`];
  assert.equal(corsa.version, authority.bundledCorsa.version);
  assert.equal(corsa.resolved, authority.bundledCorsa.resolved);
  assert.equal(corsa.integrity, authority.bundledCorsa.integrity);
  return authority;
}

export function recheckPublicBytes(authority: VizePublicRegistryInstallAuthority): void {
  exactFile(authority.node.path, authority.node.sha256);
  exactFile(authority.cli.binPath, authority.cli.binSha256, authority.installRoot);
  exactFile(authority.cli.distCliPath, authority.cli.distCliSha256, authority.installRoot);
  exactFile(authority.native.path, authority.native.sha256, authority.installRoot);
  exactFile(authority.custodyHook.path, authority.custodyHook.sha256);
  exactFile(authority.bundledCorsa.path, authority.bundledCorsa.sha256, authority.installRoot);
  assert.equal(authority.bundledCorsa.allInstalledFileBytesEqualPublicArchive, true);
  assert.equal(authority.bundledCorsa.declaredVersion, authority.bundledCorsa.version);
  recheckPublicPayload(authority);
  exactFile(authority.packageLockPath, authority.packageLockSha256, authority.installRoot);
}

export const authorityDigest = (authority: VizePublicRegistryInstallAuthority): string =>
  digest(JSON.stringify(authority));
