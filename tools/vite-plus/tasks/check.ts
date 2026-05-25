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
  shellCommand,
  task,
} from "../task-helpers.ts";

const ciPackageCheckCommand = runInPackages("check", ciCheckedPackages, {
  concurrencyLimit: 1,
});
const directPackageCheckCommand = runPackageScriptDirectly("check", directCheckPackages);
const strictRepoCheckCommand = `node tools/vite-plus/check-warning-budget.mjs -- ${localVp} check`;
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
const localActrunCommand = [runTask("actrun:check"), runTask("actrun:benchmark")].join(" && ");
const actrunWorkspaceFlag = (workflow: string) =>
  `--workspace _build/actrun/workspace/${workflow}-$$`;
const actrunWorkflowRunCommand = (workflow: string, ...args: string[]) =>
  shellCommand(
    `actrun workflow run .github/workflows/${workflow}.yml ${actrunWorkspaceFlag(workflow)} ${args.join(" ")}`,
  );
const actrunWorkflowRunForwardingCommand = (workflow: string) =>
  `actrun workflow run .github/workflows/${workflow}.yml ${actrunWorkspaceFlag(workflow)} "$@"`;

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
  // v1 alpha release branches keep a zero-warning budget for repo-wide JS/TS checks.
  "check:repo": noCacheTask(strictRepoCheckCommand),
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
  actrun: noCacheTask(localActrunCommand),
  "actrun:check": noCacheTask(runTasks("actrun:lint", "actrun:dry-run", "actrun:check-js")),
  "actrun:lint": noCacheTask("actrun lint .github/workflows/check.yml"),
  "actrun:dry-run": noCacheTask(actrunWorkflowRunCommand("check", "--dry-run")),
  "actrun:job": noCacheTask(actrunWorkflowRunForwardingCommand("check"), {
    forwardArguments: true,
  }),
  "actrun:check-js": noCacheTask(runTask("actrun:job") + " --job check-js"),
  "actrun:benchmark": noCacheTask(runTasks("actrun:benchmark:lint", "actrun:benchmark:dry-run")),
  "actrun:benchmark:lint": noCacheTask("actrun lint .github/workflows/benchmark.yml"),
  "actrun:benchmark:dry-run": noCacheTask(actrunWorkflowRunCommand("benchmark", "--dry-run")),
  "actrun:benchmark:job": noCacheTask(actrunWorkflowRunForwardingCommand("benchmark"), {
    forwardArguments: true,
  }),
  ci: noCacheTask(runTasks("fmt:all", "clippy", "test", "check:ci")),
});
