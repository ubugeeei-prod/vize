import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export function marketplaceInventory(publisher: string, name: string) {
  return {
    publisher: { publisherName: publisher, publisherId: "inert-publisher-id" },
    extensionName: name,
    extensionId: "inert-extension-id",
    // These are PublishedExtension flags, not the separate ExtensionQueryFlags enum.
    flags: 512 | 65536,
    versions: [
      { version: "0.437.0" },
      { version: "0.437.0", targetPlatform: "linux-x64" },
      {
        version: "0.437.1",
        properties: [{ key: "Microsoft.VisualStudio.Code.PreRelease", value: "true" }],
      },
    ],
  };
}

type Reply = { status: number; body: unknown; headers?: Record<string, string>; url?: string };
/** Inert version inventories and transport controls; no live Marketplace availability claim. */
export async function marketplaceControls(
  target: { publisher: string; name: string; version: string },
  url: string,
  reject: (url: string, reply: Reply, pattern: RegExp) => Promise<void>,
  accept: (
    body: unknown,
  ) => Promise<{ status: number; evidence: string; response: unknown; responseSha256: string }>,
) {
  const full = marketplaceInventory(target.publisher, target.name);
  const receipt = await accept(full);
  assert.equal(receipt.status, 200);
  assert.equal(receipt.evidence, "complete-marketplace-version-inventory");
  assert.deepEqual(receipt.response, full, "every returned version/platform row is retained");
  assert.equal(
    receipt.responseSha256,
    createHash("sha256").update(JSON.stringify(full)).digest("hex"),
  );
  const large = { ...full, displayName: "x".repeat(70_000) };
  assert.deepEqual(
    (await accept(large)).response,
    large,
    "bounded complete inventories may exceed 64KiB",
  );
  for (const body of [
    { ...full, extensionId: "" },
    { ...full, extensionName: "foreign" },
    { ...full, publisher: { ...full.publisher, publisherName: "foreign" } },
    { ...full, publisher: { publisherName: target.publisher } },
    { ...full, versions: undefined },
    { ...full, versions: [] },
    { ...full, versions: [null] },
    { ...full, versions: [{ version: 123 }] },
    { ...full, versions: [{ version: "latest" }] },
    { ...full, versions: [{ version: "0.437.0", targetPlatform: 123 }] },
    { ...full, versions: [full.versions[0], full.versions[0]] },
    { ...full, versions: [{ version: target.version }] },
    { ...full, versions: [{ version: target.version, targetPlatform: "darwin-arm64" }] },
    { ...full, versionCount: full.versions.length + 1 },
    { ...full, totalVersions: full.versions.length + 1 },
    { ...full, pagingToken: "next" },
    { ...full, continuationToken: "next" },
    { ...full, truncated: true },
    { ...full, latestOnly: true },
    { ...full, includeLatestVersionOnly: true },
    // An arbitrary former version-filter fallback has no complete PublishedExtension identity.
    {
      publisher: { publisherName: target.publisher },
      extensionName: target.name,
      versions: [{ version: "9.9.9" }],
    },
  ]) {
    await reject(url, { status: 200, body }, /Marketplace|missing response/);
  }
  await accept({ ...full, versionCount: full.versions.length });
  await reject(
    url,
    { status: 200, body: full, headers: { "x-ms-continuationtoken": "next" } },
    /continuation/,
  );
  await reject(
    url,
    { status: 200, body: full, url: `${url}&version=${target.version}` },
    /URL changed/,
  );
  await reject(url, { status: 200, body: "x".repeat(2_097_153) }, /2097152 bytes/);
  await reject(
    url,
    { status: 200, body: full, headers: { "content-length": "2097153" } },
    /2097152 bytes/,
  );
  await reject(url, { status: 404, body: "x".repeat(65_537) }, /65536 bytes/);
}
