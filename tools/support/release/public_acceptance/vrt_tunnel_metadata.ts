import assert from "node:assert/strict";
import { quickTunnelPin } from "./vrt_tunnel.ts";
import { assertFresh } from "../consumer_recovery/policy.ts";

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
  const { recoveryRun, ...original } = source;
  for (const [key, value] of Object.entries(original))
    assert.equal(publishedSource[key], value, `official release ${key}`);
  assert.equal(github.target, source.head);
  assert.equal(github.runStatus, "completed");
  assert.ok(Number.isSafeInteger(github.releaseId) && Number(github.releaseId) > 0);
  if (recoveryRun === undefined) {
    assert.equal(github.runConclusion, "success", "successful original Release required");
    assert.equal(github.consumerRecovery, undefined, "explicit recovery run required");
    return;
  }
  // prepare/verifyGithub already authenticates the signature, live run and all six consumers.
  // Bind that private publication output here; never turn an arbitrary failed R into success.
  assert.match(recoveryRun, /^[1-9][0-9]*$/u, "positive exact recovery run required");
  assert.ok(Number.isSafeInteger(Number(recoveryRun)), "safe recovery run required");
  assert.equal(github.runConclusion, "failure", "original Release failure must remain visible");
  const receipt = metadataObject(github.consumerRecovery),
    identity = metadataObject(receipt.identity),
    recovered = metadataObject(receipt.recovery);
  assert.ok(
    typeof identity.attempt === "number" &&
      Number.isSafeInteger(identity.attempt) &&
      identity.attempt > 0,
    "positive original Release attempt required",
  );
  assertFresh(receipt, {
    cut: source.cut,
    head: source.head,
    tag: source.tag,
    sourcePr: source.sourcePr,
    run: source.run,
    attempt: identity.attempt,
  });
  assert.equal(recovered.run, recoveryRun, "exact authenticated recovery run required");
  assert.ok(
    typeof recovered.attempt === "number" &&
      Number.isSafeInteger(recovered.attempt) &&
      recovered.attempt > 0,
    "positive recovery attempt required",
  );
  assert.ok(typeof recovered.policySha === "string", "recovery policy SHA required");
  assert.match(recovered.policySha, /^[0-9a-f]{40}$/u, "full recovery policy SHA required");
  assert.ok(Array.isArray(github.jobs), "authenticated original Release jobs required");
  const failed = github.jobs.map(metadataObject).filter((job) => job.conclusion === "failure");
  assert.equal(failed.length, 1, "one admitted original consumer failure required");
  assert.equal(String(failed[0].run), source.run, "original failed job run required");
  assert.equal(failed[0].head, source.head, "original failed job H required");
  assert.equal(failed[0].status, "completed", "terminal original failed job required");
  assert.equal(failed[0].attempt, identity.attempt, "exact original failed attempt required");
}
