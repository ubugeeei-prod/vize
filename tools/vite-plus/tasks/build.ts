import { cacheInputs, packedPackages } from "../task-inputs.ts";
import {
  defineTasks,
  moonScript,
  noCacheTask,
  runInPackages,
  runInVscodeExtension,
  runPackageScriptDirectly,
  runTask,
  runTasks,
  task,
} from "../task-helpers.ts";

/**
 * Build and packaging tasks for the repository's compiled artifacts.
 *
 * This group owns the expensive production-oriented work: Rust workspace
 * builds, npm package packing, WASM generation, and editor extension bundles.
 * Keeping those targets together makes dependency edges such as
 * `build:editor-extensions` -> `check:zed-extension` easy to audit without
 * forcing unrelated test or release commands into the same module.
 */
export const buildTasks = defineTasks({
  build: noCacheTask(runTasks("build:rust", "build:all")),
  "build:all": noCacheTask(runTasks("build:runtime", "package:editor-extensions")),
  "build:rust": task("cargo build --workspace", { input: cacheInputs.rust }),
  "build:runtime": noCacheTask(runTasks("build:native", "build:wasm", "build:packages")),
  "build:packages": noCacheTask(runInPackages("build", packedPackages)),
  "build:native": noCacheTask(runPackageScriptDirectly("build", ["./npm/vize-native"])),
  "build:wasm": task(moonScript("build_vitrine_wasm", "nodejs", "npm/vite-plugin-vize/wasm")),
  "build:wasm-web": task(moonScript("build_vitrine_wasm", "web", "playground/src/wasm")),
  "build:vite-plugin": noCacheTask(
    `${runInPackages("build", ["./npm/vize"])} && ${runInPackages("build", ["./npm/vite-plugin-vize"])}`,
  ),
  "build:plugin": noCacheTask(runTask("build:vite-plugin")),
  "build:cli": task("cargo build --release -p vize"),
  "build:vscode-extension": noCacheTask(runInVscodeExtension("pnpm exec vp pack")),
  "build:editor-extensions": noCacheTask(runTasks("build:vscode-extension", "check:zed-extension")),
  "package:vscode-extension": noCacheTask(
    runInVscodeExtension("pnpm exec vsce package --no-dependencies --out dist/vize.vsix"),
  ),
  "check:zed-extension": task("cargo check --manifest-path npm/zed-vize/Cargo.toml", {
    input: ["npm/zed-vize/**"],
  }),
  "package:zed-extension": noCacheTask(
    "tar --exclude 'zed-vize/target' -czf zed-vize-extension.tar.gz -C npm zed-vize",
  ),
  "package:editor-extensions": noCacheTask(
    `${runInVscodeExtension(
      "pnpm exec tsgo --noEmit",
      "pnpm exec vp check src vite.config.ts",
      "pnpm exec vsce package --no-dependencies --out dist/vize.vsix",
    )} && ${runTask("check:zed-extension")} && ${runTask("package:zed-extension")}`,
  ),
  "install:plugin": noCacheTask("vp install --filter './npm/vite-plugin-vize'"),
});
