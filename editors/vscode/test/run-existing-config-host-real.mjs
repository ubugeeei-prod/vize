import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  prepareRealVueWorkspace,
  resolveCorsaPath,
} from "../../../tools/support/compat/editor-e2e/real-vue-workspace.mjs";
import { createRealHostEnvironment, runPackagedExtensionHost } from "./packaged-host-contract.mjs";

export async function runExistingConfigHostScenarios(runCommand, options) {
  for (const mode of ["vite", "tsconfig"]) {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-host-"));
    const workspacePath = path.join(directory, "workspace");
    const extensionsPath = path.join(directory, "extensions");
    try {
      prepareRealVueWorkspace(workspacePath);
      fs.rmSync(path.join(workspacePath, "vize.config.json"));
      fs.writeFileSync(
        path.join(workspacePath, "src/Formatting.vue"),
        '<script setup lang="ts">const count=1</script><template><div>{{count}}</div></template>\n',
      );
      fs.writeFileSync(
        path.join(workspacePath, ".vscode/settings.json"),
        JSON.stringify({ "vize.serverPath": options.serverPath, "chat.disableAIFeatures": true }),
      );
      if (mode === "vite") {
        // Project feature disables must win over the extension's automatic
        // startup profile, while unrelated server defaults remain enabled.
        fs.writeFileSync(
          path.join(workspacePath, "vite.config.ts"),
          `export default ${JSON.stringify({ vize: { lsp: { hover: false } } })};\n`,
        );
      }
      await runPackagedExtensionHost(runCommand, {
        ...options,
        extensionId: "ubugeeei.vize",
        extensionsPath,
        extensionTestsPath: path.join(
          options.sourceExtensionPath,
          "test/suite/existing-config-real.cjs",
        ),
        hostEnvironment: {
          ...createRealHostEnvironment({
            extensionsPath,
            processEnvironment: process.env,
            serverPath: options.serverPath,
            sourceExtensionPath: options.sourceExtensionPath,
          }),
          CORSA_PATH: resolveCorsaPath(),
          VIZE_TEST_CONFIG_MODE: mode,
        },
        hostTimeoutMs: 180_000,
        installEnvironment: process.env,
        installTimeoutMs: 120_000,
        userDataPath: path.join(directory, "profile"),
        workspacePath,
      });
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
}
