import { collectOutput, println, sleepSeconds, writeStderr } from "../command_io.ts";
import { moonTrim } from "../command_runtime.ts";
import { isJsonObject, parseJson, stringMember, type JsonValue } from "../ordered_json.ts";

/** Result of querying crates.io for one exact crate version. */
export type CrateVersionState =
  | { kind: "published" }
  | { kind: "missing" }
  | { kind: "query-failed"; message: string };

const queryFailed = (message: string): CrateVersionState => ({ kind: "query-failed", message });

/**
 * Parses the crates.io response without turning protocol failures into a
 * false "not published" result.
 */
export function publishedVersionState(
  value: JsonValue,
  crateName: string,
  version: string,
): CrateVersionState {
  if (!isJsonObject(value)) {
    return queryFailed(`crates.io returned a non-object response for ${crateName}@${version}`);
  }
  const versionObject = value.get("version");
  if (!isJsonObject(versionObject)) {
    return queryFailed(`crates.io response for ${crateName}@${version} has no version object`);
  }
  const returnedCrate = stringMember(versionObject, "crate");
  if (returnedCrate === undefined) {
    return queryFailed(`crates.io returned a version without crate for ${crateName}@${version}`);
  }
  const returnedVersion = stringMember(versionObject, "num");
  if (returnedVersion === undefined) {
    return queryFailed(`crates.io returned a version without num for ${crateName}@${version}`);
  }
  if (returnedCrate !== crateName || returnedVersion !== version) {
    return queryFailed(`crates.io returned mismatched identity for ${crateName}@${version}`);
  }
  return { kind: "published" };
}

/**
 * Queries crates.io for an exact version. Dry-run callers consume the full
 * state so transport and schema failures stop the release preflight.
 */
export async function queryCrateVersion(
  crateName: string,
  version: string,
): Promise<CrateVersionState> {
  const { status, stdout, stderr } = await collectOutput("curl", [
    "-sS",
    "--fail-with-body",
    "--connect-timeout",
    "5",
    "--max-time",
    "15",
    "--retry",
    "2",
    "--retry-delay",
    "1",
    "--retry-connrefused",
    "--write-out",
    "VIZE_HTTP_STATUS:%{http_code}",
    "-H",
    "User-Agent: vize-release-publish (https://github.com/ubugeeei-prod/vize)",
    `https://crates.io/api/v1/crates/${crateName}/${version}`,
  ]);
  const responseParts = stdout.split("VIZE_HTTP_STATUS:");
  if (responseParts.length === 2 && responseParts[1] === "404") {
    return { kind: "missing" };
  }
  if (status !== 0) {
    const detail = moonTrim(stderr);
    const suffix = detail === "" ? "" : `: ${detail}`;
    if (responseParts.length === 2 && responseParts[1] !== "000") {
      return queryFailed(
        `crates.io query for ${crateName}@${version} returned HTTP ${responseParts[1]} (curl exit ${status})${suffix}`,
      );
    }
    return queryFailed(
      `crates.io query for ${crateName}@${version} failed with curl exit ${status}${suffix}`,
    );
  }
  if (responseParts.length !== 2) {
    return queryFailed(`crates.io query for ${crateName}@${version} returned no HTTP status`);
  }
  const httpStatus = responseParts[1];
  if (httpStatus !== "200") {
    return queryFailed(`crates.io query for ${crateName}@${version} returned HTTP ${httpStatus}`);
  }
  const parsed = parseJson(responseParts[0]);
  if (parsed === undefined) {
    return queryFailed(`crates.io returned invalid JSON for ${crateName}@${version}`);
  }
  return publishedVersionState(parsed, crateName, version);
}

/**
 * Actual publication preserves its conservative fallback: an inconclusive
 * API query is treated as unpublished and delegated to cargo's idempotent
 * publish/re-resolution path.
 */
export async function crateIsPublished(crateName: string, version: string): Promise<boolean> {
  return (await queryCrateVersion(crateName, version)).kind === "published";
}

/**
 * Returns an actionable error when Cargo cannot resolve an allegedly
 * published version from the registry index.
 */
export async function crateResolutionError(
  crateName: string,
  version: string,
): Promise<string | null> {
  const { status, stdout, stderr } = await collectOutput("cargo", [
    "info",
    "--registry",
    "crates-io",
    `${crateName}@${version}`,
  ]);
  if (status === 0) return null;
  const detail = [moonTrim(stdout), moonTrim(stderr)].filter((text) => text !== "").join("\n");
  const suffix = detail === "" ? "" : `\n${detail}`;
  return `Cargo could not resolve published prerequisite ${crateName}@${version} (exit ${status})${suffix}`;
}

export async function crateIsResolvable(crateName: string, version: string): Promise<boolean> {
  return (await crateResolutionError(crateName, version)) === null;
}

/** Waits until `cargo info crate@version` succeeds or a timeout is reached. */
export async function waitForResolution(
  crateName: string,
  version: string,
  maxAttempts: number,
  delaySeconds: number,
): Promise<number> {
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    if (await crateIsResolvable(crateName, version)) {
      println(`${crateName} v${version} is resolvable from crates.io.`);
      return 0;
    }
    if (attempt >= maxAttempts) {
      writeStderr(
        `Timed out waiting for ${crateName} v${version} to become resolvable from crates.io.\n`,
      );
      return 1;
    }
    println(
      `Waiting for ${crateName} v${version} to become resolvable from crates.io... (${attempt}/${maxAttempts})`,
    );
    await sleepSeconds(delaySeconds);
  }
  return 1;
}
