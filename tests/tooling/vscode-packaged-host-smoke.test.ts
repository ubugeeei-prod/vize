import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  createPackagedHostInstallArgs,
  createPackagedHostLaunchArgs,
  resolveInstalledExtensionPath,
  runPackagedExtensionHost,
  runVSCodeCommandWithTimeout,
} from "../../editors/vscode/test/packaged-host-contract.mjs";
import { testAndBenchmarkTasks } from "../../tools/vite-plus/tasks/test-benchmark.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

test("packaged VS Code host smoke installs the VSIX before launching its tests", () => {
  const installArgs = createPackagedHostInstallArgs({
    extensionsPath: "/tmp/vize-extensions",
    userDataPath: "/tmp/vize-user-data",
    vsixPath: "/repo/editors/vscode/dist/vize.vsix",
  });
  const launchArgs = createPackagedHostLaunchArgs({
    extensionsPath: "/tmp/vize-extensions",
    extensionTestsPath: "/repo/editors/vscode/test/suite/extension-host-real.cjs",
    installedExtensionPath: "/tmp/vize-extensions/ubugeeei.vize-0.311.0",
    userDataPath: "/tmp/vize-user-data",
    workspacePath: "/repo/editors/vscode/.vscode-test/workspaces/real-vue",
  });

  assert.deepEqual(installArgs, [
    "--install-extension",
    "/repo/editors/vscode/dist/vize.vsix",
    "--force",
    "--extensions-dir=/tmp/vize-extensions",
    "--user-data-dir=/tmp/vize-user-data",
  ]);
  assert.deepEqual(launchArgs, [
    "--no-sandbox",
    "--disable-gpu-sandbox",
    "--disable-updates",
    "--disable-workspace-trust",
    "--skip-welcome",
    "--skip-release-notes",
    "--extensions-dir=/tmp/vize-extensions",
    "--user-data-dir=/tmp/vize-user-data",
    "--extensionDevelopmentPath=/tmp/vize-extensions/ubugeeei.vize-0.311.0",
    "--extensionTestsPath=/repo/editors/vscode/test/suite/extension-host-real.cjs",
    "/repo/editors/vscode/.vscode-test/workspaces/real-vue",
  ]);
  assert.equal(launchArgs.includes("--extensionDevelopmentPath=/repo/editors/vscode"), false);
  assert.equal(launchArgs.includes("--disable-extensions"), false);
});

test("the real-server fixture workspace turns off the built-in AI code actions", () => {
  // VS Code contributes "Fix"/"Explain" quick fixes of its own to every
  // diagnostic span. The real-server scenario asserts the COMPLETE code-action
  // list the language server answers with (#3457), so the fixture workspace has
  // to keep the workbench's own AI actions out of that list.
  //
  // `chat.disableAIFeatures` only exists from VS Code 1.104, which is below the
  // stable build `@vscode/test-electron` downloads for this harness. It does not
  // constrain the extension's own `engines.vscode` range: this file is a test
  // fixture workspace, never shipped, and older builds ignore unknown keys.
  const settings = JSON.parse(
    readRepoFile(
      "editors",
      "vscode",
      "test-fixtures",
      "extension-host",
      "real-vue",
      ".vscode",
      "settings.json",
    ),
  );

  assert.deepEqual(settings, { "chat.disableAIFeatures": true, "vize.enable": false });
});

test("packaged host resolves exactly one installed Vize extension", () => {
  const extensionsPath = fs.mkdtempSync(path.join(os.tmpdir(), "vize-installed-extension-"));
  try {
    writeManifest(path.join(extensionsPath, "other.extension-1.0.0"), "other", "extension");
    const installedPath = path.join(extensionsPath, "ubugeeei.vize-0.311.0");
    writeManifest(installedPath, "ubugeeei", "vize");

    assert.equal(
      resolveInstalledExtensionPath(extensionsPath, "ubugeeei.vize"),
      fs.realpathSync(installedPath),
    );
    writeManifest(path.join(extensionsPath, "ubugeeei.vize-duplicate"), "ubugeeei", "vize");
    assert.throws(
      () => resolveInstalledExtensionPath(extensionsPath, "ubugeeei.vize"),
      /expected exactly one installed ubugeeei\.vize extension, found 2/,
    );
  } finally {
    fs.rmSync(extensionsPath, { force: true, recursive: true });
  }
});

test("packaged host aborts a stuck VS Code command", async () => {
  let receivedSignal;
  const stuckCommand = (_args, options) => {
    receivedSignal = options.spawn.signal;
    return new Promise((_resolve, reject) => {
      receivedSignal.addEventListener("abort", () => reject(new Error("aborted")), { once: true });
    });
  };

  await assert.rejects(
    runVSCodeCommandWithTimeout(stuckCommand, ["--version"], {
      environment: {},
      timeoutMs: 5,
    }),
    /VS Code command timed out after 5ms/,
  );
  assert.equal(receivedSignal.aborted, true);
});

test("real host task packages and statically validates the same VSIX that it runs", () => {
  const { command } = taskShape(testAndBenchmarkTasks["test:vscode-extension:host-real"]);

  const packageAt = command.indexOf("vsce package --no-dependencies --out dist/vize.vsix");
  const staticAssertAt = command.indexOf(
    "node ../../tools/vscode-vize/assert-vsix-package.mjs dist/vize.vsix",
  );
  const hostAt = command.indexOf("node test/run-extension-host-real.mjs");
  assert.ok(packageAt >= 0, "real host smoke must build the production VSIX");
  assert.ok(staticAssertAt > packageAt, "the packaged VSIX must retain its static allowlist check");
  assert.ok(hostAt > staticAssertAt, "the validated VSIX must run in the real host");
});

test("real host runner installs the VSIX and launches the host from the installed copy", async () => {
  const profilePath = fs.mkdtempSync(path.join(os.tmpdir(), "vize-packaged-host-"));
  const extensionsPath = path.join(profilePath, "extensions");
  const userDataPath = path.join(profilePath, "user-data");
  const sourceExtensionPath = path.join(root, "editors", "vscode");
  const vsixPath = path.join(profilePath, "vize.vsix");
  fs.mkdirSync(extensionsPath);
  fs.writeFileSync(vsixPath, "");

  try {
    const invocations: { args: string[]; environment: unknown }[] = [];
    const runCommand = async (args: string[], options: { spawn: { env: unknown } }) => {
      invocations.push({ args, environment: options.spawn.env });
      if (args.includes("--install-extension")) {
        writeManifest(path.join(extensionsPath, "ubugeeei.vize-0.311.0"), "ubugeeei", "vize");
      }
      return { stderr: "", stdout: "" };
    };

    const installedExtensionPath = await runPackagedExtensionHost(runCommand, {
      extensionId: "ubugeeei.vize",
      extensionsPath,
      extensionTestsPath: path.join(sourceExtensionPath, "test/suite/extension-host-real.cjs"),
      hostEnvironment: {
        VIZE_TEST_PACKAGED_EXTENSIONS_DIR: extensionsPath,
        VIZE_TEST_SOURCE_EXTENSION_PATH: sourceExtensionPath,
      },
      hostTimeoutMs: 300_000,
      installEnvironment: {},
      installTimeoutMs: 120_000,
      onOutput: () => {},
      userDataPath,
      vsixPath,
      workspacePath: path.join(profilePath, "workspace"),
    });

    assert.equal(invocations.length, 2);
    assert.equal(invocations[0].args[0], "--install-extension");
    assert.equal(invocations[0].args[1], vsixPath);

    const launchArgs = invocations[1].args;
    assert.equal(
      installedExtensionPath,
      fs.realpathSync(path.join(extensionsPath, "ubugeeei.vize-0.311.0")),
    );
    assert.ok(launchArgs.includes(`--extensionDevelopmentPath=${installedExtensionPath}`));
    assert.equal(launchArgs.includes(`--extensionDevelopmentPath=${sourceExtensionPath}`), false);
    assert.deepEqual(invocations[1].environment, {
      VIZE_TEST_PACKAGED_EXTENSIONS_DIR: extensionsPath,
      VIZE_TEST_SOURCE_EXTENSION_PATH: sourceExtensionPath,
    });
  } finally {
    fs.rmSync(profilePath, { force: true, recursive: true });
  }
});

test("real host runner refuses to launch without a packaged VSIX", async () => {
  const profilePath = fs.mkdtempSync(path.join(os.tmpdir(), "vize-packaged-host-missing-"));
  const extensionsPath = path.join(profilePath, "extensions");
  fs.mkdirSync(extensionsPath);

  try {
    let launched = false;
    await assert.rejects(
      runPackagedExtensionHost(
        async () => {
          launched = true;
          return { stderr: "", stdout: "" };
        },
        {
          extensionId: "ubugeeei.vize",
          extensionsPath,
          extensionTestsPath: path.join(profilePath, "extension-host-real.cjs"),
          hostEnvironment: {},
          hostTimeoutMs: 1000,
          installEnvironment: {},
          installTimeoutMs: 1000,
          onOutput: () => {},
          userDataPath: path.join(profilePath, "user-data"),
          vsixPath: path.join(profilePath, "vize.vsix"),
          workspacePath: path.join(profilePath, "workspace"),
        },
      ),
      /missing packaged VS Code extension/,
    );
    assert.equal(launched, false);
  } finally {
    fs.rmSync(profilePath, { force: true, recursive: true });
  }
});

function taskShape(value: unknown): { command: string } {
  return value as { command: string };
}

function writeManifest(directory: string, publisher: string, name: string): void {
  fs.mkdirSync(directory);
  fs.writeFileSync(path.join(directory, "package.json"), JSON.stringify({ name, publisher }));
}
