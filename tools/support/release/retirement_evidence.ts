import {
  marketplaceExtension,
  marketplaceInventoryLimit,
  marketplaceQuery,
  marketplaceQueryUrl,
} from "./marketplace_query.ts";
export { marketplaceInventoryLimit } from "./marketplace_query.ts";

export type AbsenceTarget = {
  channel: "npm" | "crates" | "marketplace" | "openvsx";
  name: string;
  version: string;
  url: string;
  method: "GET" | "POST";
  query?: ReturnType<typeof marketplaceQuery>;
};
export type AbsenceEvidence =
  | "typed-exact-version-missing"
  | "complete-marketplace-version-inventory";

export const missingResponseLimit = 65_536;

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
function marketplaceInventory(target: AbsenceTarget, body: unknown, headers: Headers) {
  requireValue(
    target.url === marketplaceQueryUrl &&
      target.method === "POST" &&
      JSON.stringify(target.query) === JSON.stringify(marketplaceQuery(target.name)),
    "Marketplace complete public query required",
  );
  const extension = marketplaceExtension(target.name, body, headers);
  for (const value of extension.versions as unknown[]) {
    const row = object(value);
    requireValue(
      row.version !== target.version,
      `Marketplace exact version already exists: ${target.version}`,
    );
  }
}

function missing(target: AbsenceTarget, value: unknown) {
  // The public exact-version npm route returns a JSON string for missing versions.
  if (target.channel === "npm" && typeof value === "string") {
    requireValue(
      value === `version not found: ${target.version}`,
      `unsupported npm missing response: ${target.name}`,
    );
    return;
  }
  const body = object(value);
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
      (keys.length === 1 ||
        (keys.length === 3 && body.deprecated === false && body.downloadable === false)) &&
        body.error === `Extension not found: ${target.name} ${target.version}`,
      "unsupported Open VSX missing response",
    );
  } else {
    throw new Error("unsupported Marketplace missing response");
  }
}

/** Only the complete public Marketplace query supports a 200 absence proof. */
export function absenceEvidence(
  target: AbsenceTarget,
  status: number,
  body: unknown,
  headers: Headers,
): AbsenceEvidence {
  if (target.channel === "marketplace" && status === 200) {
    marketplaceInventory(target, body, headers);
    return "complete-marketplace-version-inventory";
  }
  requireValue(
    target.channel !== "marketplace" && status === 404,
    "absence requires HTTP 404 or complete Marketplace inventory",
  );
  missing(target, body);
  return "typed-exact-version-missing";
}

export function absenceResponseLimit(target: AbsenceTarget, status: number) {
  return target.channel === "marketplace" && status === 200
    ? marketplaceInventoryLimit
    : missingResponseLimit;
}
