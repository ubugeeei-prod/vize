import {
  defineTasks,
  installVscodeExtensionDependencies,
  moonScript,
  noCacheTask,
  runInPackages,
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
  release: noCacheTask(moonScript("release")),
  "publish:wasm": noCacheTask(
    `${moonScript("build_vize_wasm_package")} && ${moonScript("publish_npm_package", "npm/vize-wasm")}`,
  ),
  "publish:native": noCacheTask(
    `${runTask("build:native")} && ${moonScript("publish_npm_package", "npm/vize-native")}`,
  ),
  "publish:vite-plugin": noCacheTask(
    `${runTask("build:vite-plugin")} && ${moonScript("publish_npm_package", "npm/vite-plugin-vize")}`,
  ),
  "publish:oxlint-plugin": noCacheTask(
    `${runInPackages("build", ["./npm/oxlint-plugin-vize"])} && ${moonScript("inject_native_optional_deps", "npm/oxlint-plugin-vize/package.json", "npm/vize-native/package.json")} && ${moonScript("publish_npm_package", "npm/oxlint-plugin-vize")}`,
  ),
  "publish:npm": noCacheTask(
    runTasks("publish:wasm", "publish:native", "publish:vite-plugin", "publish:oxlint-plugin"),
  ),
  "publish:crates": noCacheTask(moonScript("publish_crates")),
  "publish:vscode-extension": noCacheTask(
    `${installVscodeExtensionDependencies} && ${moonScript("publish_vscode_extension", "npm/vscode-vize/dist/vize.vsix")}`,
  ),
  publish: noCacheTask(runTasks("publish:npm", "publish:crates")),
});
