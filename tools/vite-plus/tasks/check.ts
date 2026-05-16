import {
  cacheInputs,
  checkedPackages,
  checkedPackagesViaVpRun,
  ciCheckedPackages,
  directCheckPackages,
} from "../task-inputs.ts";
import {
  defineTasks,
  localVp,
  noCacheTask,
  runInDirectory,
  runInPackages,
  runInVscodeExtension,
  runPackageScriptDirectly,
  runTask,
  runTasks,
  task,
} from "../task-helpers.ts";

const ciPackageCheckCommand = runInPackages("check", ciCheckedPackages, {
  concurrencyLimit: 1,
});
const directPackageCheckCommand = runPackageScriptDirectly("check", directCheckPackages);
const ciVizeAppCheckCommand = [
  runInDirectory(
    "./examples/vite-musea",
    "vize check && vp check src vite.config.ts vite.app.config.ts vize.config.ts playwright.config.ts",
  ),
  runInDirectory(
    "./playground",
    "vp check src 'e2e/*.ts' 'e2e/vrt/*.ts' vite.config.ts vite.app.config.ts vite.test.config.ts vite.node.config.ts playwright.config.ts && vize lint --max-warnings 0",
  ),
].join(" && ");

/**
 * Repository-wide formatting, linting, package checks, and CI aggregate tasks.
 *
 * The Vite+ task graph deliberately separates cached checks from non-cached
 * commands so local development can stay fast while CI remains strict. Package
 * checks that run through `vp run` are throttled to avoid CPU-heavy production
 * builds competing with each other on Windows runners.
 */
export const checkTasks = defineTasks({
  check: noCacheTask(
    runTasks("check:repo", "check:rust", "check:js", "check:vize-apps", "check:editor-extensions"),
  ),
  "check:js": noCacheTask(runTask("check:js:packages")),
  "check:js:packages": task(
    runInPackages("check", checkedPackagesViaVpRun, { concurrencyLimit: 1 }),
    {
      input: cacheInputs.jsChecks,
    },
  ),
  "check:vize-apps": noCacheTask(directPackageCheckCommand),
  "check:ci:vize-apps": noCacheTask(ciVizeAppCheckCommand),
  "check:repo": noCacheTask(`${localVp} check`),
  // The oxlint example intentionally exits non-zero for its default lint script,
  // so CI checks every package except that runnable failure-case fixture.
  "check:ci": noCacheTask(`${runTask("check:repo")} && ${ciPackageCheckCommand}`),
  "check:fix": noCacheTask(runInPackages("check:fix", checkedPackages)),
  "check:rust": noCacheTask("cargo check --workspace"),
  "check:vscode-extension": noCacheTask(
    runInVscodeExtension("pnpm exec tsgo --noEmit", "pnpm exec vp check src vite.config.ts"),
  ),
  "check:editor-extensions": noCacheTask(runTasks("check:vscode-extension", "check:zed-extension")),
  clippy: task("cargo clippy --workspace -- -D warnings", { input: cacheInputs.rust }),
  fmt: noCacheTask(runTasks("fmt:repo", "fmt:rust", "fmt:js")),
  "fmt:repo": noCacheTask(`${localVp} fmt --write`),
  "fmt:js": noCacheTask(runInPackages("fmt", checkedPackages)),
  "fmt:rust": task("cargo fmt --all", { input: cacheInputs.rust }),
  "fmt:all": noCacheTask(runTask("fmt")),
  lint: noCacheTask(runTask("check")),
  "lint:fix": noCacheTask(runTask("check:fix")),
  "lint:rust": task("cargo clippy --workspace -- -D warnings", { input: cacheInputs.rust }),
  "lint:all": noCacheTask(runTasks("lint:rust", "check")),
  "fmt:check": noCacheTask(runTask("check")),
  ci: noCacheTask(runTasks("fmt:all", "clippy", "test", "check:ci")),
});
