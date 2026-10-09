import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export function marketplaceInventory(publisher: string, name: string) {
  return {
    publisher: { publisherName: publisher, publisherId: "inert-publisher-id" },
    extensionName: name,
    extensionId: "inert-extension-id",
    // These are PublishedExtension flags, not the separate ExtensionQueryFlags enum.
    flags: "validated, public",
    versions: [
      { version: "0.437.0", flags: "validated" },
      { version: "0.437.0", targetPlatform: "linux-x64" },
      {
        version: "0.437.1",
        properties: [{ key: "Microsoft.VisualStudio.Code.PreRelease", value: "true" }],
      },
    ],
  };
}

export function marketplaceEnvelope(extension: unknown) {
  return {
    results: [
      {
        extensions: [extension],
        pagingToken: null,
        resultMetadata: [
          { metadataType: "ResultCount", metadataItems: [{ name: "TotalCount", count: 1 }] },
        ],
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
  const extension = marketplaceInventory(target.publisher, target.name);
  const full = marketplaceEnvelope(extension);
  const receipt = await accept(full);
  assert.equal(receipt.status, 200);
  assert.equal(receipt.evidence, "complete-marketplace-version-inventory");
  assert.deepEqual(receipt.response, full, "every returned version/platform row is retained");
  assert.equal(
    receipt.responseSha256,
    createHash("sha256").update(JSON.stringify(full)).digest("hex"),
  );
  const large = marketplaceEnvelope({
    ...extension,
    displayName: "x".repeat(70_000),
  } as typeof extension);
  assert.deepEqual(
    (await accept(large)).response,
    large,
    "bounded complete inventories may exceed 64KiB",
  );
  for (const body of [
    { ...extension, extensionId: "" },
    { ...extension, extensionName: "foreign" },
    { ...extension, publisher: { ...extension.publisher, publisherName: "foreign" } },
    { ...extension, publisher: { publisherName: target.publisher } },
    { ...extension, versions: undefined },
    { ...extension, versions: [] },
    { ...extension, versions: [null] },
    { ...extension, versions: [{ version: 123 }] },
    { ...extension, versions: [{ version: "latest" }] },
    { ...extension, versions: [{ version: "0.437.0", targetPlatform: 123 }] },
    { ...extension, versions: [extension.versions[0], extension.versions[0]] },
    { ...extension, versions: [{ version: target.version }] },
    { ...extension, versions: [{ version: target.version, targetPlatform: "darwin-arm64" }] },
    { ...extension, versionCount: extension.versions.length + 1 },
    { ...extension, totalVersions: extension.versions.length + 1 },
    { ...extension, pagingToken: "next" },
    { ...extension, continuationToken: "next" },
    { ...extension, truncated: true },
    { ...extension, latestOnly: true },
    { ...extension, includeLatestVersionOnly: true },
    // An arbitrary former version-filter fallback has no complete PublishedExtension identity.
    {
      publisher: { publisherName: target.publisher },
      extensionName: target.name,
      versions: [{ version: "9.9.9" }],
    },
  ]) {
    await reject(
      url,
      { status: 200, body: marketplaceEnvelope(body as typeof extension) },
      /Marketplace|missing response/,
    );
  }
  await accept(
    marketplaceEnvelope({
      ...extension,
      versionCount: extension.versions.length,
    } as typeof extension),
  );
  await accept(marketplaceEnvelope({ ...extension, flags: 512 | 65536 }));
  for (const body of [
    {},
    { results: [] },
    { results: [full.results[0], full.results[0]] },
    { results: [{ ...full.results[0], pagingToken: "next" }] },
    { results: [{ ...full.results[0], extensions: [] }] },
    { results: [{ ...full.results[0], extensions: [extension, extension] }] },
    { results: [{ ...full.results[0], resultMetadata: [] }] },
    {
      results: [
        {
          ...full.results[0],
          resultMetadata: [
            { metadataType: "ResultCount", metadataItems: [{ name: "TotalCount", count: 2 }] },
          ],
        },
      ],
    },
    {
      results: [
        {
          ...full.results[0],
          resultMetadata: [...full.results[0].resultMetadata, ...full.results[0].resultMetadata],
        },
      ],
    },
    {
      results: [
        {
          ...full.results[0],
          resultMetadata: [
            {
              metadataType: "ResultCount",
              metadataItems: [
                { name: "TotalCount", count: 1 },
                { name: "TotalCount", count: 1 },
              ],
            },
          ],
        },
      ],
    },
  ])
    await reject(url, { status: 200, body }, /Marketplace/);
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
  await reject(url, { status: 404, body: "not found" }, /HTTP 200/);
}
