import assert from "node:assert/strict";
import { taskCatalog } from "../../config/vite-plus/task-groups.ts";
import { workspaceTaskSteps } from "../../config/vite-plus/task-plan.ts";
import { workspaceTaskEnvironment } from "../../config/vite-plus/task-shell.ts";
import { runPackageTask } from "../../support/tasks/package-task.ts";

const [name, ...forwarded] = process.argv.slice(2);
assert.ok(name && Object.hasOwn(taskCatalog, name), "unknown workspace task");
process.exitCode = runPackageTask(
  { [name]: workspaceTaskSteps(taskCatalog[name as keyof typeof taskCatalog]) },
  [name, ...forwarded],
  workspaceTaskEnvironment(process.platform, process.env),
);
