import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { repository, sha256 } from "./custody.mjs";

export const archiveName = "vize-x86_64-unknown-linux-gnu.tar.gz";

export function selectAssets(release, tag) {
  assert(Array.isArray(release.assets));
  return [archiveName, `${archiveName}.sha256`].map((name) => {
    const matches = release.assets.filter((asset) => asset.name === name);
    assert.equal(matches.length, 1, `exactly one official asset required: ${name}`);
    const asset = matches[0];
    assert.equal(asset.state, "uploaded");
    assert.equal(
      asset.browser_download_url,
      `https://github.com/${repository}/releases/download/${tag}/${name}`,
    );
    assert.match(asset.digest ?? "", /^sha256:[0-9a-f]{64}$/);
    assert(Number.isSafeInteger(asset.size) && asset.size > 0);
    assert(asset.size <= (name === archiveName ? 256 * 1024 * 1024 : 16 * 1024));
    return asset;
  });
}

export function verifyAssets(assets, archive, checksum) {
  assert.equal(archive.length, assets[0].size);
  assert.equal(checksum.length, assets[1].size);
  assert.equal(`sha256:${sha256(archive)}`, assets[0].digest, "official archive digest mismatch");
  assert.equal(`sha256:${sha256(checksum)}`, assets[1].digest, "official checksum digest mismatch");
  assert.equal(
    checksum.toString("utf8"),
    `${sha256(archive)}  ${archiveName}\n`,
    "exact archive checksum sidecar required",
  );
}

const loadedIdentities = new WeakMap();

export function assertLoadedBinary(binary, receipt, consumer) {
  const real = fs.realpathSync(binary);
  assert.equal(
    real,
    path.join(fs.realpathSync(consumer), "distribution", "vize"),
    "foreign loaded binary refused",
  );
  const stat = fs.lstatSync(binary, { bigint: true });
  assert(stat.isFile() && !stat.isSymbolicLink(), "loaded CLI must be a regular file");
  assert.equal(stat.size, BigInt(receipt.bytes));
  assert.equal(stat.mode & 0o777n, BigInt(receipt.mode));
  const identity = [real, stat.dev, stat.ino, stat.size, stat.mode, stat.mtimeNs, stat.ctimeNs];
  const prior = loadedIdentities.get(receipt);
  if (prior)
    assert.deepEqual(identity, prior, "loaded public CLI identity changed during observation");
  // Filesystem timestamps can collide during same-size rewrites. Every consumer
  // boundary must verify the complete bytes, even when all metadata is unchanged.
  assert.equal(
    sha256(fs.readFileSync(binary)),
    receipt.sha256,
    "loaded public CLI identity changed: bytes differ from archived member",
  );
  if (!prior) loadedIdentities.set(receipt, identity);
}

export async function acquireDistribution(release, tag, consumer) {
  const assets = selectAssets(release, tag);
  const payloads = [];
  for (const asset of assets) {
    const response = await fetch(asset.browser_download_url, {
      signal: AbortSignal.timeout(120_000),
    });
    assert(response.ok, `public asset download failed: HTTP ${response.status}`);
    const chunks = [];
    let count = 0;
    for await (const chunk of response.body) {
      count += chunk.length;
      assert(count <= asset.size, "public asset exceeds declared bytes");
      chunks.push(Buffer.from(chunk));
    }
    payloads.push(Buffer.concat(chunks));
  }
  verifyAssets(assets, ...payloads);
  for (let i = 0; i < assets.length; i++)
    fs.writeFileSync(path.join(consumer, assets[i].name), payloads[i], { flag: "wx" });
  const archive = path.join(consumer, archiveName);
  const result = spawnSync(
    process.execPath,
    [
      fileURLToPath(new URL("./archive.ts", import.meta.url)),
      archive,
      path.join(consumer, "distribution"),
    ],
    {
      encoding: "utf8",
      timeout: 60_000,
      maxBuffer: 1024 * 1024,
    },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  const member = JSON.parse(result.stdout);
  assertLoadedBinary(member.binary, member, consumer);
  return {
    assets,
    archiveSha256: sha256(payloads[0]),
    checksumSha256: sha256(payloads[1]),
    member,
  };
}
