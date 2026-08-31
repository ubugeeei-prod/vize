import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  createPackagedHostEnvironment,
  createPackagedHostInstallArgs,
  createPackagedHostLaunchArgs,
  createRealHostEnvironment,
  resolveInstalledExtensionPath,
  runPackagedExtensionHost,
  runVSCodeCommandWithTimeout,
} from "../../editors/vscode/test/packaged-host-contract.mjs";
import { readPinnedCreateVueHostResult } from "../../editors/vscode/test/pinned-create-vue-host-result.mjs";
import { prepareRealVueWorkspace } from "../../legacy-tools/editor-e2e/real-vue-workspace.mjs";
import { testAndBenchmarkTasks } from "../../tools/config/vite-plus/tasks/test-benchmark.ts";
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
    "--disable-extensions",
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
  assert.equal(launchArgs.includes("--disable-extensions"), true);
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

test("the real-server fixture can preserve a pinned oracle while adding its harness", () => {
  const workspacePath = fs.mkdtempSync(path.join(os.tmpdir(), "vize-preserved-oracle-"));
  const pinnedPath = path.join(workspacePath, "pinned", "App.vue");
  fs.mkdirSync(path.dirname(pinnedPath), { recursive: true });
  fs.writeFileSync(pinnedPath, "<template>pinned</template>\n");

  try {
    prepareRealVueWorkspace(workspacePath, { preserveExisting: true });

    assert.equal(fs.readFileSync(pinnedPath, "utf8"), "<template>pinned</template>\n");
    assert.ok(fs.existsSync(path.join(workspacePath, "src", "App.vue")));
    assert.ok(fs.existsSync(path.join(workspacePath, ".vscode", "settings.json")));
    assert.ok(fs.existsSync(path.join(workspacePath, "vize.config.json")));
    assert.ok(fs.lstatSync(path.join(workspacePath, "node_modules", "vue")).isSymbolicLink());
    assert.equal(
      JSON.parse(fs.readFileSync(path.join(workspacePath, "tsconfig.json"), "utf-8"))
        .compilerOptions.allowImportingTsExtensions,
      true,
    );
  } finally {
    fs.rmSync(workspacePath, { force: true, recursive: true });
  }
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
      version: "1.107.1",
    }),
    /VS Code command timed out after 5ms/,
  );
  assert.equal(receivedSignal.aborted, true);
});

test("packaged host strips Node-only options from the VS Code app environment", () => {
  assert.deepEqual(
    createPackagedHostEnvironment({
      NODE_OPTIONS: "--disable-warning=DEP0040",
      VIZE_TEST_SERVER_PATH: "/repo/target/ci/vize",
    }),
    { VIZE_TEST_SERVER_PATH: "/repo/target/ci/vize" },
  );
});

test("real host task packages and statically validates the same VSIX that it runs", () => {
  const { command } = taskShape(testAndBenchmarkTasks["test:vscode-extension:host-real"]);

  const packageAt = command.indexOf("../../tools/commands/editors/vscode/run-package-bin.rs");
  const vsceAt = command.indexOf("@vscode/vsce");
  const packageArgsAt = command.indexOf("package --no-dependencies --out dist/vize.vsix");
  const staticAssertAt = command.indexOf(
    "../../tools/commands/editors/vscode/assert-vsix-package.rs",
  );
  const hostAt = command.indexOf("node test/run-extension-host-real.mjs");
  assert.ok(packageAt >= 0, "real host smoke must build the production VSIX");
  assert.ok(vsceAt > packageAt, "real host smoke must package with the VS Code CLI");
  assert.ok(packageArgsAt > vsceAt, "real host smoke must pass production VSIX package args");
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
    const invocations: { args: string[]; environment: unknown; version: string }[] = [];
    const runCommand = async (
      args: string[],
      options: { spawn: { env: unknown }; version: string },
    ) => {
      invocations.push({ args, environment: options.spawn.env, version: options.version });
      if (args.includes("--install-extension")) {
        writeManifest(path.join(extensionsPath, "ubugeeei.vize-0.311.0"), "ubugeeei", "vize");
      }
      return { stderr: "", stdout: "" };
    };

    const installedExtensionPath = await runPackagedExtensionHost(runCommand, {
      extensionId: "ubugeeei.vize",
      extensionsPath,
      extensionTestsPath: path.join(sourceExtensionPath, "test/suite/extension-host-real.cjs"),
      hostEnvironment: createRealHostEnvironment({
        extensionsPath,
        processEnvironment: { NODE_OPTIONS: "--disable-warning=DEP0040" },
        resultPath: path.join(profilePath, "result.json"),
        serverPath: "/repo/target/ci/vize",
        sourceExtensionPath,
      }),
      hostTimeoutMs: 600_000,
      installEnvironment: {},
      installTimeoutMs: 120_000,
      onOutput: () => {},
      userDataPath,
      vscodeVersion: "1.107.1",
      vsixPath,
      workspacePath: path.join(profilePath, "workspace"),
    });

    assert.equal(invocations.length, 2);
    assert.equal(invocations[0].args[0], "--install-extension");
    assert.equal(invocations[0].args[1], vsixPath);
    assert.equal(invocations[0].version, "1.107.1");
    assert.equal(invocations[1].version, "1.107.1");

    const launchArgs = invocations[1].args;
    assert.equal(
      installedExtensionPath,
      fs.realpathSync(path.join(extensionsPath, "ubugeeei.vize-0.311.0")),
    );
    assert.ok(launchArgs.includes("--disable-extensions"));
    assert.ok(launchArgs.includes(`--extensionDevelopmentPath=${installedExtensionPath}`));
    assert.equal(launchArgs.includes(`--extensionDevelopmentPath=${sourceExtensionPath}`), false);
    // The host commands the completion smoke relies on are gated behind this
    // flag, so it has to reach the launched VS Code app while the Node-only
    // options do not.
    assert.deepEqual(invocations[1].environment, {
      VIZE_TEST_ENABLE_HOST_COMMANDS: "1",
      VIZE_TEST_PACKAGED_EXTENSIONS_DIR: extensionsPath,
      VIZE_TEST_PINNED_CREATE_VUE_RESULT_PATH: path.join(profilePath, "result.json"),
      VIZE_TEST_SERVER_PATH: "/repo/target/ci/vize",
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
          vscodeVersion: "1.107.1",
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

test("packaged host result proves the pinned create-vue diagnostic transition", () => {
  const resultDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-create-vue-host-result-"));
  const resultPath = path.join(resultDirectory, "result.json");
  const passingResult = {
    brokenDiagnostics: [
      {
        code: 2322,
        message: "Type 'string' is not assignable to type 'number'.",
        range: [1, 6, 1, 11],
        severity: 0,
        source: "vize/types",
      },
    ],
    documentDirty: true,
    extensionActive: true,
    fixtureId: "create-vue",
    repairedDiagnostics: [],
    schemaVersion: 1,
  };

  try {
    fs.writeFileSync(resultPath, JSON.stringify(passingResult));
    assert.deepEqual(readPinnedCreateVueHostResult(resultPath), passingResult);

    fs.writeFileSync(
      resultPath,
      JSON.stringify({ ...passingResult, documentDirty: false, extensionActive: false }),
    );
    assert.throws(
      () => readPinnedCreateVueHostResult(resultPath),
      /Expected values to be strictly/,
    );

    fs.writeFileSync(
      resultPath,
      JSON.stringify({
        ...passingResult,
        brokenDiagnostics: [{ ...passingResult.brokenDiagnostics[0], range: [1, 7, 1, 12] }],
      }),
    );
    assert.throws(
      () => readPinnedCreateVueHostResult(resultPath),
      /Expected values to be strictly/,
    );
  } finally {
    fs.rmSync(resultDirectory, { force: true, recursive: true });
  }
});

function taskShape(value: unknown): { command: string } {
  return value as { command: string };
}

function writeManifest(directory: string, publisher: string, name: string): void {
  fs.mkdirSync(directory);
  fs.writeFileSync(path.join(directory, "package.json"), JSON.stringify({ name, publisher }));
}
