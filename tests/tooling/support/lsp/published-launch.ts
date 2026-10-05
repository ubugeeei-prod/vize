import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "../../../differential/manifest.mjs";
import type { VerifiedLspLaunch } from "./launch.ts";

export type PublishedAuthority = {
  schema: "vize.original400.published.authority";
  version: 1;
  releaseVersion: string;
  cut: { sha: string; tree: string; manifestSha256: string };
  asset: { id: number; size: number; sha256: string };
  sourceArtifact: {
    id: number;
    run: number;
    attempt: number;
    size: number;
    sha256: string;
    driverRevision: string;
  };
};
export type PublishedReceipt = {
  schema: "vize.original400.published.installation";
  version: 1;
  authority: PublishedAuthority;
  release: { draft: boolean; published_at: string; tag_name: string };
  tagCommit: { sha: string; commit: { tree: { sha: string } } };
  asset: { name: string; id: number; size: number; digest: string; browser_download_url: string };
  downloadSha256: string;
  installed: { path: string; sha256: string; archiveMember: string };
};
export type VerifiedPublishedLspLaunch = Omit<VerifiedLspLaunch, "receipt"> & {
  authority: "published-release";
  receipt: PublishedReceipt;
};

/** A public release payload has publication custody, never a Cargo build receipt. */
export function validatePublishedLaunch(launch: VerifiedPublishedLspLaunch): void {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(launch.authority, "published-release");
  const receipt = launch.receipt;
  assert.equal(receipt.schema, "vize.original400.published.installation");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.release.draft, false);
  assert.ok(receipt.release.published_at);
  assert.equal(receipt.release.tag_name, `v${receipt.authority.releaseVersion}`);
  assert.equal(receipt.tagCommit.sha, receipt.authority.cut.sha);
  assert.equal(receipt.tagCommit.commit.tree.sha, receipt.authority.cut.tree);
  assert.equal(receipt.asset.name, "vize-x86_64-unknown-linux-gnu.tar.gz");
  assert.equal(receipt.asset.id, receipt.authority.asset.id);
  assert.equal(receipt.asset.size, receipt.authority.asset.size);
  assert.equal(receipt.asset.digest, `sha256:${receipt.authority.asset.sha256}`);
  assert.equal(receipt.downloadSha256, receipt.authority.asset.sha256);
  assert.equal(fs.realpathSync(launch.binary), launch.binary);
  assert.equal(launch.expected.binaryPath, launch.binary);
  assert.equal(launch.expected.sourceRevision, receipt.authority.cut.sha);
  assert.equal(launch.expected.cliVersion, `vize ${receipt.authority.releaseVersion}`);
  const bytes = fs.readFileSync(launch.binary);
  assert.equal(bytes.subarray(0, 4).toString("hex"), "7f454c46");
  assert.equal(bytes[4], 2);
  assert.equal(bytes[5], 1);
  assert.equal(bytes.readUInt16LE(18), 62);
  assert.equal(sha256(bytes), launch.expected.binarySha256);
  assert.equal(launch.expected.binarySha256, receipt.installed.sha256);
  assert.equal(launch.binary, receipt.installed.path);
  assert.equal(receipt.installed.archiveMember, "vize");
}

export function verifyPublishedLaunch(
  receiptPath: string,
  digest: string,
): VerifiedPublishedLspLaunch {
  assert.match(digest, /^[a-f0-9]{64}$/u);
  const bytes = fs.readFileSync(receiptPath);
  assert.equal(sha256(bytes), digest, "the owned installation receipt must remain exact");
  const receipt = JSON.parse(bytes.toString("utf8"));
  const binary = fs.realpathSync(receipt.installed.path);
  const launch: VerifiedPublishedLspLaunch = {
    authority: "published-release",
    binary,
    expected: {
      sourceRevision: receipt.authority.cut.sha,
      binaryPath: binary,
      binarySha256: receipt.installed.sha256,
      cliVersion: `vize ${receipt.authority.releaseVersion}`,
    },
    receipt,
  };
  validatePublishedLaunch(launch);
  const probe = spawnSync(binary, ["--version"]);
  launch.versionProbe = {
    exitStatus: probe.status,
    signal: probe.signal,
    stdoutBase64: probe.stdout?.toString("base64") ?? "",
    stderrBase64: probe.stderr?.toString("base64") ?? "",
    processError: probe.error?.message ?? null,
  };
  fs.writeFileSync(
    `${receiptPath}.version-probe.json`,
    `${JSON.stringify(launch.versionProbe, null, 2)}\n`,
  );
  assert.equal(probe.error, undefined);
  assert.equal(probe.signal, null);
  assert.equal(probe.status, 0);
  assert.equal(probe.stdout.toString().trim(), launch.expected.cliVersion);
  assert.equal(path.resolve(binary), binary);
  return launch;
}
