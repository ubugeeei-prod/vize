#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runVSCodeCommand } from "@vscode/test-electron";
import {
  resolveCorsaPath,
  resolveRealServerPath,
} from "../../../tools/support/compat/editor-e2e/real-vue-workspace.ts";
import { createRealHostEnvironment, runPackagedExtensionHost } from "./packaged-host-contract.mjs";
import { prepareRouterHostWorkspace } from "./router-host-workspace.mjs";
import { recordRouterHostIdentity } from "./router-host-artifact-identity.mjs";

const sourceExtension = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repository = path.resolve(sourceExtension, "../..");
const output = path.resolve(
  process.env.VIZE_ROUTER_HOST_ARTIFACT_DIR ??
    path.join(repository, "target/router-editor-artifact"),
);
fs.mkdirSync(output, { recursive: true });
const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-router-host-"));
const server = resolveRealServerPath();
const corsa = resolveCorsaPath();
const vscodeVersion = process.env.VIZE_TEST_VSCODE_VERSION ?? "1.107.1";
assert.equal(vscodeVersion, "1.107.1");
const vsix = path.join(sourceExtension, "dist/vize.vsix");
const producer = recordRouterHostIdentity({
  output,
  root: repository,
  server,
  corsa,
  vsix,
  vscodeVersion,
});
let campaign;
try {
  campaign = await prepareRouterHostWorkspace({
    directory,
    output,
    repository,
    sourceExtension,
    corsa,
  });
  const extensionsPath = path.join(directory, "extensions");
  const installed = await runPackagedExtensionHost(runVSCodeCommand, {
    extensionId: "ubugeeei.vize",
    extensionsPath,
    extensionTestsPath: path.join(sourceExtension, "test/suite/router-config-real.cjs"),
    hostEnvironment: {
      ...createRealHostEnvironment({
        extensionsPath,
        processEnvironment: process.env,
        serverPath: server,
        sourceExtensionPath: sourceExtension,
      }),
      CORSA_PATH: corsa,
      VIZE_TEST_OBSERVED_SERVER_PATH: server,
      VIZE_TEST_CONFIG_PROTOCOL_PATH: path.join(output, "protocol.jsonl"),
      VIZE_TEST_ROUTER_CAMPAIGN: campaign.manifestPath,
      VIZE_TEST_ROUTER_OUTPUT: output,
    },
    hostTimeoutMs: 360_000,
    installEnvironment: process.env,
    installTimeoutMs: 120_000,
    onOutput: ({ stdout, stderr }) => {
      if (stdout) process.stdout.write(stdout);
      if (stderr) process.stderr.write(stderr);
    },
    userDataPath: path.join(directory, "profile"),
    vscodeVersion,
    vsixPath: vsix,
    workspacePath: campaign.workspace,
  });
  assert.ok(installed.startsWith(`${fs.realpathSync(extensionsPath)}${path.sep}`));
  const consumed = JSON.parse(
    fs.readFileSync(path.join(output, "installed-extension.json"), "utf8"),
  );
  assert.equal(consumed.bundleSha256, producer.packagedBundleSha256);
  for (const field of ["publisher", "name", "version", "main"])
    assert.equal(consumed.packageJSON[field], producer.packagedManifest[field]);
  const processes = JSON.parse(
    fs.readFileSync(path.join(output, "process-isolation.json"), "utf8"),
  );
  assert.equal(processes.spawns.length, 1);
  assert.equal(processes.spawns[0].binarySha256, producer.native.sha256);
} catch (error) {
  fs.writeFileSync(
    path.join(output, "failure.json"),
    `${JSON.stringify({ message: error.message, stack: error.stack }, null, 2)}\n`,
  );
  throw error;
} finally {
  // Keep every authored/config byte and complete raw packet before owned temp cleanup.
  for (const project of campaign?.projects ?? [])
    fs.cpSync(project.root, path.join(output, "authored", project.name), {
      recursive: true,
      filter: (file) => !file.split(path.sep).includes("node_modules"),
    });
  if (fs.existsSync(path.join(directory, "profile/logs")))
    fs.cpSync(path.join(directory, "profile/logs"), path.join(output, "host-logs"), {
      recursive: true,
    });
  campaign?.provider.cleanup();
  fs.rmSync(directory, { recursive: true, force: true });
}
