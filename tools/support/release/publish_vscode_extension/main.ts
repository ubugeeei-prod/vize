// Publishes the VS Code extension in a rerun-safe way.
// Release retries are common when a tag is force-pushed to re-run a partially
// completed workflow. npm packages and crates are already idempotent in this
// repository, so this script applies the same principle to the VS Code
// Marketplace publish step:
// 1. Read the extension id and target version from `package.json`.
// 2. Query the public gallery inventory directly (one small POST per probe).
// 3. Skip publish entirely when the exact version and target are validated.
// 4. Treat a non-zero `vsce publish` exit as success if the version becomes
//    visible immediately after the attempted publish.
import { collectOutput, println, run, runMain, sleepSeconds, writeStderr } from "../command_io.ts";
import { cliArgs, nonEmptyEnv, resolvePath } from "../command_runtime.ts";
import { envIntAtLeast, readExtensionInfo, readExtensionTarget } from "../editor_extension_info.ts";
import {
  marketplaceExtension,
  marketplaceQuery,
  marketplaceQueryAccept,
  marketplaceQueryUrl,
} from "../marketplace_query.ts";

function configuredVsceDlxBin(): string {
  return nonEmptyEnv("VSCE_DLX_BIN") ?? nonEmptyEnv("VP_BIN") ?? "vp";
}

export function vsceDlxArgs(
  dlxBin: string,
  command: string,
  extraArgs: readonly string[],
): string[] {
  if (dlxBin === "npx") {
    return ["-y", "-p", "@vscode/vsce@^3.3.2", "vsce", command, ...extraArgs];
  }
  const args: string[] = [];
  if (dlxBin === "corepack") args.push("pnpm");
  args.push("dlx");
  if (dlxBin === "pnpm" || dlxBin === "corepack") {
    args.push("--allow-build", "@vscode/vsce-sign", "--allow-build", "keytar", "--package");
  } else {
    args.push("-p");
  }
  args.push("@vscode/vsce@^3.3.2", "vsce", command, ...extraArgs);
  return args;
}

/**
 * Returns whether a gallery response lists a validated `version` for `target`. A row
 * without `targetPlatform` is universal. A malformed or partial inventory is
 * treated as not visible, so the caller keeps polling.
 */
export function galleryListsVersion(
  extensionId: string,
  body: string,
  version: string,
  target: string,
): boolean {
  let versions: unknown;
  try {
    versions = marketplaceExtension(extensionId, JSON.parse(body)).versions;
  } catch {
    return false;
  }
  return (
    Array.isArray(versions) &&
    versions.some((row: { version?: unknown; targetPlatform?: unknown; flags?: unknown }) => {
      const platform =
        row.targetPlatform == null || row.targetPlatform === "" ? "universal" : row.targetPlatform;
      return (
        row.version === version &&
        platform === target &&
        typeof row.flags === "string" &&
        row.flags.split(",").some((flag) => flag.trim() === "validated")
      );
    })
  );
}

// `vsce show` lists the same rows, but each probe spawns a package runner and
// downloads the 700 KB detail record. This asks the gallery for versions only.
async function extensionVersionIsVisible(
  extensionId: string,
  version: string,
  target: string,
): Promise<boolean> {
  let query: string;
  try {
    query = JSON.stringify(marketplaceQuery(extensionId));
  } catch {
    return false;
  }
  const { status, stdout } = await collectOutput("curl", [
    "--fail",
    "--silent",
    "--show-error",
    "--max-time",
    "30",
    "--header",
    `Accept: ${marketplaceQueryAccept}`,
    "--header",
    "Content-Type: application/json",
    "--header",
    "Cache-Control: no-cache",
    "--data",
    query,
    marketplaceQueryUrl,
  ]);
  return status === 0 && galleryListsVersion(extensionId, stdout, version, target);
}

async function waitForExtensionVisibility(
  extensionId: string,
  version: string,
  maxAttempts: number,
  delaySeconds: number,
  target: string,
): Promise<boolean> {
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    if (await extensionVersionIsVisible(extensionId, version, target)) {
      println(`${extensionId} ${version} is visible in the VS Code Marketplace.`);
      return true;
    }
    if (attempt >= maxAttempts) return false;
    println(
      `Waiting for ${extensionId} ${version} to appear in the VS Code Marketplace... (${attempt}/${maxAttempts})`,
    );
    await sleepSeconds(delaySeconds);
  }
  return false;
}

export async function publishVscodeExtension(args: readonly string[]): Promise<number> {
  if (args.length === 0 || args.length > 2) {
    println(
      "Usage: node tools/support/release/publish_vscode_extension/main.ts <vsix-path> [package-json-path]",
    );
    return 1;
  }
  const packageJsonPath = resolvePath(args.length === 2 ? args[1] : "editors/vscode/package.json");
  const extensionInfo = readExtensionInfo(packageJsonPath);
  if (extensionInfo === null) return 1;
  const target = readExtensionTarget();
  if (target === null) return 1;
  const dlxBin = configuredVsceDlxBin();
  const resolutionAttempts = envIntAtLeast("PUBLISH_RESOLUTION_RETRY_LIMIT", 1, 24);
  const resolutionDelaySeconds = envIntAtLeast("PUBLISH_RESOLUTION_RETRY_DELAY", 0, 10);
  if (await extensionVersionIsVisible(extensionInfo.extensionId, extensionInfo.version, target)) {
    println(
      `${extensionInfo.extensionId} ${extensionInfo.version} is already published, skipping VS Code Marketplace publish.`,
    );
    return 0;
  }

  const tag = nonEmptyEnv("NPM_TAG") ?? "latest";
  const publishArgs = ["--no-dependencies", "--packagePath", resolvePath(args[0])];
  if (tag !== "latest") {
    publishArgs.push("--pre-release");
  }
  const exitCode = await run(dlxBin, vsceDlxArgs(dlxBin, "publish", publishArgs));
  // Keep the existing universal upload contract. A platform upload must also
  // become visible under its own target before that matrix entry can pass.
  if (exitCode === 0 && target === "universal") return 0;
  // A failed exit can still have published; recheck the same exact target.
  if (
    await waitForExtensionVisibility(
      extensionInfo.extensionId,
      extensionInfo.version,
      resolutionAttempts,
      resolutionDelaySeconds,
      target,
    )
  ) {
    if (exitCode !== 0)
      println(
        `${extensionInfo.extensionId} ${extensionInfo.version} was published despite a non-zero vsce publish exit.`,
      );
    return 0;
  }
  if (exitCode === 0) {
    writeStderr(
      `VS Code Marketplace did not expose ${extensionInfo.extensionId} ${extensionInfo.version} for ${target} before the visibility deadline (${resolutionAttempts} x ${resolutionDelaySeconds} s).\n`,
    );
    return 1;
  }
  return exitCode;
}

if (import.meta.main) {
  runMain(() => publishVscodeExtension(cliArgs()));
}
