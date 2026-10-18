import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
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
  return manifestMatches(value, name, version);
}

function manifestMatches(value: JsonValue | undefined, name: string, version: string): boolean {
  return (
    isJsonObject(value) &&
    stringMember(value, "name") === name &&
    stringMember(value, "version") === version
  );
}

/** Require the ordinary manifest URL to serve the complete archive with its exact SRI. */
export async function npmTarballIsVisible(
  manifest: JsonValue,
  remainingMs = 6000,
): Promise<boolean> {
  const timeoutMs = Math.min(6000, Math.floor(remainingMs));
  if (!Number.isFinite(timeoutMs) || timeoutMs < 1) return false;
  if (!isJsonObject(manifest)) return false;
  const dist = manifest.get("dist");
  if (!isJsonObject(dist)) return false;
  const tarball = stringMember(dist, "tarball"),
    integrity = stringMember(dist, "integrity");
  if (!tarball || !integrity || !/^sha512-[A-Za-z0-9+/]{86}==$/.test(integrity)) return false;
  try {
    const url = new URL(tarball);
    if (!["http:", "https:"].includes(url.protocol) || url.username || url.password) return false;
  } catch {
    return false;
  }
  return new Promise((resolve) => {
    // Keep the existing 3s connect / 6s total probe limits, including a stalled body.
    const child = spawn(
      "curl",
      [
        "--silent",
        "--show-error",
        "--fail",
        "--connect-timeout",
        String(Math.min(3000, timeoutMs) / 1000),
        "--max-time",
        String(timeoutMs / 1000),
        "--proto",
        "=http,https",
        "--header",
        "Cache-Control: no-cache",
        "--write-out",
        "%{stderr}VIZE_HTTP_STATUS:%{http_code}",
        tarball,
      ],
      { stdio: ["ignore", "pipe", "pipe"], timeout: timeoutMs, killSignal: "SIGKILL" },
    );
    const hash = createHash("sha512");
    let bytes = 0,
      status = "",
      statusBytes = 0,
      bounded = true;
    child.stdout.on("data", (chunk: Buffer) => {
      bytes += chunk.length;
      if (bytes > 512 * 1024 * 1024) {
        bounded = false;
        child.kill("SIGKILL");
      } else hash.update(chunk);
    });
    child.stderr.on("data", (chunk: Buffer) => {
      statusBytes += chunk.length;
      if (statusBytes > 4096) {
        bounded = false;
        child.kill("SIGKILL");
      } else status += chunk.toString("utf8");
    });
    child.once("error", () => resolve(false));
    child.once("close", (code) =>
      resolve(
        code === 0 &&
          bounded &&
          bytes > 0 &&
          status === "VIZE_HTTP_STATUS:200" &&
          `sha512-${hash.digest("base64")}` === integrity,
      ),
    );
  });
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
 * Polls until exact metadata, dist-tag and the whole SRI-verified archive are visible.
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
    const deadline = performance.now() + 6000;
    const [manifest, tagVisible] = await Promise.all([
      npmRegistryJson(info.name, info.version, attempt),
      npmTagPointsToVersion(info.name, info.tag, info.version, attempt),
    ]);
    if (
      manifestMatches(manifest, info.name, info.version) &&
      tagVisible &&
      manifest !== undefined &&
      (await npmTarballIsVisible(manifest, deadline - performance.now()))
    ) {
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
