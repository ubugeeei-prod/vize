import { collectOutput, println, sleepSeconds, writeStderr } from "../command_io.ts";
import { replaceEvery } from "../command_runtime.ts";
import { isJsonObject, parseJson, stringMember, type JsonValue } from "../ordered_json.ts";

/** Minimal package metadata for registry checks. */
export interface NpmPackageInfo {
  name: string;
  version: string;
  tag: string;
}

/** Returns `true` when a `dist-tags` JSON object maps `tag` to `version`. */
export function jsonDistTagMatches(value: JsonValue, tag: string, version: string): boolean {
  const tagVersion = stringMember(value, tag);
  return tagVersion !== undefined && tagVersion === version;
}

/** Escapes scoped package names for npm's registry URL path. */
export function npmRegistryPackagePath(name: string): string {
  return replaceEvery(replaceEvery(name, "@", "%40"), "/", "%2F");
}

/**
 * Reads npm directly so a package manager's cached 404 cannot hide a newly
 * published version. Each probe has a distinct URL and a short timeout.
 */
export async function npmRegistryJson(
  name: string,
  version: string | null,
  probe: number,
): Promise<JsonValue | undefined> {
  const runId = process.env.GITHUB_RUN_ID ?? "local";
  const runAttempt = process.env.GITHUB_RUN_ATTEMPT ?? "1";
  const versionPath = version === null ? "" : `/${version}`;
  const url = `https://registry.npmjs.org/${npmRegistryPackagePath(name)}${versionPath}?vize-publish-check=${runId}-${runAttempt}-${probe}`;
  const { status, stdout } = await collectOutput("curl", [
    "--silent",
    "--show-error",
    "--connect-timeout",
    "3",
    "--max-time",
    "6",
    "--header",
    "Accept: application/json",
    "--header",
    "Cache-Control: no-cache",
    "--write-out",
    "VIZE_HTTP_STATUS:%{http_code}",
    url,
  ]);
  if (status !== 0) return undefined;
  const parts = stdout.split("VIZE_HTTP_STATUS:");
  if (parts.length !== 2 || parts[1] !== "200") return undefined;
  return parseJson(parts[0]);
}

/**
 * Checks whether the exact package version is visible from npm registry reads.
 * Visibility is more important than the publish command's exit code: if the
 * registry already serves `name@version`, rerunning the workflow should treat
 * that state as success rather than attempting to publish again.
 */
export async function npmVersionIsVisible(
  name: string,
  version: string,
  probe: number,
): Promise<boolean> {
  const value = await npmRegistryJson(name, version, probe);
  if (!isJsonObject(value)) return false;
  const returnedName = stringMember(value, "name");
  if (returnedName === undefined) return false;
  const returnedVersion = stringMember(value, "version");
  if (returnedVersion === undefined) return false;
  return returnedName === name && returnedVersion === version;
}

/**
 * Checks whether the package's dist-tag has propagated to the expected version.
 * For consumers of prerelease channels, the dist-tag is often the real source
 * of truth. We therefore wait for both the version and the dist-tag instead of
 * declaring success immediately after `publish` returns.
 */
export async function npmTagPointsToVersion(
  name: string,
  tag: string,
  version: string,
  probe: number,
): Promise<boolean> {
  const value = await npmRegistryJson(name, null, probe);
  if (!isJsonObject(value)) return false;
  const returnedName = stringMember(value, "name");
  if (returnedName === undefined || returnedName !== name) return false;
  const tags = value.get("dist-tags");
  if (tags === undefined) return false;
  return jsonDistTagMatches(tags, tag, version);
}

/**
 * Polls the registry until both the version and dist-tag become visible.
 * Publishing to npm is eventually consistent. This helper turns that
 * propagation window into an explicit, bounded retry loop so workflow failures
 * point to a real timeout rather than a transient post-publish race.
 */
export async function waitForRegistryVisibility(
  info: NpmPackageInfo,
  maxAttempts: number,
  delaySeconds: number,
): Promise<number> {
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    const [versionVisible, tagVisible] = await Promise.all([
      npmVersionIsVisible(info.name, info.version, attempt),
      npmTagPointsToVersion(info.name, info.tag, info.version, attempt),
    ]);
    if (versionVisible && tagVisible) {
      println(`${info.name}@${info.version} is visible in npm with dist-tag ${info.tag}.`);
      return 0;
    }
    if (attempt >= maxAttempts) {
      writeStderr(
        `Timed out waiting for ${info.name}@${info.version} to appear in npm with dist-tag ${info.tag}.\n`,
      );
      return 1;
    }
    println(
      `Waiting for ${info.name}@${info.version} to appear in npm with dist-tag ${info.tag}... (${attempt}/${maxAttempts})`,
    );
    await sleepSeconds(delaySeconds);
  }
  return 1;
}
