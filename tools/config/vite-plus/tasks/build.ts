import { cacheInputs, packedPackages } from "../task-inputs.ts";
import {
  assertVscodePackage,
  packageVscodeExtension,
  stageVscodeTypeScriptPlugin,
  steps,
  defineTasks,
  nodeCommand,
  nodeTool,
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

const localBuildCommand = runTasks("build:rust", "build:all");

function editorArchive(editor: string, verify: string, options: string[] = []) {
  const archive = editor + "-vize-extension.tar.gz";
  return noCacheTask(
    steps(
      {
        program: "tar",
        arguments: [...options, "-czf", archive, "-C", "editors", editor],
        environment: { COPYFILE_DISABLE: "1", LC_ALL: "C", LANG: "C" },
      },
      nodeCommand(verify, archive),
    ),
  );
}

export const buildTasks = defineTasks({
  // Local is the default so a clean checkout never needs hosted credentials.
  // Testbox remains an explicit CI-parity opt-in with the same task graph.
  build: noCacheTask(localBuildCommand),
  "build:testbox": noCacheTask(inTestbox("build")),
  "build:all": noCacheTask(runTasks("build:runtime", "package:editor-extensions")),
  "build:rust": task("cargo build --workspace", { input: cacheInputs.rust }),
  "build:runtime": noCacheTask(runTasks("build:native", "build:wasm", "build:packages")),
  "build:packages": noCacheTask(runInPackages("build", packedPackages)),
  "build:native": noCacheTask(runPackageScriptDirectly("build", ["./npm/native"])),
  // The debug build preserves the token API wrappers in index.js and is reused by test:js.
  "build:native:test": noCacheTask(runPackageScriptDirectly("build:debug", ["./npm/native"])),
  "build:wasm": task(nodeTool("tasks/build-vitrine-wasm", "nodejs", "npm/builder/vite/wasm")),
  "build:wasm-web": task(nodeTool("tasks/build-vitrine-wasm", "web", "playground/src/wasm")),
  "build:vite-plugin": noCacheTask(
    steps(runInPackages("build", ["./npm/cli"]), runInPackages("build", ["./npm/builder/vite"])),
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
    runInVscodeExtension(packageVscodeExtension, assertVscodePackage),
  ),
  "check:zed-extension": task("cargo check --manifest-path editors/zed/Cargo.toml", {
    input: ["editors/zed/**"],
  }),
  "package:zed-extension": editorArchive("zed", "editors/zed/assert-zed-package", [
    "--exclude",
    "zed/target",
  ]),
  "package:nvim-extension": editorArchive("nvim", "editors/neovim/assert-nvim-package"),
  "package:vim-extension": editorArchive("vim", "editors/vim/assert-vim-package"),
  "package:helix-extension": editorArchive("helix", "editors/helix/assert-helix-package"),
  "package:emacs-extension": editorArchive("emacs", "editors/emacs/assert-emacs-package"),
  "package:editor-extensions": noCacheTask(
    steps(
      runInVscodeExtension(
        vscodeExtensionPackageBin("typescript", "tsc", "--noEmit"),
        vscodeExtensionPackageBin("vite-plus", "vp", "check", "src", "vite.config.ts"),
        packageVscodeExtension,
        assertVscodePackage,
      ),
      runTasks(
        "check:zed-extension",
        "test:zed-extension:unit",
        "package:zed-extension",
        "package:nvim-extension",
        "package:vim-extension",
        "package:helix-extension",
        "package:emacs-extension",
      ),
    ),
  ),
  "install:plugin": noCacheTask("vp install --filter './npm/builder/vite'"),
});
