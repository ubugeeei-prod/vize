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
  vscodeExtensionPackageBin,
} from "../task-helpers.ts";
import { inTestbox } from "./testbox.ts";

const stageVscodeTypeScriptPlugin = "node ../../tools/vscode-vize/sync-typescript-plugin.mjs stage";
const injectVscodeTypeScriptPlugin =
  "node ../../tools/vscode-vize/sync-typescript-plugin.mjs inject dist/vize.vsix";
const packageVscodeExtension = [
  stageVscodeTypeScriptPlugin,
  `${vscodeExtensionPackageBin("@vscode/vsce", "vsce")} package --no-dependencies --out dist/vize.vsix`,
  injectVscodeTypeScriptPlugin,
].join(" && ");
const localBuildCommand = runTasks("build:rust", "build:all");

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
  // Local is the default so a clean checkout never needs hosted credentials.
  // Testbox remains an explicit CI-parity opt-in with the same task graph.
  build: noCacheTask(localBuildCommand),
  "build:testbox": noCacheTask(inTestbox(localBuildCommand)),
  "build:all": noCacheTask(runTasks("build:runtime", "package:editor-extensions")),
  "build:rust": task("cargo build --workspace", { input: cacheInputs.rust }),
  "build:runtime": noCacheTask(runTasks("build:native", "build:wasm", "build:packages")),
  "build:packages": noCacheTask(runInPackages("build", packedPackages)),
  "build:native": noCacheTask(runPackageScriptDirectly("build", ["./npm/native"])),
  // Fast variant for test pipelines: dev cargo profile via the local
  // `build:debug` script. We deliberately route through `build:debug`
  // (which wraps `build-local.mjs --no-js`) rather than `build:ci`
  // because `build:ci` regenerates `index.js` / `index.d.ts` and
  // wipes the manual JSON.parse wrappers that the token API depends on.
  // The dev profile shaves ~2 minutes off the release-profile build and
  // matches the profile that vite-plugin-vize already uses at test time,
  // so cargo's incremental cache makes the second invocation a no-op.
  "build:native:test": noCacheTask(runPackageScriptDirectly("build:debug", ["./npm/native"])),
  "build:wasm": task(moonScript("build_vitrine_wasm", "nodejs", "npm/builder/vite/wasm")),
  "build:wasm-web": task(moonScript("build_vitrine_wasm", "web", "playground/src/wasm")),
  "build:vite-plugin": noCacheTask(
    `${runInPackages("build", ["./npm/cli"])} && ${runInPackages("build", ["./npm/builder/vite"])}`,
  ),
  "build:nuxt-stack": noCacheTask(
    runInPackages("build", [
      "./npm/builder/vite-musea",
      "./npm/framework/musea-nuxt",
      "./npm/framework/nuxt-lint-config",
      "./npm/framework/nuxt",
    ]),
  ),
  "build:plugin": noCacheTask(runTask("build:vite-plugin")),
  "build:cli": task("cargo build --release -p vize"),
  "build:vscode-extension": noCacheTask(
    runInVscodeExtension(
      stageVscodeTypeScriptPlugin,
      `${vscodeExtensionPackageBin("vite-plus", "vp")} pack`,
    ),
  ),
  "build:editor-extensions": noCacheTask(runTasks("build:vscode-extension", "check:zed-extension")),
  "package:vscode-extension": noCacheTask(
    runInVscodeExtension(
      packageVscodeExtension,
      "node ../../tools/vscode-vize/assert-vsix-package.mjs dist/vize.vsix",
    ),
  ),
  "check:zed-extension": task("cargo check --manifest-path editors/zed/Cargo.toml", {
    input: ["editors/zed/**"],
  }),
  "package:zed-extension": noCacheTask(
    "COPYFILE_DISABLE=1 LC_ALL=C LANG=C tar --exclude 'zed/target' -czf zed-vize-extension.tar.gz -C editors zed && node tools/zed-vize/assert-zed-package.mjs zed-vize-extension.tar.gz",
  ),
  "package:nvim-extension": noCacheTask(
    "COPYFILE_DISABLE=1 LC_ALL=C LANG=C tar -czf nvim-vize-extension.tar.gz -C editors nvim && node tools/nvim-vize/assert-nvim-package.mjs nvim-vize-extension.tar.gz",
  ),
  "package:vim-extension": noCacheTask(
    "COPYFILE_DISABLE=1 LC_ALL=C LANG=C tar -czf vim-vize-extension.tar.gz -C editors vim && node tools/vim-vize/assert-vim-package.mjs vim-vize-extension.tar.gz",
  ),
  "package:helix-extension": noCacheTask(
    "COPYFILE_DISABLE=1 LC_ALL=C LANG=C tar -czf helix-vize-extension.tar.gz -C editors helix && node tools/helix-vize/assert-helix-package.mjs helix-vize-extension.tar.gz",
  ),
  "package:emacs-extension": noCacheTask(
    "COPYFILE_DISABLE=1 LC_ALL=C LANG=C tar -czf emacs-vize-extension.tar.gz -C editors emacs && node tools/emacs-vize/assert-emacs-package.mjs emacs-vize-extension.tar.gz",
  ),
  "package:editor-extensions": noCacheTask(
    `${runInVscodeExtension(
      `${vscodeExtensionPackageBin("typescript", "tsc")} --noEmit`,
      `${vscodeExtensionPackageBin("vite-plus", "vp")} check src vite.config.ts`,
      packageVscodeExtension,
      "node ../../tools/vscode-vize/assert-vsix-package.mjs dist/vize.vsix",
    )} && ${runTask("check:zed-extension")} && ${runTask(
      "test:zed-extension:unit",
    )} && ${runTask("package:zed-extension")} && ${runTask(
      "package:nvim-extension",
    )} && ${runTask("package:vim-extension")} && ${runTask(
      "package:helix-extension",
    )} && ${runTask("package:emacs-extension")}`,
  ),
  "install:plugin": noCacheTask("vp install --filter './npm/builder/vite'"),
});
