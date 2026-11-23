import { defineTasks, nodeCommand, noCacheTask } from "../task-helpers.ts";

export const inTestbox = (taskName: "build" | "test" | "lint") =>
  nodeCommand("ci/testbox", "run", taskName);

export const testboxTasks = defineTasks({
  "testbox:warmup": noCacheTask(nodeCommand("ci/testbox", "warmup")),
  "testbox:stop": noCacheTask(nodeCommand("ci/testbox", "stop")),
});
