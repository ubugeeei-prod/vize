import {
  steps,
  defineTasks,
  installVscodeExtensionDependencies,
  nodeTool,
  noCacheTask,
  runInPackages,
  runTask,
  runTasks,
} from "../task-helpers.ts";

/** Runs a TypeScript release command with Node's native type stripping. */
const releaseCommand = (name: string, ...args: string[]) =>
  ["node", `tools/support/release/${name}/main.ts`, ...args].join(" ");

export const releaseTasks = defineTasks({
  // The release command owns argument parsing and confirmation before mutation.
  release: noCacheTask(releaseCommand("release", "dispatch")),
  "publish:wasm": noCacheTask(
    steps(
      nodeTool("tasks/build-vize-wasm-package"),
      releaseCommand("publish_npm_package", "npm/wasm"),
    ),
  ),
  "publish:native": noCacheTask(
    steps(runTask("build:native"), releaseCommand("publish_npm_package", "npm/native")),
  ),
  "publish:vite-plugin": noCacheTask(
    steps(runTask("build:vite-plugin"), releaseCommand("publish_npm_package", "npm/builder/vite")),
  ),
  "publish:oxlint-plugin": noCacheTask(
    steps(
      runInPackages("build", ["./npm/linter/oxlint"]),
      releaseCommand(
        "inject_native_optional_deps",
        "npm/linter/oxlint/package.json",
        "npm/native/package.json",
      ),
      releaseCommand("publish_npm_package", "npm/linter/oxlint"),
    ),
  ),
  "publish:npm": noCacheTask(
    runTasks("publish:wasm", "publish:native", "publish:vite-plugin", "publish:oxlint-plugin"),
  ),
  "publish:crates": noCacheTask(releaseCommand("publish_crates")),
  "publish:vscode-extension": noCacheTask(
    steps(installVscodeExtensionDependencies, {
      program: "node",
      arguments: [
        "tools/support/release/publish_vscode_extension/main.ts",
        "editors/vscode/dist/vize.vsix",
      ],
      environment: { VSCE_DLX_BIN: "corepack" },
    }),
  ),
  publish: noCacheTask(runTasks("publish:npm", "publish:crates")),
});
