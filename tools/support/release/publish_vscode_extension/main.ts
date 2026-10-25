// Publishes the VS Code extension in a rerun-safe way.
// Release retries are common when a tag is force-pushed to re-run a partially
// completed workflow. npm packages and crates are already idempotent in this
// repository, so this script applies the same principle to the VS Code
// Marketplace publish step:
// 1. Read the extension id and target version from `package.json`.
// 2. Query Marketplace metadata through `vsce show --json`.
// 3. Skip publish entirely when the exact version is already visible.
// 4. Treat a non-zero `vsce publish` exit as success if the version becomes
//    visible immediately after the attempted publish.
import { collectOutput, println, run, runMain, sleepSeconds, writeStderr } from "../command_io.ts";
import { cliArgs, moonTrim, nonEmptyEnv, resolvePath } from "../command_runtime.ts";
import {
  envIntAtLeast,
  jsonHasExtensionVersion,
  readExtensionInfo,
  readExtensionTarget,
} from "../editor_extension_info.ts";
import { parseJson } from "../ordered_json.ts";

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

async function extensionVersionIsVisible(
  dlxBin: string,
  extensionId: string,
  version: string,
  target: string,
): Promise<boolean> {
  const { status, stdout } = await collectOutput(
    dlxBin,
    vsceDlxArgs(dlxBin, "show", [extensionId, "--json"]),
  );
  if (status !== 0) return false;
  const body = moonTrim(stdout);
  if (body === "") return false;
  const parsed = parseJson(body);
  return parsed !== undefined && jsonHasExtensionVersion(parsed, version, target);
}

async function waitForExtensionVisibility(
  dlxBin: string,
  extensionId: string,
  version: string,
  maxAttempts: number,
  delaySeconds: number,
  target: string,
): Promise<boolean> {
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    if (await extensionVersionIsVisible(dlxBin, extensionId, version, target)) {
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
  if (
    await extensionVersionIsVisible(
      dlxBin,
      extensionInfo.extensionId,
      extensionInfo.version,
      target,
    )
  ) {
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
      dlxBin,
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
      `VS Code Marketplace did not expose ${extensionInfo.extensionId} ${extensionInfo.version} for ${target} before the visibility deadline.\n`,
    );
    return 1;
  }
  return exitCode;
}

if (import.meta.main) {
  runMain(() => publishVscodeExtension(cliArgs()));
}
