import fs from "node:fs";
import path from "node:path";

export function createPackagedHostInstallArgs({ extensionsPath, userDataPath, vsixPath }) {
  return [
    "--install-extension",
    vsixPath,
    "--force",
    `--extensions-dir=${extensionsPath}`,
    `--user-data-dir=${userDataPath}`,
  ];
}

// VS Code only executes --extensionTestsPath for an extension development
// host. Point that protocol requirement at the extracted VSIX itself; using
// the repository source path here would silently stop testing the artifact.
export function createPackagedHostLaunchArgs({
  extensionsPath,
  extensionTestsPath,
  installedExtensionPath,
  userDataPath,
  workspacePath,
}) {
  return [
    "--no-sandbox",
    "--disable-gpu-sandbox",
    "--disable-updates",
    "--disable-workspace-trust",
    "--skip-welcome",
    "--skip-release-notes",
    `--extensions-dir=${extensionsPath}`,
    `--user-data-dir=${userDataPath}`,
    `--extensionDevelopmentPath=${installedExtensionPath}`,
    `--extensionTestsPath=${extensionTestsPath}`,
    workspacePath,
  ];
}

/**
 * Runs the packaged extension host protocol: install the VSIX into the
 * isolated profile, resolve the extracted copy, then execute the test suite
 * against that copy. Keeping the sequence here (instead of inline in the
 * runner) lets tests drive it with a recording `runCommand` and observe that
 * the host is always launched from the installed extension.
 */
export async function runPackagedExtensionHost(
  runCommand,
  {
    extensionId,
    extensionsPath,
    extensionTestsPath,
    hostEnvironment,
    hostTimeoutMs,
    installEnvironment,
    installTimeoutMs,
    onOutput,
    userDataPath,
    vsixPath,
    workspacePath,
  },
) {
  if (!fs.existsSync(vsixPath)) {
    throw new Error(`missing packaged VS Code extension: ${vsixPath}`);
  }

  const installArgs = createPackagedHostInstallArgs({ extensionsPath, userDataPath, vsixPath });
  onOutput(
    await runVSCodeCommandWithTimeout(runCommand, installArgs, {
      environment: installEnvironment,
      timeoutMs: installTimeoutMs,
    }),
  );

  const installedExtensionPath = resolveInstalledExtensionPath(extensionsPath, extensionId);
  const launchArgs = createPackagedHostLaunchArgs({
    extensionsPath,
    extensionTestsPath,
    installedExtensionPath,
    userDataPath,
    workspacePath,
  });
  onOutput(
    await runVSCodeCommandWithTimeout(runCommand, launchArgs, {
      environment: hostEnvironment,
      timeoutMs: hostTimeoutMs,
    }),
  );

  return installedExtensionPath;
}

export function resolveInstalledExtensionPath(extensionsPath, extensionId) {
  const matches = fs
    .readdirSync(extensionsPath, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(extensionsPath, entry.name))
    .filter((candidate) => readExtensionId(candidate) === extensionId);

  if (matches.length !== 1) {
    throw new Error(
      `expected exactly one installed ${extensionId} extension, found ${matches.length}: ${matches.join(", ")}`,
    );
  }

  return fs.realpathSync(matches[0]);
}

export async function runVSCodeCommandWithTimeout(runCommand, args, { environment, timeoutMs }) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    return await runCommand(args, {
      spawn: { env: environment, signal: controller.signal },
    });
  } catch (error) {
    if (controller.signal.aborted) {
      throw new Error(`VS Code command timed out after ${timeoutMs}ms`, { cause: error });
    }
    throw error;
  } finally {
    clearTimeout(timeout);
  }
}

function readExtensionId(extensionPath) {
  try {
    const manifest = JSON.parse(fs.readFileSync(path.join(extensionPath, "package.json"), "utf8"));
    return `${manifest.publisher}.${manifest.name}`;
  } catch {
    return undefined;
  }
}
