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
    "--disable-extensions",
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
 * Builds the environment the real host smoke launches VS Code with. The hidden
 * `vize.test.*` commands only exist when `VIZE_TEST_ENABLE_HOST_COMMANDS` is
 * "1", so this lives next to the launch protocol and tests can assert the flag
 * survives all the way into the recorded launch environment.
 */
export function createRealHostEnvironment({
  extensionsPath,
  processEnvironment,
  resultPath,
  serverPath,
  sourceExtensionPath,
}) {
  return {
    ...processEnvironment,
    VIZE_TEST_ENABLE_HOST_COMMANDS: "1",
    VIZE_TEST_PACKAGED_EXTENSIONS_DIR: extensionsPath,
    VIZE_TEST_PINNED_CREATE_VUE_RESULT_PATH: resultPath,
    VIZE_TEST_SERVER_PATH: serverPath,
    VIZE_TEST_SOURCE_EXTENSION_PATH: sourceExtensionPath,
  };
}

export function createPackagedHostEnvironment(environment) {
  const cleanEnvironment = { ...environment };
  delete cleanEnvironment.NODE_OPTIONS;
  return cleanEnvironment;
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
    vscodeVersion,
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
      version: vscodeVersion,
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
      environment: createPackagedHostEnvironment(hostEnvironment),
      // The suite inside the host runs for minutes, and its progress log is the
      // only evidence of where a stall happened. A piped child hands its output
      // back when the command resolves, so a host that outruns `hostTimeoutMs`
      // reports the abort with everything the suite printed already discarded.
      // Inheriting the streams publishes each line as it is written instead.
      stdio: ["ignore", "inherit", "inherit"],
      timeoutMs: hostTimeoutMs,
      version: vscodeVersion,
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

export async function runVSCodeCommandWithTimeout(
  runCommand,
  args,
  { environment, stdio, timeoutMs, version },
) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    return await runCommand(args, {
      spawn: { env: environment, signal: controller.signal, ...(stdio ? { stdio } : {}) },
      version,
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
