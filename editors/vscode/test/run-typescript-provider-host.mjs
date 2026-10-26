import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runVSCodeCommand } from "@vscode/test-electron";
import { runPackagedExtensionHost } from "./packaged-host-contract.mjs";

const extensionPath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const vsixPath = path.resolve(process.env.VIZE_TYPESCRIPT_PROVIDER_VSIX);
const evidence = path.resolve(process.env.VIZE_TYPESCRIPT_PROVIDER_CAPTURE);
const source = execFileSync("git", ["rev-parse", "HEAD"], {
  cwd: extensionPath,
  encoding: "utf8",
}).trim();
const producer = JSON.parse(fs.readFileSync(`${vsixPath}.receipt.json`, "utf8"));
assert.equal(producer.source, source);
const hash = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
assert.equal(hash(vsixPath), producer.sha256);
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ts-native-"));
const workspace = path.join(profile, "workspace 日本語 with spaces");
const userData = path.join(profile, "user-data");
fs.mkdirSync(path.join(workspace, "src"), { recursive: true });
fs.mkdirSync(path.join(userData, "User"), { recursive: true });
fs.mkdirSync(evidence, { recursive: true });
fs.writeFileSync(
  path.join(userData, "User/settings.json"),
  JSON.stringify({
    "extensions.autoUpdate": false,
    "extensions.autoCheckUpdates": false,
    "update.mode": "none",
    "typescript.tsserver.log": "verbose",
    "vize.enable": false,
  }),
);
fs.writeFileSync(
  path.join(workspace, "package.json"),
  JSON.stringify({
    private: true,
    name: "vize-native-host-consumer",
    dependencies: { vue: "3.5.35" },
  }),
);
const npm = [
  process.env.npm_execpath,
  path.resolve(path.dirname(process.execPath), "../lib/node_modules/npm/bin/npm-cli.js"),
  path.resolve(path.dirname(process.execPath), "node_modules/npm/bin/npm-cli.js"),
].find((candidate) => candidate && fs.existsSync(candidate));
assert(npm, "Managed Node must provide npm");
execFileSync(process.execPath, [npm, "install", "--ignore-scripts", "--no-audit", "--no-fund"], {
  cwd: workspace,
  stdio: "inherit",
});
const marker = path.join(workspace, "workspace-native-executed.txt");
const trap = path.join(workspace, "node_modules/@vizejs/native");
fs.mkdirSync(trap, { recursive: true });
fs.writeFileSync(
  path.join(trap, "package.json"),
  JSON.stringify({ name: "@vizejs/native", version: "0.0.0", main: "index.cjs" }),
);
fs.writeFileSync(
  path.join(trap, "index.cjs"),
  `require('node:fs').writeFileSync(${JSON.stringify(marker)}, 'executed'); throw new Error('workspace native override executed');\n`,
);
fs.writeFileSync(
  path.join(workspace, "tsconfig.json"),
  JSON.stringify({
    compilerOptions: {
      strict: true,
      module: "ESNext",
      moduleResolution: "Bundler",
      target: "ES2022",
      skipLibCheck: true,
      noEmit: true,
      plugins: [{ name: "@vizejs/typescript-vue-plugin", nativePath: trap }],
    },
    include: ["src"],
  }),
);
const main =
  'import App from "./App.vue";\nimport Missing from "./Missing.vue";\nconst props: InstanceType<typeof App>["$props"] = {};\nprops.title; Missing;\n';
fs.writeFileSync(path.join(workspace, "src/main.ts"), main);
fs.writeFileSync(
  path.join(workspace, "src/App.vue"),
  '<script setup lang="ts">\ndefineProps<{ title: string }>();\n</script>\n<template />\n',
);
const resultPath = path.join(evidence, "host-result.json");
const manifest = JSON.parse(fs.readFileSync(path.join(extensionPath, "package.json"), "utf8"));
try {
  const installed = await runPackagedExtensionHost(runVSCodeCommand, {
    extensionId: `${manifest.publisher}.${manifest.name}`,
    extensionsPath: path.join(profile, "extensions"),
    extensionTestsPath: path.join(extensionPath, "test/suite/typescript-provider-host.cjs"),
    hostEnvironment: { ...process.env, VIZE_TYPESCRIPT_PROVIDER_RESULT: resultPath },
    hostTimeoutMs: 120_000,
    installEnvironment: process.env,
    installTimeoutMs: 60_000,
    onOutput: (output) => {
      if (output) process.stdout.write(String(output));
    },
    userDataPath: userData,
    vscodeVersion: "1.107.1",
    vsixPath,
    workspacePath: workspace,
  });
  const installedProvider = path.join(installed, "native/typescript-projection");
  for (const file of producer.provider.files)
    assert.equal(hash(path.join(installedProvider, file.path)), file.sha256);
  const result = JSON.parse(fs.readFileSync(resultPath, "utf8"));
  assert.equal(path.relative(installed, result.extensionPath), "");
  assert.equal(
    fs.existsSync(marker),
    false,
    "The bundled service must not execute the workspace override",
  );
  const logs = [];
  function walk(directory) {
    if (!fs.existsSync(directory)) return;
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) walk(file);
      else if (/tsserver.*\.(?:log|txt)$/.test(entry.name)) logs.push(file);
    }
  }
  walk(path.join(userData, "logs"));
  assert(logs.length > 0, "Retain the actual built-in TypeScript server packets");
  const text = logs.map((file) => fs.readFileSync(file, "utf8")).join("\n");
  assert(
    text.includes('[vize] TypeScript native provider {"origin":"bundled"'),
    "tsserver must report its loaded bundled provider",
  );
  assert(text.includes(producer.provider.nativeBuildToken));
  assert(text.includes(producer.provider.source));
  const sdk = /Version:\s*(\S+)/.exec(text)?.[1];
  assert(
    sdk && /^(5\.|6\.)/.test(sdk),
    `Measure the current JavaScript TypeScript host SDK: ${sdk}`,
  );
  fs.writeFileSync(
    path.join(evidence, "receipt.json"),
    JSON.stringify(
      {
        source,
        vsixSha256: hash(vsixPath),
        platform: producer.target,
        nativeBuildToken: producer.provider.nativeBuildToken,
        nativeImageSha256: producer.provider.files.find((file) => file.path.endsWith(".node"))
          .sha256,
        vscode: "1.107.1",
        hostTypeScript: sdk,
        result,
        passed: true,
      },
      null,
      2,
    ),
  );
} finally {
  const logs = path.join(userData, "logs");
  if (fs.existsSync(logs)) fs.cpSync(logs, path.join(evidence, "vscode-logs"), { recursive: true });
  fs.copyFileSync(
    path.join(workspace, "package-lock.json"),
    path.join(evidence, "workspace-package-lock.json"),
  );
}
