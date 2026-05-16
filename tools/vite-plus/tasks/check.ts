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

/**
 * Repository-wide formatting, linting, package checks, and CI aggregate tasks.
 *
 * The Vite+ task graph deliberately separates cached checks from non-cached
 * commands so local development can stay fast while CI remains strict. Package
 * checks that run through `vp run` are throttled to avoid CPU-heavy production
 * builds competing with each other on Windows runners.
 */
export const checkTasks = defineTasks({
  check: noCacheTask(runTasks("check:repo", "check:rust", "check:js", "check:editor-extensions")),
  "check:js": noCacheTask(runTasks("check:js:packages", "check:js:direct-packages")),
  "check:js:packages": task(
    runInPackages("check", checkedPackagesViaVpRun, { concurrencyLimit: 1 }),
    {
      input: cacheInputs.jsChecks,
    },
  ),
  "check:js:direct-packages": noCacheTask(directPackageCheckCommand),
  "check:repo": noCacheTask(`${localVp} check`),
  // The oxlint example intentionally exits non-zero for its default lint script,
  // so CI checks every package except that runnable failure-case fixture.
  "check:ci": noCacheTask(
    `${runTask("check:repo")} && ${ciPackageCheckCommand} && ${directPackageCheckCommand}`,
  ),
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
