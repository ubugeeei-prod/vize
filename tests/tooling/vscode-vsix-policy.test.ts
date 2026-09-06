import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readText(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

function readJson(relativePath: string): unknown {
  return JSON.parse(readText(relativePath));
}

test("VSIX package policy keeps the production CJS host entrypoint shippable", () => {
  const manifest = readJson("editors/vscode/package.json") as { main?: string; type?: string };
  const tsconfig = readJson("editors/vscode/tsconfig.json") as {
    compilerOptions?: { types?: string[] };
  };
  const packConfig = readText("editors/vscode/vite.config.ts");
  const ignore = readText("editors/vscode/.vscodeignore");
  const smoke = readText("tools/commands/editors/vscode/assert-vsix-package.rs");

  assert.equal(manifest.type, "module");
  assert.equal(manifest.main, "./dist/extension.cjs");
  assert.deepEqual(tsconfig.compilerOptions?.types, ["node"]);
  assert.match(packConfig, /entry:\s*\["src\/extension\.ts"\]/);
  assert.match(packConfig, /outDir:\s*"dist"/);
  assert.match(packConfig, /format:\s*"cjs"/);
  assert.match(packConfig, /platform:\s*"node"/);
  assert.match(packConfig, /neverBundle:\s*\["vscode"\]/);
  assert.doesNotMatch(ignore, /^dist\/?$/m);
  assert.match(smoke, /"extension\/dist\/extension\.cjs"/);
  assert.match(
    smoke,
    /assert_json_string\(&package_json, &\["main"\], "\.\/dist\/extension\.cjs"\)\?/,
  );
  assert.match(smoke, /"exports\.activate="/);
  assert.match(smoke, /"exports\.deactivate="/);
  assert.match(smoke, /"sourceMappingURL="/);
});

test("VSIX TypeScript plugin staging follows the runtime require closure", () => {
  const pluginDir = "editors/vscode/typescript-vue-plugin";
  const sync = readText("tools/commands/editors/vscode/sync-typescript-plugin.rs");
  const smoke = readText("tools/commands/editors/vscode/assert-vsix-package.rs");
  const runtimeFiles = new Set(["package.json"]);
  const pending = ["index.cjs"];

  while (pending.length > 0) {
    const file = pending.pop();
    assert.ok(file);
    if (runtimeFiles.has(file)) {
      continue;
    }
    runtimeFiles.add(file);

    const source = readText(`${pluginDir}/${file}`);
    for (const match of source.matchAll(/require\("\.\/([^"]+\.cjs)"\)/g)) {
      pending.push(match[1]);
    }
  }

  assert.deepEqual([...runtimeFiles].sort(), [
    "component-contracts.cjs",
    "import-resolution.cjs",
    "index.cjs",
    "module-resolution.cjs",
    "package.json",
    "virtual-modules.cjs",
  ]);
  for (const file of runtimeFiles) {
    assert.match(sync, new RegExp(`"${escapeRegExp(file)}"`), `${file} is not staged`);
    assert.match(
      smoke,
      new RegExp(`extension/node_modules/@vizejs/typescript-vue-plugin/${escapeRegExp(file)}`),
      `${file} is not required by VSIX smoke`,
    );
  }
});

test("VSIX package policy excludes source-only extension host inputs", () => {
  const ignore = readText("editors/vscode/.vscodeignore");
  const smoke = readText("tools/commands/editors/vscode/assert-vsix-package.rs");

  for (const pattern of [
    "src/",
    "test/",
    "test-fixtures/",
    ".vscode-test/",
    "tsconfig.json",
    "vite.config.ts",
  ]) {
    assert.match(ignore, new RegExp(`^${escapeRegExp(pattern)}$`, "m"));
  }

  for (const forbidden of [
    "extension/.vscode-test/",
    "extension/src/",
    "extension/test/",
    "extension/test-fixtures/",
    "extension/tsconfig.json",
    "extension/vite.config.ts",
  ]) {
    assert.match(smoke, new RegExp(escapeRegExp(forbidden)));
  }
});

test("VSIX package policy excludes workspace manifests from shipped files", () => {
  assert.match(readText("editors/vscode/.vscodeignore"), /^pnpm-workspace\.yaml$/m);
  assert.match(
    readText("tools/commands/editors/vscode/assert-vsix-package.rs"),
    /name == "extension\/pnpm-lock\.yaml"[\s\S]*name == "extension\/pnpm-workspace\.yaml"/,
  );
});

test("VSIX package policy keeps hidden host-smoke commands out of the manifest", () => {
  const manifest = readJson("editors/vscode/package.json") as {
    activationEvents?: string[];
    contributes?: { commands?: Array<{ command?: string }> };
  };
  const hostTestCore = readText("editors/vscode/src/host-test-core.ts");
  const smoke = readText("tools/commands/editors/vscode/assert-vsix-package.rs");
  const hiddenCommands = [
    ...hostTestCore.matchAll(/export const HOST_TEST_[A-Z_]+_COMMAND = "(vize\.test\.[^"]+)";/g),
  ].map((match) => match[1]);

  assert.deepEqual(hiddenCommands.sort(), [
    "vize.test.executeCompletion",
    "vize.test.executeLspRequest",
    "vize.test.executeReferences",
    "vize.test.getServerInfo",
  ]);
  const contributedCommands = new Set(
    (manifest.contributes?.commands ?? []).map((command) => command.command),
  );
  const activationEvents = new Set(manifest.activationEvents ?? []);
  for (const command of hiddenCommands) {
    assert.equal(contributedCommands.has(command), false);
    assert.equal(activationEvents.has(`onCommand:${command}`), false);
  }
  assert.match(smoke, /assert_no_hidden_host_test_commands/);
  assert.match(smoke, /starts_with\("vize\.test\."\)/);
});

test("VSIX archive reader escapes unzip member globs", () => {
  const reader = readText("tools/support/editors/archive.rs");
  const smoke = readText("tools/commands/editors/vscode/assert-vsix-package.rs");

  assert.match(reader, /\.arg\(unzip_member_pattern\(name\)\)/);
  assert.match(reader, /matches!\(character, '\[' \| '\]' \| '\*' \| '\?' \| '\\\\'\)/);
  assert.match(smoke, /let vsix = absolute_from_cwd\(&vsix\)\?/);
});

function escapeRegExp(source: string): string {
  return source.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
