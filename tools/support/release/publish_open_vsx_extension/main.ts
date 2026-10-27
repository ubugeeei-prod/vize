// Publishes the VS Code extension to Open VSX in a rerun-safe way.
// Release retries are common when a tag is force-pushed to re-run a partially
// completed workflow. This script mirrors the VS Code Marketplace publish
// guard by checking Open VSX metadata before publishing and after any non-zero
// publish exit.
import { collectOutput, println, run, runMain, sleepSeconds, writeStderr } from "../command_io.ts";
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

async function ensureNamespace(vpBin: string, publisher: string): Promise<boolean> {
  const { status, stdout, stderr } = await collectOutput(vpBin, [
    "dlx",
    "-p",
    "ovsx@^1.0.0",
    "ovsx",
    "create-namespace",
    publisher,
  ]);
  if (status === 0) return true;
  const detail = [moonTrim(stdout), moonTrim(stderr)].filter((text) => text !== "").join("\n");
  if (asciiLowercase(detail).includes("already exists")) {
    println(`Open VSX namespace ${publisher} already exists; continuing to publish.`);
    return true;
  }
  const suffix = detail === "" ? "" : `\n${detail}`;
  writeStderr(`Failed to ensure Open VSX namespace ${publisher} (exit ${status}).${suffix}\n`);
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
  if (!(await ensureNamespace(vpBin, extensionInfo.publisher))) {
    return 1;
  }

  const exitCode = await run(vpBin, [
    "dlx",
    "-p",
    "ovsx@^1.0.0",
    "ovsx",
    "publish",
    resolvePath(args[0]),
  ]);
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
