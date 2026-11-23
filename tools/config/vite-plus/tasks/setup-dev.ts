import { defineTasks, devApp, noCacheTask, runInPackages, runTask } from "../task-helpers.ts";

export const setupAndDevTasks = defineTasks({
  setup: noCacheTask("vp install"),
  dev: noCacheTask(runTask("dev:app")),
  "dev:app": noCacheTask(devApp()),
  "dev:playground": noCacheTask(devApp("playground")),
  "dev:misskey": noCacheTask(devApp("misskey")),
  "dev:npmx": noCacheTask(devApp("npmx")),
  "dev:elk": noCacheTask(devApp("elk")),
  "dev:vuefes": noCacheTask(devApp("vuefes")),
  example: noCacheTask(runInPackages("dev", ["./npm/builder/vite/example"])),
});
