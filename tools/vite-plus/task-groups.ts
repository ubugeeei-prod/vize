import { defineTasks } from "./task-helpers.ts";
import { buildTasks } from "./tasks/build.ts";
import { checkTasks } from "./tasks/check.ts";
import { generationAndCliTasks } from "./tasks/generation-cli.ts";
import { releaseTasks } from "./tasks/release.ts";
import { setupAndDevTasks } from "./tasks/setup-dev.ts";
import { testAndBenchmarkTasks } from "./tasks/test-benchmark.ts";

/**
 * Fully assembled root Vite+ task catalog.
 *
 * Each imported task group owns one workflow family, which keeps the root
 * `vite.config.ts` type-safe and compact while preserving detailed comments
 * close to the shell commands they explain. This file should stay a thin
 * composition layer: add new task behavior in a focused `tasks/*` module and
 * expose it here only by spreading that module's catalog.
 */
export const taskCatalog = defineTasks({
  ...setupAndDevTasks,
  ...buildTasks,
  ...generationAndCliTasks,
  ...testAndBenchmarkTasks,
  ...checkTasks,
  ...releaseTasks,
});
