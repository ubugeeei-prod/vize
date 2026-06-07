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
import { inTestbox } from "./testbox.ts";

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
  // `vp build` runs inside a Blacksmith Testbox; the underlying build:* tasks
  // stay local. See tools/vite-plus/tasks/testbox.ts.
  build: noCacheTask(inTestbox(runTasks("build:rust", "build:all"))),
  "build:all": noCacheTask(runTasks("build:runtime", "package:editor-extensions")),
  "build:rust": task("cargo build --workspace", { input: cacheInputs.rust }),
  "build:runtime": noCacheTask(runTasks("build:native", "build:wasm", "build:packages")),
  "build:packages": noCacheTask(runInPackages("build", packedPackages)),
  "build:native": noCacheTask(runPackageScriptDirectly("build", ["./npm/vize-native"])),
  // Fast variant for test pipelines: dev cargo profile via the local
  // `build:debug` script. We deliberately route through `build:debug`
  // (which wraps `build-local.mjs --no-js`) rather than `build:ci`
  // because `build:ci` regenerates `index.js` / `index.d.ts` and
  // wipes the manual JSON.parse wrappers that the token API depends on.
  // The dev profile shaves ~2 minutes off the release-profile build and
  // matches the profile that vite-plugin-vize already uses at test time,
  // so cargo's incremental cache makes the second invocation a no-op.
  "build:native:test": noCacheTask(runPackageScriptDirectly("build:debug", ["./npm/vize-native"])),
  "build:wasm": task(moonScript("build_vitrine_wasm", "nodejs", "npm/vite-plugin-vize/wasm")),
  "build:wasm-web": task(moonScript("build_vitrine_wasm", "web", "playground/src/wasm")),
  "build:vite-plugin": noCacheTask(
    `${runInPackages("build", ["./npm/vize"])} && ${runInPackages("build", ["./npm/vite-plugin-vize"])}`,
  ),
  "build:nuxt-stack": noCacheTask(
    runInPackages("build", ["./npm/vite-plugin-musea", "./npm/musea-nuxt", "./npm/nuxt"]),
  ),
  "build:plugin": noCacheTask(runTask("build:vite-plugin")),
  "build:cli": task("cargo build --release -p vize"),
  "build:vscode-extension": noCacheTask(runInVscodeExtension("pnpm exec vp pack")),
  "build:editor-extensions": noCacheTask(runTasks("build:vscode-extension", "check:zed-extension")),
  "package:vscode-extension": noCacheTask(
    runInVscodeExtension(
      "pnpm exec vsce package --no-dependencies --out dist/vize.vsix",
      "node ../../tools/vscode-vize/assert-vsix-package.mjs dist/vize.vsix",
    ),
  ),
  "check:zed-extension": task("cargo check --manifest-path npm/zed-vize/Cargo.toml", {
    input: ["npm/zed-vize/**"],
  }),
  "package:zed-extension": noCacheTask(
    "COPYFILE_DISABLE=1 LC_ALL=C LANG=C tar --exclude 'zed-vize/target' -czf zed-vize-extension.tar.gz -C npm zed-vize && node tools/zed-vize/assert-zed-package.mjs zed-vize-extension.tar.gz",
  ),
  "package:nvim-extension": noCacheTask(
    "COPYFILE_DISABLE=1 LC_ALL=C LANG=C tar -czf nvim-vize-extension.tar.gz -C npm nvim-vize && node tools/nvim-vize/assert-nvim-package.mjs nvim-vize-extension.tar.gz",
  ),
  "package:vim-extension": noCacheTask(
    "COPYFILE_DISABLE=1 LC_ALL=C LANG=C tar -czf vim-vize-extension.tar.gz -C npm vim-vize && node tools/vim-vize/assert-vim-package.mjs vim-vize-extension.tar.gz",
  ),
  "package:helix-extension": noCacheTask(
    "COPYFILE_DISABLE=1 LC_ALL=C LANG=C tar -czf helix-vize-extension.tar.gz -C npm helix-vize && node tools/helix-vize/assert-helix-package.mjs helix-vize-extension.tar.gz",
  ),
  "package:emacs-extension": noCacheTask(
    "COPYFILE_DISABLE=1 LC_ALL=C LANG=C tar -czf emacs-vize-extension.tar.gz -C npm emacs-vize && node tools/emacs-vize/assert-emacs-package.mjs emacs-vize-extension.tar.gz",
  ),
  "package:editor-extensions": noCacheTask(
    `${runInVscodeExtension(
      "pnpm exec tsgo --noEmit",
      "pnpm exec vp check src vite.config.ts",
      "pnpm exec vsce package --no-dependencies --out dist/vize.vsix",
      "node ../../tools/vscode-vize/assert-vsix-package.mjs dist/vize.vsix",
    )} && ${runTask("check:zed-extension")} && ${runTask(
      "test:zed-extension:unit",
    )} && ${runTask("package:zed-extension")} && ${runTask(
      "package:nvim-extension",
    )} && ${runTask("package:vim-extension")} && ${runTask(
      "package:helix-extension",
    )} && ${runTask("package:emacs-extension")}`,
  ),
  "install:plugin": noCacheTask("vp install --filter './npm/vite-plugin-vize'"),
});
