import { createHash } from "node:crypto";

export const marketplaceQueryUrl =
  "https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery";
export const marketplaceQueryAccept = "application/json;api-version=3.0-preview.1";
export const marketplaceInventoryLimit = 2 * 1024 * 1024;

function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
function object(value: unknown): Record<string, unknown> {
  requireValue(
    value && typeof value === "object" && !Array.isArray(value),
    "Marketplace response object required",
  );
  return value as Record<string, unknown>;
}

/** Name=7 and IncludeVersions=1; no latest-only, platform or publication-state filter. */
export function marketplaceQuery(name: string) {
  requireValue(/^[a-z0-9-]+\.[a-z0-9-]+$/.test(name), "Marketplace exact identity required");
  return {
    filters: [{ criteria: [{ filterType: 7, value: name }], pageNumber: 1, pageSize: 1 }],
    flags: 1,
  };
}

function complete(value: Record<string, unknown>) {
  for (const key of ["pagingToken", "continuationToken", "nextLink", "@odata.nextLink"]) {
    requireValue(value[key] == null || value[key] === "", `Marketplace continuation: ${key}`);
  }
  for (const key of ["truncated", "hasMore", "latestOnly", "includeLatestVersionOnly"]) {
    requireValue(!(key in value), `Marketplace partial inventory: ${key}`);
  }
}

/** Validate the full public-query envelope before selecting the one exact extension. */
export function marketplaceExtension(name: string, value: unknown, headers = new Headers()) {
  // The official VS Code client reads results[].extensions and ResultCount/TotalCount.
  // https://github.com/microsoft/vscode/blob/main/src/vs/platform/extensionManagement/common/extensionGalleryService.ts
  // https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/GalleryInterfaces.ts
  const body = object(value);
  complete(body);
  const results = body.results;
  requireValue(
    Array.isArray(results) && results.length === 1,
    "Marketplace one query result required",
  );
  const result = object(results[0]);
  complete(result);
  const extensions = result.extensions;
  requireValue(
    Array.isArray(extensions) && extensions.length === 1,
    "Marketplace one exact extension required",
  );
  const metadata = result.resultMetadata;
  requireValue(Array.isArray(metadata), "Marketplace complete result count required");
  const counts = metadata.map(object).filter((row) => row.metadataType === "ResultCount");
  requireValue(counts.length === 1, "Marketplace unambiguous result count required");
  const items = counts[0].metadataItems;
  requireValue(Array.isArray(items), "Marketplace total count required");
  const totals = items.map(object).filter((row) => row.name === "TotalCount");
  requireValue(totals.length === 1 && totals[0].count === 1, "Marketplace total count differs");
  for (const key of [
    "x-ms-continuationtoken",
    "x-ms-continuation-token",
    "x-vss-continuationtoken",
    "link",
  ]) {
    requireValue(!headers.has(key), "Marketplace inventory continuation header refused");
  }
  const extension = object(extensions[0]);
  complete(extension);
  const [publisherName, extensionName] = name.split(".");
  const publisher = object(extension.publisher);
  requireValue(
    publisher.publisherName === publisherName &&
      extension.extensionName === extensionName &&
      typeof publisher.publisherId === "string" &&
      publisher.publisherId.length > 0 &&
      typeof extension.extensionId === "string" &&
      extension.extensionId.length > 0,
    "Marketplace inventory identity missing or changed",
  );
  const versions = extension.versions;
  requireValue(
    Array.isArray(versions) && versions.length > 0 && versions.length <= 10_000,
    "Marketplace complete nonempty versions required",
  );
  for (const key of ["versionCount", "totalVersions", "totalCount", "count"]) {
    requireValue(
      !(key in extension) || extension[key] === versions.length,
      `Marketplace inventory count differs: ${key}`,
    );
  }
  const seen = new Set<string>();
  for (const value of versions) {
    const row = object(value);
    requireValue(
      typeof row.version === "string" &&
        /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(row.version) &&
        (row.targetPlatform == null || typeof row.targetPlatform === "string"),
      "Marketplace malformed version/platform row",
    );
    const key = `${row.version}\0${row.targetPlatform ?? ""}`;
    requireValue(!seen.has(key), "Marketplace duplicate version/platform row");
    seen.add(key);
  }
  return extension;
}

/** One bounded anonymous read of the complete public inventory, preserving original JSON bytes. */
export async function readMarketplace(name: string, fetchImpl = globalThis.fetch) {
  const query = marketplaceQuery(name);
  const controller = new AbortController();
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(() => {
      controller.abort();
      reject(new Error(`Marketplace request timed out after 30000ms: ${marketplaceQueryUrl}`));
    }, 30_000);
  });
  try {
    return await Promise.race([
      timeout,
      (async () => {
        const response = await fetchImpl(marketplaceQueryUrl, {
          method: "POST",
          body: JSON.stringify(query),
          credentials: "omit",
          redirect: "manual",
          cache: "no-store",
          signal: controller.signal,
          headers: {
            Accept: marketplaceQueryAccept,
            "Content-Type": "application/json",
            "Cache-Control": "no-cache",
            Pragma: "no-cache",
            "User-Agent": "vize-public-marketplace-inventory",
          },
        });
        requireValue(
          response.status === 200,
          `Marketplace requires public HTTP 200, received ${response.status}`,
        );
        requireValue(
          !response.redirected &&
            response.url === marketplaceQueryUrl &&
            !response.headers.has("location"),
          "Marketplace response URL changed or redirects refused",
        );
        requireValue(
          /^application\/(?:json|[a-z0-9.+-]+\+json)(?:;|$)/i.test(
            response.headers.get("content-type") ?? "",
          ),
          "Marketplace JSON content type required",
        );
        const declared = response.headers.get("content-length");
        requireValue(
          declared === null ||
            (/^\d+$/.test(declared) && Number(declared) <= marketplaceInventoryLimit),
          `Marketplace response exceeds ${marketplaceInventoryLimit} bytes`,
        );
        requireValue(response.body, "Marketplace response body required");
        const reader = response.body.getReader(),
          chunks: Buffer[] = [];
        let length = 0;
        try {
          for (;;) {
            const part = await reader.read();
            if (part.done) break;
            length += part.value.byteLength;
            requireValue(
              length <= marketplaceInventoryLimit,
              `Marketplace response exceeds ${marketplaceInventoryLimit} bytes`,
            );
            chunks.push(Buffer.from(part.value));
          }
        } finally {
          void reader.cancel().catch(() => {});
          reader.releaseLock();
        }
        requireValue(length > 0, "Marketplace response body empty");
        const bytes = Buffer.concat(chunks, length);
        const responseSha256 = createHash("sha256").update(bytes).digest("hex");
        let body: unknown;
        let extension: Record<string, unknown>;
        try {
          body = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)) as unknown;
          extension = marketplaceExtension(name, body, response.headers);
        } catch (error) {
          throw new Error(
            `${JSON.stringify(String(error))}; HTTP ${response.status} at ${marketplaceQueryUrl}; response sha256=${responseSha256}; bytes=${length}; body=${JSON.stringify(bytes.toString("utf8"))}`,
          );
        }
        return {
          url: marketplaceQueryUrl,
          method: "POST" as const,
          query,
          status: 200 as const,
          observedAt: new Date().toISOString(),
          responseSha256,
          responseBytes: length,
          response: body,
          date: response.headers.get("date"),
          extension,
        };
      })(),
    ]);
  } finally {
    clearTimeout(timer);
    controller.abort();
  }
}
