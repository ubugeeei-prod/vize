import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  resolveCorsaPath,
  resolveVuePackagePath,
} from "../../../tools/support/compat/editor-e2e/real-vue-workspace.mjs";
import { createRealHostEnvironment, runPackagedExtensionHost } from "./packaged-host-contract.mjs";

/** Separate profiles prove language activation without an earlier Vue session. */
export async function runTsxHostScenarios(runCommand, options) {
  for (const enabled of [false, true]) {
    const profile = fs.mkdtempSync(path.join(os.tmpdir(), "vize-tsx-host-"));
    const workspacePath = path.join(profile, "workspace");
    const extensionsPath = path.join(profile, "extensions");
    try {
      prepareWorkspace(workspacePath, options.serverPath, enabled);
      await runPackagedExtensionHost(runCommand, {
        ...options,
        extensionId: "ubugeeei.vize",
        extensionsPath,
        extensionTestsPath: path.join(options.sourceExtensionPath, "test/suite/real-tsx.cjs"),
        hostEnvironment: {
          ...createRealHostEnvironment({
            ...options,
            extensionsPath,
            processEnvironment: process.env,
          }),
          VIZE_TEST_JSX_ENABLED: String(enabled),
        },
        hostTimeoutMs: 180_000,
        installEnvironment: process.env,
        installTimeoutMs: 120_000,
        userDataPath: path.join(profile, "user-data"),
        workspacePath,
      });
    } finally {
      fs.rmSync(profile, { force: true, recursive: true });
    }
  }
}

function prepareWorkspace(workspacePath, serverPath, enabled) {
  fs.mkdirSync(path.join(workspacePath, ".vscode"), { recursive: true });
  fs.mkdirSync(path.join(workspacePath, "node_modules"));
  fs.symlinkSync(resolveVuePackagePath(), path.join(workspacePath, "node_modules/vue"), "junction");
  writeJson(workspacePath, ".vscode/settings.json", {
    "vize.enable": true,
    "vize.editor.enable": true,
    "vize.typecheck.enable": true,
    "vize.lint.enable": true,
    "vize.serverPath": serverPath,
    "typescript.validate.enable": false,
    "javascript.validate.enable": false,
  });
  fs.writeFileSync(
    path.join(workspacePath, "vite.config.mjs"),
    `export default ${JSON.stringify({
      vue: {
        languageServer: { editor: true, typecheck: true, lint: true },
        typeChecker: { corsaPath: resolveCorsaPath(), jsxTypecheck: enabled },
      },
    })};\n`,
  );
  writeJson(workspacePath, "tsconfig.json", {
    compilerOptions: {
      allowJs: true,
      checkJs: true,
      jsx: "preserve",
      jsxImportSource: "vue",
      module: "ESNext",
      moduleResolution: "bundler",
      noEmit: true,
      strict: true,
      target: "ES2022",
    },
    include: ["*.ts", "*.tsx", "*.jsx"],
  });
  fs.writeFileSync(
    path.join(workspacePath, "model.ts"),
    'export const account = { label: "Ada", visits: 1 };\n',
  );
  for (const extension of ["tsx", "jsx"]) {
    const binding =
      extension === "tsx" ? "const wrong: string = 1;" : "const wrong = account.missing;";
    const component = extension === "tsx" ? "App" : "Sibling";
    const sibling = extension === "tsx" ? "Sibling" : "App";
    fs.writeFileSync(
      path.join(workspacePath, `${component}.${extension}`),
      `import { account } from "./model"; import "./${sibling}";\n${binding}\nexport const view = <p>{account.label}{wrong}</p>;\n`,
    );
  }
}

function writeJson(workspace, filename, value) {
  fs.writeFileSync(path.join(workspace, filename), `${JSON.stringify(value, null, 2)}\n`);
}
