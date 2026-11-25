import { ciCommand, defineTasks, noCacheTask } from "../task-helpers.ts";

export const inTestbox = (taskName: "build" | "test" | "lint") =>
  ciCommand("testbox", "run", taskName);

export const testboxTasks = defineTasks({
  "testbox:warmup": noCacheTask(ciCommand("testbox", "warmup")),
  "testbox:stop": noCacheTask(ciCommand("testbox", "stop")),
});
