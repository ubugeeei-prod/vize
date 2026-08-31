import {
  defineTasks,
  installVscodeExtensionDependencies,
  moonScript,
  moonScriptWithFreshRegistry,
  noCacheTask,
  runInPackages,
  rustTool,
  runTask,
  runTasks,
} from "../task-helpers.ts";

/**
 * Publication tasks for npm packages, crates, and editor extensions.
 *
 * Release automation is intentionally isolated from build and check commands so
 * publishing can evolve without touching the day-to-day developer task graph.
 * Each target names the artifact boundary it publishes, which keeps the root
 * catalog readable even as the repository gains more package families.
 */
export const releaseTasks = defineTasks({
  release: noCacheTask(moonScriptWithFreshRegistry("release", '"$@"'), {
    forwardArguments: true,
  }),
  "publish:wasm": noCacheTask(
    `${moonScript("build_vize_wasm_package")} && ${moonScript("publish_npm_package", "npm/wasm")}`,
  ),
  "publish:native": noCacheTask(
    `${runTask("build:native")} && ${moonScript("publish_npm_package", "npm/native")}`,
  ),
  "publish:vite-plugin": noCacheTask(
    `${runTask("build:vite-plugin")} && ${moonScript("publish_npm_package", "npm/builder/vite")}`,
  ),
  "publish:oxlint-plugin": noCacheTask(
    `${runInPackages("build", ["./npm/oxlint"])} && ${rustTool("release/npm/inject-native-optional-deps", "npm/oxlint/package.json", "npm/native/package.json")} && ${moonScript("publish_npm_package", "npm/oxlint")}`,
  ),
  "publish:npm": noCacheTask(
    runTasks("publish:wasm", "publish:native", "publish:vite-plugin", "publish:oxlint-plugin"),
  ),
  "publish:crates": noCacheTask(moonScript("publish_crates")),
  "publish:vscode-extension": noCacheTask(
    `${installVscodeExtensionDependencies} && ${moonScript("publish_vscode_extension", "editors/vscode/dist/vize.vsix")}`,
  ),
  publish: noCacheTask(runTasks("publish:npm", "publish:crates")),
});
