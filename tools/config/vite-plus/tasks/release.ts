import {
  defineTasks,
  installVscodeExtensionDependencies,
  nodeTool,
  noCacheTask,
  runInPackages,
  rustTool,
  runTask,
  runTasks,
} from "../task-helpers.ts";

/** Runs a TypeScript release command with Node's native type stripping. */
const releaseCommand = (name: string, ...args: string[]) =>
  ["node", `tools/support/release/${name}/main.ts`, ...args].join(" ");

/**
 * Publication tasks for npm packages, crates, and editor extensions.
 *
 * Release automation is intentionally isolated from build and check commands so
 * publishing can evolve without touching the day-to-day developer task graph.
 * Each target names the artifact boundary it publishes, which keeps the root
 * catalog readable even as the repository gains more package families.
 */
export const releaseTasks = defineTasks({
  // The release command owns argument parsing and confirmation before mutation.
  release: noCacheTask(releaseCommand("release", "dispatch", '"$@"'), {
    forwardArguments: true,
  }),
  "publish:wasm": noCacheTask(
    `${nodeTool("tasks/build-vize-wasm-package")} && ${releaseCommand("publish_npm_package", "npm/wasm")}`,
  ),
  "publish:native": noCacheTask(
    `${runTask("build:native")} && ${releaseCommand("publish_npm_package", "npm/native")}`,
  ),
  "publish:vite-plugin": noCacheTask(
    `${runTask("build:vite-plugin")} && ${releaseCommand("publish_npm_package", "npm/builder/vite")}`,
  ),
  "publish:oxlint-plugin": noCacheTask(
    `${runInPackages("build", ["./npm/oxlint"])} && ${rustTool("release/npm/inject-native-optional-deps", "npm/oxlint/package.json", "npm/native/package.json")} && ${releaseCommand("publish_npm_package", "npm/oxlint")}`,
  ),
  "publish:npm": noCacheTask(
    runTasks("publish:wasm", "publish:native", "publish:vite-plugin", "publish:oxlint-plugin"),
  ),
  "publish:crates": noCacheTask(releaseCommand("publish_crates")),
  "publish:vscode-extension": noCacheTask(
    `${installVscodeExtensionDependencies} && VSCE_DLX_BIN=corepack ${releaseCommand("publish_vscode_extension", "editors/vscode/dist/vize.vsix")}`,
  ),
  publish: noCacheTask(runTasks("publish:npm", "publish:crates")),
});
