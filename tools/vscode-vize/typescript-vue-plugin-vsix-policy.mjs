import assert from "node:assert/strict";

const packageName = "@vizejs/typescript-vue-plugin";
const entryRoot = `extension/node_modules/${packageName}`;

export const typescriptVuePluginRequiredFiles = [
  `${entryRoot}/index.cjs`,
  `${entryRoot}/package.json`,
];

export const typescriptVuePluginAllowedEntry =
  /^extension\/node_modules\/@vizejs\/typescript-vue-plugin\/(?:index\.cjs|package\.json)$/;

export const forbiddenNonPluginNodeModules =
  /^extension\/node_modules\/(?!@vizejs\/typescript-vue-plugin\/(?:index\.cjs|package\.json)$)/;

export function assertTypeScriptVuePluginPackage({ packageJson, readJsonEntry, readTextEntry }) {
  assert.deepEqual(packageJson.contributes?.typescriptServerPlugins, [
    {
      enableForWorkspaceTypeScriptVersions: true,
      name: packageName,
    },
  ]);

  const pluginPackage = readJsonEntry(`${entryRoot}/package.json`);
  assert.equal(pluginPackage.name, packageName);
  assert.equal(pluginPackage.main, "index.cjs");
  assert.match(readTextEntry(`${entryRoot}/index.cjs`), /function init\(\{ typescript: ts \}\)/);
}
