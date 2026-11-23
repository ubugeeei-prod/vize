import { defineTasks } from "./task-helpers.ts";
import { buildTasks } from "./tasks/build.ts";
import { checkTasks } from "./tasks/check.ts";
import { generationAndCliTasks } from "./tasks/generation-cli.ts";
import { releaseTasks } from "./tasks/release.ts";
import { setupAndDevTasks } from "./tasks/setup-dev.ts";
import { testAndBenchmarkTasks } from "./tasks/test-benchmark.ts";
import { testboxTasks } from "./tasks/testbox.ts";

export const taskCatalog = defineTasks({
  ...setupAndDevTasks,
  ...buildTasks,
  ...generationAndCliTasks,
  ...testAndBenchmarkTasks,
  ...checkTasks,
  ...releaseTasks,
  ...testboxTasks,
});
