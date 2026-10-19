import fs from "node:fs";
import os from "node:os";
import { createRequire } from "node:module";
import path from "node:path";
import {
  prepareRealVueWorkspace,
  resolveCorsaPath,
} from "../../../tools/support/compat/editor-e2e/real-vue-workspace.ts";
import { createRealHostEnvironment, runPackagedExtensionHost } from "./packaged-host-contract.mjs";

export async function runExistingConfigHostScenarios(runCommand, options) {
  for (const mode of ["vite", "tsconfig", "external"]) {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-host-"));
    const workspacePath = path.join(directory, "workspace");
    const extensionsPath = path.join(directory, "extensions");
    try {
      prepareRealVueWorkspace(workspacePath);
      fs.rmSync(path.join(workspacePath, "vite.config.mjs"));
      fs.writeFileSync(
        path.join(workspacePath, "src/Formatting.vue"),
        '<script setup lang="ts">const count=1</script><template><div>{{count}}</div></template>\n',
      );
      const observed =
        mode === "external"
          ? prepareExternalConfigHost(directory, workspacePath, options)
          : undefined;
      fs.writeFileSync(
        path.join(workspacePath, ".vscode/settings.json"),
        JSON.stringify({
          "vize.serverPath": observed?.launcher ?? options.serverPath,
          "chat.disableAIFeatures": true,
        }),
      );
      if (mode === "vite") {
        // Project feature disables must win over the extension's automatic
        // startup profile, while unrelated server defaults remain enabled.
        fs.writeFileSync(
          path.join(workspacePath, "vite.config.ts"),
          `export default ${JSON.stringify({ vue: { languageServer: { hover: false } } })};\n`,
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
          ...(observed
            ? {
                VIZE_TEST_OBSERVED_SERVER_PATH: options.serverPath,
                VIZE_TEST_CONFIG_PROTOCOL_PATH: observed.protocol,
                VIZE_TEST_EXTERNAL_CONFIG_MANIFEST: observed.manifest,
              }
            : {}),
        },
        hostTimeoutMs: 180_000,
        installEnvironment: process.env,
        installTimeoutMs: 120_000,
        userDataPath: path.join(directory, "profile"),
        workspacePath,
      });
    } finally {
      if (mode === "external") retainExternalConfigHost(directory, options.sourceExtensionPath);
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
}

function prepareExternalConfigHost(directory, workspace, options) {
  const write = (file, content) => {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, content);
  };
  const policy = path.join(directory, "共有 policy.mjs");
  const pending = path.join(directory, "new policy.mjs");
  const base = path.join(directory, "tsconfig [共有].json");
  const inherited = path.join(directory, "policy-package/tsconfig.json");
  const a = path.join(workspace, "packages/a");
  const b = path.join(workspace, "packages/b");
  const project = JSON.parse(fs.readFileSync(path.join(workspace, "tsconfig.json"), "utf8"));
  for (const root of [a, b]) {
    write(path.join(root, "package.json"), '{"private":true,"type":"module"}');
    write(path.join(root, "tsconfig.json"), JSON.stringify(project));
    write(
      path.join(root, "vite.config.mjs"),
      "export default { vue: { linter: { enabled: false } } };",
    );
  }
  write(
    path.join(a, "tsconfig.json"),
    JSON.stringify({
      ...project,
      extends: ["../../../tsconfig [共有].json", "@config/base"],
    }),
  );
  write(
    path.join(a, "tsconfig.text.json"),
    JSON.stringify({
      extends: "./tsconfig.json",
      compilerOptions: { paths: { "@policy": [path.join(directory, "types/string.ts")] } },
    }),
  );
  write(
    path.join(directory, "policy-package/package.json"),
    '{"name":"@config/base","tsconfig":"tsconfig.json"}',
  );
  write(inherited, '{"compilerOptions":{"strictNullChecks":true}}');
  fs.mkdirSync(path.join(a, "node_modules/@config"), { recursive: true });
  fs.symlinkSync(path.dirname(inherited), path.join(a, "node_modules/@config/base"), "junction");
  write(
    base,
    JSON.stringify({
      compilerOptions: { baseUrl: ".", paths: { "@policy": ["./types/number.ts"] } },
    }),
  );
  write(path.join(directory, "types/number.ts"), "export type Policy = number;\n");
  write(path.join(directory, "types/string.ts"), "export type Policy = string;\n");
  write(
    policy,
    "export default { languageServer: { hover: false }, linter: { enabled: false } };\n",
  );
  const vite = path.join(a, "vite.config.mjs");
  write(vite, `import vue from '../../../共有 policy.mjs'; export default { vue };\n`);
  write(
    path.join(a, "src/Probe.vue"),
    '<script setup lang="ts">\nimport type { Policy } from "@policy";\nconst typed: Policy = 42;\nconst nullable: string = null;\n</script>\n<template>{{ typed }}{{ nullable }}</template>\n',
  );
  write(
    path.join(b, "src/Control.vue"),
    '<script setup lang="ts">\nconst stable = "unchanged";\n</script>\n<template>{{ stable }}</template>\n',
  );
  write(
    path.join(workspace, "vite.config.mjs"),
    "export default { vue: { linter: { enabled: false } } };\n",
  );
  const require = createRequire(import.meta.url);
  fs.symlinkSync(
    path.dirname(require.resolve("vite/package.json")),
    path.join(workspace, "node_modules/vite"),
    "junction",
  );
  const protocol = path.join(directory, "protocol.jsonl");
  const manifest = path.join(directory, "campaign.json");
  write(manifest, JSON.stringify({ policy, pending, base, inherited, a, b, vite }));
  const launcher = path.join(directory, "vize-observed");
  const proxy = path.join(options.sourceExtensionPath, "test/fixtures/passive-vize-server.cjs");
  write(launcher, `#!${process.execPath}\nrequire(${JSON.stringify(proxy)});\n`);
  fs.chmodSync(launcher, 0o755);
  return { launcher, protocol, manifest };
}

function retainExternalConfigHost(directory, extension) {
  const destination = process.env.VIZE_REAL_HOST_ARTIFACT_DIR
    ? path.join(process.env.VIZE_REAL_HOST_ARTIFACT_DIR, "external-config")
    : path.join(extension, ".vscode-test/external-config-evidence");
  fs.mkdirSync(destination, { recursive: true });
  for (const name of ["protocol.jsonl", "campaign.json"]) {
    const source = path.join(directory, name);
    if (fs.existsSync(source)) fs.copyFileSync(source, path.join(destination, name));
  }
  fs.cpSync(directory, path.join(destination, "authored"), {
    recursive: true,
    filter: (file) =>
      !["node_modules", "extensions", "profile"].some((name) =>
        file.split(path.sep).includes(name),
      ),
  });
}
