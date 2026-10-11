import assert from "node:assert/strict";
import { quickTunnelPin } from "./vrt_tunnel.ts";

const repository = "https://api.github.com/repos/cloudflare/cloudflared";
export const quickTunnelMetadata = {
  release: `${repository}/releases/${quickTunnelPin.releaseId}`,
  asset: `${repository}/releases/assets/${quickTunnelPin.assetId}`,
  tag: `${repository}/git/ref/tags/${quickTunnelPin.version}`,
  tagObject: `${repository}/git/tags/5eb7b904cb26e9ddc9b0ab490c8f1a0898155b34`,
} as const;

export function metadataObject(value: unknown): Record<string, unknown> {
  assert.ok(
    value && typeof value === "object" && !Array.isArray(value),
    "metadata object required",
  );
  return value as Record<string, unknown>;
}

function assertAsset(value: unknown) {
  const asset = metadataObject(value);
  for (const [key, expected] of Object.entries({
    id: quickTunnelPin.assetId,
    name: "cloudflared-linux-amd64",
    state: "uploaded",
    size: quickTunnelPin.bytes,
    digest: `sha256:${quickTunnelPin.sha256}`,
    browser_download_url: quickTunnelPin.url,
    url: quickTunnelMetadata.asset,
  }))
    assert.equal(asset[key], expected, `pinned asset ${key}`);
}

/** Exact public API identities and digest; mutable releases need this check at acquisition. */
export function assertQuickTunnelMetadata(
  values: Record<keyof typeof quickTunnelMetadata, unknown>,
) {
  const release = metadataObject(values.release);
  assert.equal(release.id, quickTunnelPin.releaseId);
  assert.equal(release.tag_name, quickTunnelPin.version);
  assert.equal(release.url, quickTunnelMetadata.release);
  assert.equal(
    release.html_url,
    `https://github.com/cloudflare/cloudflared/releases/tag/${quickTunnelPin.version}`,
  );
  assert.equal(release.draft, false);
  assert.equal(release.prerelease, false);
  assert.ok(
    typeof release.published_at === "string" && Number.isFinite(Date.parse(release.published_at)),
  );
  assert.equal(typeof release.immutable, "boolean", "record release mutability explicitly");
  assert.ok(Array.isArray(release.assets));
  const matches = release.assets
    .map(metadataObject)
    .filter((asset) => asset.name === "cloudflared-linux-amd64");
  assert.equal(matches.length, 1, "one pinned named release asset");
  assertAsset(matches[0]);
  assertAsset(values.asset);
  const tag = metadataObject(values.tag),
    target = metadataObject(tag.object);
  assert.equal(tag.ref, `refs/tags/${quickTunnelPin.version}`);
  assert.equal(tag.url, quickTunnelMetadata.tag.replace("/git/ref/", "/git/refs/"));
  assert.deepEqual(target, {
    type: "tag",
    sha: "5eb7b904cb26e9ddc9b0ab490c8f1a0898155b34",
    url: quickTunnelMetadata.tagObject,
  });
  const annotation = metadataObject(values.tagObject);
  assert.equal(annotation.sha, target.sha);
  assert.equal(annotation.tag, quickTunnelPin.version);
  assert.equal(annotation.url, quickTunnelMetadata.tagObject);
  assert.deepEqual(metadataObject(annotation.object), {
    type: "commit",
    sha: quickTunnelPin.source,
    url: `${repository}/git/commits/${quickTunnelPin.source}`,
  });
  return { immutable: release.immutable, publishedAt: release.published_at };
}

/** Only the official pinned browser URL may redirect to GitHub's release-asset host. */
export function assertQuickTunnelDownloadLocation(value: string, initial: boolean) {
  const url = new URL(value);
  assert.ok(
    url.protocol === "https:" && !url.username && !url.password && !url.port && !url.hash,
    "HTTPS asset location without userinfo, port or fragment required",
  );
  if (initial) assert.equal(url.href, quickTunnelPin.url, "exact pinned initial asset URL");
  else
    assert.equal(
      url.hostname,
      "release-assets.githubusercontent.com",
      "official release-asset host only",
    );
  return url.href;
}

/** Existing authenticated prepare output must describe this exact official frozen release. */
export function assertTunnelReleaseCustody(
  source: Record<string, string>,
  preparation: unknown,
  publication: unknown,
) {
  const prepared = metadataObject(preparation),
    published = metadataObject(publication);
  assert.deepEqual(prepared.source, source);
  assert.equal(prepared.version, source.tag.slice(1));
  assert.equal(published.schema, "vize-public-release-acceptance-v1");
  assert.equal(published.success, true);
  const publishedSource = metadataObject(published.source),
    github = metadataObject(published.github);
  for (const [key, value] of Object.entries(source))
    assert.equal(publishedSource[key], value, `official release ${key}`);
  assert.equal(github.target, source.head);
  assert.equal(github.runStatus, "completed");
  assert.equal(github.runConclusion, "success");
  assert.ok(Number.isSafeInteger(github.releaseId) && Number(github.releaseId) > 0);
}
