// Publishes the VS Code extension to Open VSX in a rerun-safe way.
// Release retries are common when a tag is force-pushed to re-run a partially
// completed workflow. This script mirrors the VS Code Marketplace publish
// guard by checking Open VSX metadata before publishing and after any non-zero
// publish exit. During registry maintenance every write answers "Registry is
// in read-only mode."; those writes are retried with backoff.
import {
  collectOutput,
  println,
  runMain,
  sleepSeconds,
  writeStderr,
  writeStdout,
  type CollectedOutput,
} from "../command_io.ts";
import { asciiLowercase, cliArgs, moonTrim, nonEmptyEnv, resolvePath } from "../command_runtime.ts";
import {
  envIntAtLeast,
  jsonHasExtensionVersion,
  readExtensionInfo,
  readExtensionTarget,
  extensionTargetMatches,
  type ExtensionInfo,
} from "../editor_extension_info.ts";
import { parseJson, stringMember } from "../ordered_json.ts";

async function extensionVersionIsVisible(
  publisher: string,
  name: string,
  version: string,
  target: string,
): Promise<boolean> {
  // The exact-version API returns the version directly. `ovsx get --metadata`
  // now returns `allVersions`, which is not the `versions` array checked below.
  // The version-only endpoint may select a platform build even when the
  // universal version exists. Every target needs its own immutable lookup.
  const targetPath = `${target}/`;
  const url = `https://open-vsx.org/api/${publisher}/${name}/${targetPath}${version}`;
  const { status, stdout } = await collectOutput("curl", [
    "--fail",
    "--silent",
    "--show-error",
    url,
  ]);
  if (status !== 0) return false;
  const body = moonTrim(stdout);
  if (body === "") return false;
  const parsed = parseJson(body);
  if (parsed === undefined) return false;
  if (stringMember(parsed, "version") === version && extensionTargetMatches(parsed, target))
    return true;
  return jsonHasExtensionVersion(parsed, version, target);
}

async function waitForExtensionVisibility(
  info: ExtensionInfo,
  maxAttempts: number,
  delaySeconds: number,
  target: string,
): Promise<boolean> {
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    if (await extensionVersionIsVisible(info.publisher, info.name, info.version, target)) {
      println(`${info.extensionId} ${info.version} is visible in Open VSX.`);
      return true;
    }
    if (attempt >= maxAttempts) return false;
    println(
      `Waiting for ${info.extensionId} ${info.version} to appear in Open VSX... (${attempt}/${maxAttempts})`,
    );
    await sleepSeconds(delaySeconds);
  }
  return false;
}

/** Whether an ovsx write was refused because Open VSX is in maintenance. */
export function openVsxIsReadOnly(output: CollectedOutput): boolean {
  return asciiLowercase(`${output.stdout}\n${output.stderr}`).includes(
    "registry is in read-only mode",
  );
}

interface ReadOnlyRetry {
  attempts: number;
  delaySeconds: number;
}

/** Runs one ovsx write, retrying with doubling delays while Open VSX is read-only. */
async function ovsxWrite(
  vpBin: string,
  args: readonly string[],
  retry: ReadOnlyRetry,
): Promise<CollectedOutput> {
  let delay = retry.delaySeconds;
  for (let attempt = 1; ; attempt++) {
    const output = await collectOutput(vpBin, ["dlx", "-p", "ovsx@^1.0.0", "ovsx", ...args]);
    if (output.status === 0 || !openVsxIsReadOnly(output) || attempt >= retry.attempts) {
      return output;
    }
    println(
      `Open VSX is in read-only mode for maintenance; retrying ovsx ${args[0]} in ${delay} s (${attempt}/${retry.attempts}).`,
    );
    await sleepSeconds(delay);
    delay *= 2;
  }
}

function reportReadOnly(info: ExtensionInfo, retry: ReadOnlyRetry): void {
  writeStderr(
    `Open VSX stayed in read-only mode (maintenance) through ${retry.attempts} attempts; ${info.extensionId} ${info.version} was not uploaded. Re-run this job once open-vsx.org accepts writes.\n`,
  );
}

async function ensureNamespace(
  vpBin: string,
  info: ExtensionInfo,
  retry: ReadOnlyRetry,
): Promise<boolean> {
  const { publisher } = info;
  const output = await ovsxWrite(vpBin, ["create-namespace", publisher], retry);
  if (output.status === 0) return true;
  if (openVsxIsReadOnly(output)) {
    reportReadOnly(info, retry);
    return false;
  }
  const detail = [moonTrim(output.stdout), moonTrim(output.stderr)]
    .filter((text) => text !== "")
    .join("\n");
  if (asciiLowercase(detail).includes("already exists")) {
    println(`Open VSX namespace ${publisher} already exists; continuing to publish.`);
    return true;
  }
  const suffix = detail === "" ? "" : `\n${detail}`;
  writeStderr(
    `Failed to ensure Open VSX namespace ${publisher} (exit ${output.status}).${suffix}\n`,
  );
  return false;
}

export async function publishOpenVsxExtension(args: readonly string[]): Promise<number> {
  if (args.length === 0 || args.length > 2) {
    println(
      "Usage: node tools/support/release/publish_open_vsx_extension/main.ts <vsix-path> [package-json-path]",
    );
    return 1;
  }
  const packageJsonPath = resolvePath(args.length === 2 ? args[1] : "editors/vscode/package.json");
  const extensionInfo = readExtensionInfo(packageJsonPath);
  if (extensionInfo === null) return 1;
  const target = readExtensionTarget();
  if (target === null) return 1;
  const vpBin = nonEmptyEnv("VP_BIN") ?? "vp";
  const resolutionAttempts = envIntAtLeast("PUBLISH_RESOLUTION_RETRY_LIMIT", 1, 24);
  const resolutionDelaySeconds = envIntAtLeast("PUBLISH_RESOLUTION_RETRY_DELAY", 0, 10);
  // 30 + 60 + 120 + 240 s covers short maintenance; longer ones need a re-run.
  const readOnlyRetry = {
    attempts: envIntAtLeast("OPEN_VSX_READ_ONLY_RETRY_LIMIT", 1, 5),
    delaySeconds: envIntAtLeast("OPEN_VSX_READ_ONLY_RETRY_DELAY", 0, 30),
  };
  if (
    await extensionVersionIsVisible(
      extensionInfo.publisher,
      extensionInfo.name,
      extensionInfo.version,
      target,
    )
  ) {
    println(
      `${extensionInfo.extensionId} ${extensionInfo.version} is already published, skipping Open VSX publish.`,
    );
    return 0;
  }
  if (!(await ensureNamespace(vpBin, extensionInfo, readOnlyRetry))) {
    return 1;
  }

  const published = await ovsxWrite(vpBin, ["publish", resolvePath(args[0])], readOnlyRetry);
  writeStdout(published.stdout);
  writeStderr(published.stderr);
  const exitCode = published.status;
  // A refused write uploaded nothing, so there is no listing to wait for.
  if (exitCode !== 0 && openVsxIsReadOnly(published)) {
    reportReadOnly(extensionInfo, readOnlyRetry);
    return exitCode;
  }
  if (
    await waitForExtensionVisibility(
      extensionInfo,
      resolutionAttempts,
      resolutionDelaySeconds,
      target,
    )
  ) {
    if (exitCode !== 0) {
      println(
        `${extensionInfo.extensionId} ${extensionInfo.version} was published despite a non-zero ovsx publish exit.`,
      );
    }
    return 0;
  }
  if (exitCode === 0) {
    writeStderr(
      `Open VSX did not expose ${extensionInfo.extensionId} ${extensionInfo.version} before the visibility deadline.\n`,
    );
    return 1;
  }
  return exitCode;
}

if (import.meta.main) {
  runMain(() => publishOpenVsxExtension(cliArgs()));
}
