import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { FEATURE_SETTING_KEYS } from "../../editors/vscode/src/extension-core.ts";

const require = createRequire(import.meta.url);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const moduleRuntime = require("node:module") as {
  _load: (request: string, parent: unknown, isMain: boolean) => unknown;
};
const { featureSettingKeys } =
  require("../../editors/vscode/test/suite/extension-host-fixtures.cjs") as {
    featureSettingKeys: string[];
  };

test("VS Code host smokes reset every extension feature switch before starting a server", async () => {
  assert.deepEqual([...featureSettingKeys].sort(), [...FEATURE_SETTING_KEYS].sort());

  const realServerUpdates: ConfigurationUpdate[] = [];
  const { prepareConfiguredRealServer } = requireWithVscodeStub(
    "../../editors/vscode/test/suite/real-server-support.cjs",
    createVscodeStub(realServerUpdates),
  ) as {
    prepareConfiguredRealServer: (serverPath: string) => Promise<void>;
  };
  await prepareConfiguredRealServer("/tmp/vize-language-server");
  assertClearsEveryFeatureSwitch(realServerUpdates, "real server support");

  const autoInsertUpdates: ConfigurationUpdate[] = [];
  const stopAfterReset = new Error("stop after feature reset");
  const { runAutoInsertSmoke } = requireWithVscodeStub(
    "../../editors/vscode/test/suite/auto-insert-smoke.cjs",
    createVscodeStub(autoInsertUpdates, stopAfterReset),
  ) as {
    runAutoInsertSmoke: () => Promise<void>;
  };
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vscode-feature-reset-"));
  const serverPath = path.join(tmp, "server.cjs");
  const logPath = path.join(tmp, "server.log");
  const previousServerPath = process.env.VIZE_TEST_SERVER_PATH;
  const previousServerLog = process.env.VIZE_TEST_SERVER_LOG;
  fs.writeFileSync(serverPath, "");
  try {
    process.env.VIZE_TEST_SERVER_PATH = serverPath;
    process.env.VIZE_TEST_SERVER_LOG = logPath;
    await assert.rejects(
      () => runAutoInsertSmoke(),
      (error) => error === stopAfterReset,
    );
  } finally {
    if (previousServerPath === undefined) {
      delete process.env.VIZE_TEST_SERVER_PATH;
    } else {
      process.env.VIZE_TEST_SERVER_PATH = previousServerPath;
    }
    if (previousServerLog === undefined) {
      delete process.env.VIZE_TEST_SERVER_LOG;
    } else {
      process.env.VIZE_TEST_SERVER_LOG = previousServerLog;
    }
    fs.rmSync(tmp, { force: true, recursive: true });
  }
  assertClearsEveryFeatureSwitch(autoInsertUpdates, "auto insert smoke");
});

type ConfigurationUpdate = {
  key: string;
  target: unknown;
  value: unknown;
};

function requireWithVscodeStub(relativePath: string, vscode: unknown) {
  const modulePath = require.resolve(relativePath);
  delete require.cache[modulePath];
  const load = moduleRuntime._load;
  moduleRuntime._load = (request, parent, isMain) =>
    request === "vscode" ? vscode : load(request, parent, isMain);
  try {
    return require(modulePath);
  } finally {
    moduleRuntime._load = load;
  }
}

function createVscodeStub(updates: ConfigurationUpdate[], openTextDocumentError?: Error) {
  const workspaceTarget = Symbol("workspace");
  return {
    ConfigurationTarget: { Workspace: workspaceTarget },
    Uri: {
      file: (fsPath: string) => ({ fsPath }),
    },
    workspace: {
      getConfiguration(section: string) {
        assert.equal(section, "vize");
        return {
          async update(key: string, value: unknown, target: unknown) {
            updates.push({ key, target, value });
          },
        };
      },
      openTextDocument() {
        if (openTextDocumentError) {
          throw openTextDocumentError;
        }
        assert.fail("openTextDocument is outside this reset contract");
      },
      workspaceFolders: [{ uri: { fsPath: root } }],
    },
  };
}

function assertClearsEveryFeatureSwitch(updates: ConfigurationUpdate[], label: string) {
  const missing = FEATURE_SETTING_KEYS.filter(
    (key) => !updates.some((update) => update.key === key && update.value === undefined),
  );
  assert.deepEqual(missing, [], `${label} must clear every feature switch`);
  for (const key of FEATURE_SETTING_KEYS) {
    assert.equal(
      updates.filter((update) => update.key === key && update.value === undefined).length,
      1,
      `${label} must clear ${key} exactly once`,
    );
  }
}
