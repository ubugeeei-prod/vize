import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const extensionDir = path.join(root, "editors/vscode");
const sourceDir = path.join(extensionDir, "typescript-vue-plugin");
const packagePath = "node_modules/@vizejs/typescript-vue-plugin";
const pluginFiles = ["index.cjs", "package.json", "virtual-modules.cjs"];

const command = process.argv[2];

if (command === "stage") {
  stagePlugin(path.join(extensionDir, packagePath));
} else if (command === "inject") {
  const vsixPath = path.resolve(process.cwd(), process.argv[3] ?? "dist/vize.vsix");
  injectPlugin(vsixPath);
} else {
  console.error("Usage: sync-typescript-plugin.mjs <stage|inject> [vsix]");
  process.exit(2);
}

function stagePlugin(targetDir) {
  fs.rmSync(targetDir, { force: true, recursive: true });
  fs.mkdirSync(targetDir, { recursive: true });
  for (const file of pluginFiles) {
    fs.copyFileSync(path.join(sourceDir, file), path.join(targetDir, file));
  }
}

function injectPlugin(vsixPath) {
  if (!fs.existsSync(vsixPath)) {
    throw new Error(`VSIX does not exist: ${vsixPath}`);
  }

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vsix-plugin-"));
  try {
    const targetDir = path.join(tempDir, "extension", packagePath);
    stagePlugin(targetDir);
    const entries = pluginFiles.map((file) => path.posix.join("extension", packagePath, file));
    const result = spawnSync("zip", ["-X", "-q", vsixPath, ...entries], {
      cwd: tempDir,
      encoding: "utf8",
    });
    if (result.error) {
      throw result.error;
    }
    if (result.status !== 0) {
      throw new Error(
        `zip failed with status ${result.status ?? "null"}\n${result.stderr}${result.stdout}`,
      );
    }
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
}
