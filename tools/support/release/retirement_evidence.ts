export type AbsenceTarget = {
  channel: "npm" | "crates" | "marketplace" | "openvsx";
  name: string;
  version: string;
  url: string;
};
export type AbsenceEvidence =
  | "typed-exact-version-missing"
  | "complete-marketplace-version-inventory";

export const missingResponseLimit = 65_536;
export const marketplaceInventoryLimit = 2 * 1024 * 1024;

function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
function object(value: unknown): Record<string, unknown> {
  requireValue(
    value && typeof value === "object" && !Array.isArray(value),
    "missing response object",
  );
  return value as Record<string, unknown>;
}
function nonempty(value: unknown): value is string {
  return typeof value === "string" && value.length > 0;
}

function marketplaceInventory(
  target: AbsenceTarget,
  body: Record<string, unknown>,
  headers: Headers,
) {
  // vsce duplicate-version check uses getExtension(undefined version, IncludeVersions=1).
  // https://github.com/microsoft/vscode-vsce/blob/main/src/publish.ts
  // GalleryClient.getExtension has no version pagination; query flags 512/65536 limit versions.
  // https://github.com/microsoft/azure-devops-extension-api/blob/master/src/Gallery/GalleryClient.ts
  // PublishedExtension.flags is a separate enum, so it is NOT a query-flags echo.
  const [publisherName, extensionName] = target.name.split(".");
  requireValue(
    target.url ===
      `https://marketplace.visualstudio.com/_apis/gallery/publishers/${publisherName}/extensions/${extensionName}?flags=1&api-version=7.2-preview.2`,
    "Marketplace complete inventory route required",
  );
  const publisher = object(body.publisher);
  requireValue(
    publisher.publisherName === publisherName &&
      body.extensionName === extensionName &&
      nonempty(publisher.publisherId) &&
      nonempty(body.extensionId),
    "Marketplace inventory identity missing or changed",
  );
  const versions = body.versions;
  requireValue(
    Array.isArray(versions) && versions.length > 0 && versions.length <= 10_000,
    "Marketplace complete nonempty versions required",
  );
  for (const key of [
    "pagingToken",
    "continuationToken",
    "nextLink",
    "@odata.nextLink",
    "truncated",
    "hasMore",
    "latestOnly",
    "includeLatestVersionOnly",
  ]) {
    requireValue(!(key in body), `Marketplace partial inventory indicator: ${key}`);
  }
  for (const key of ["versionCount", "totalVersions", "totalCount", "count"]) {
    requireValue(
      !(key in body) || body[key] === versions.length,
      `Marketplace inventory count differs: ${key}`,
    );
  }
  for (const key of [
    "x-ms-continuationtoken",
    "x-ms-continuation-token",
    "x-vss-continuationtoken",
    "link",
  ]) {
    requireValue(!headers.has(key), "Marketplace inventory continuation header refused");
  }
  const seen = new Set<string>();
  for (const value of versions) {
    const row = object(value);
    requireValue(
      nonempty(row.version) &&
        /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(row.version) &&
        (row.targetPlatform == null || typeof row.targetPlatform === "string"),
      "Marketplace malformed version/platform row",
    );
    const key = `${row.version}\0${row.targetPlatform ?? ""}`;
    requireValue(!seen.has(key), "Marketplace duplicate version/platform row");
    seen.add(key);
    requireValue(
      row.version !== target.version,
      `Marketplace exact version already exists: ${target.version}`,
    );
  }
}

function missing(target: AbsenceTarget, body: Record<string, unknown>) {
  const keys = Object.keys(body);
  if (target.channel === "npm") {
    requireValue(
      keys.length === 1 &&
        (body.error === "Not found" || body.error === `version not found: ${target.version}`),
      `unsupported npm missing response: ${target.name}`,
    );
  } else if (target.channel === "crates") {
    // https://github.com/rust-lang/crates.io/blob/main/src/util/errors.rs
    const errors = body.errors;
    requireValue(
      keys.length === 1 && Array.isArray(errors) && errors.length === 1,
      "unsupported crate missing response",
    );
    const error = object(errors[0]);
    requireValue(
      Object.keys(error).length === 1 &&
        [
          `crate \`${target.name}\` does not exist`,
          `crate \`${target.name}\` does not have a version \`${target.version}\``,
        ].includes(error.detail as string),
      `unsupported crate missing detail: ${target.name}`,
    );
  } else if (target.channel === "openvsx") {
    // https://github.com/eclipse-openvsx/openvsx/blob/main/server/src/main/java/org/eclipse/openvsx/RegistryAPI.java
    requireValue(
      keys.length === 1 && body.error === `Extension not found: ${target.name} ${target.version}`,
      "unsupported Open VSX missing response",
    );
  } else {
    const type = body.typeKey,
      message = body.message;
    const known = [
      "ExtensionNotFoundException",
      "ExtensionVersionNotFoundException",
      "ExtensionDoesNotExistException",
      "ExtensionVersionDoesNotExistException",
      "VersionNotFoundException",
    ];
    requireValue(
      typeof type === "string" &&
        known.includes(type) &&
        typeof message === "string" &&
        /not found|does not exist|does not have.*version/i.test(message) &&
        message.includes(target.name) &&
        (!/Version/.test(type) || message.includes(target.version)) &&
        !keys.some((key) => ["publisher", "extensionName", "version", "versions"].includes(key)),
      "unsupported Marketplace missing response",
    );
  }
}

/** Only the complete unversioned Marketplace contract supports a 200 absence proof. */
export function absenceEvidence(
  target: AbsenceTarget,
  status: number,
  body: Record<string, unknown>,
  headers: Headers,
): AbsenceEvidence {
  if (target.channel === "marketplace" && status === 200) {
    marketplaceInventory(target, body, headers);
    return "complete-marketplace-version-inventory";
  }
  requireValue(status === 404, "absence requires HTTP 404 or complete Marketplace inventory");
  missing(target, body);
  return "typed-exact-version-missing";
}

export function absenceResponseLimit(target: AbsenceTarget, status: number) {
  return target.channel === "marketplace" && status === 200
    ? marketplaceInventoryLimit
    : missingResponseLimit;
}
