import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  createPackagedHostInstallArgs,
  createPackagedHostLaunchArgs,
  resolveInstalledExtensionPath,
  runVSCodeCommandWithTimeout,
} from "../../editors/vscode/test/packaged-host-contract.mjs";
import { root } from "./support/github-workflows.ts";

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
  const tasks = read("tools/vite-plus/tasks/test-benchmark.ts");
  const taskBody = tasks.match(
    /"test:vscode-extension:host-real": noCacheTask\(([\s\S]*?)\n  \),\n  "test:zed-extension/,
  )?.[1];
  assert.ok(taskBody, "missing test:vscode-extension:host-real task");

  const packageAt = taskBody.indexOf("packageVscodeExtension");
  const staticAssertAt = taskBody.indexOf(
    "node ../../tools/vscode-vize/assert-vsix-package.mjs dist/vize.vsix",
  );
  const hostAt = taskBody.indexOf("node test/run-extension-host-real.mjs");
  assert.ok(packageAt >= 0, "real host smoke must build the production VSIX");
  assert.ok(staticAssertAt > packageAt, "the packaged VSIX must retain its static allowlist check");
  assert.ok(hostAt > staticAssertAt, "the validated VSIX must run in the real host");
});

test("real host runner cannot fall back to loading the source extension", () => {
  const runner = read("editors/vscode/test/run-extension-host-real.mjs");
  assert.match(runner, /import \{ runVSCodeCommand \} from "@vscode\/test-electron"/);
  assert.doesNotMatch(runner, /\brunTests\b/);
  assert.match(runner, /resolveInstalledExtensionPath\(extensionsPath, "ubugeeei\.vize"\)/);
  assert.match(runner, /runVSCodeCommandWithTimeout\(runVSCodeCommand, installArgs/);
  assert.match(runner, /runVSCodeCommandWithTimeout\(runVSCodeCommand, launchArgs/);

  const suite = read("editors/vscode/test/suite/extension-host-real.cjs");
  assert.match(suite, /VIZE_TEST_PACKAGED_EXTENSIONS_DIR/);
  assert.match(suite, /VIZE_TEST_SOURCE_EXTENSION_PATH/);
  assert.match(suite, /fs\.realpathSync\(extension\.extensionPath\)/);
  assert.match(suite, /assertPackagedExtension\(extension\);/);
});

function read(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

function writeManifest(directory: string, publisher: string, name: string): void {
  fs.mkdirSync(directory);
  fs.writeFileSync(path.join(directory, "package.json"), JSON.stringify({ name, publisher }));
}
