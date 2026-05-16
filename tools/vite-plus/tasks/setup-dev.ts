import { defineTasks, devApp, noCacheTask, runInPackages, runTask } from "../task-helpers.ts";

/**
 * Root setup and development tasks.
 *
 * These commands are intentionally thin wrappers around package scripts or
 * MoonBit automation. Keeping setup and foreground dev targets together makes
 * it obvious which commands are interactive and which command merely prepares
 * the workspace.
 */
export const setupAndDevTasks = defineTasks({
  setup: noCacheTask("vp install"),
  dev: noCacheTask(runTask("dev:app")),
  "dev:app": noCacheTask(devApp()),
  "dev:playground": noCacheTask(devApp("playground")),
  "dev:misskey": noCacheTask(devApp("misskey")),
  "dev:npmx": noCacheTask(devApp("npmx")),
  "dev:elk": noCacheTask(devApp("elk")),
  "dev:vuefes": noCacheTask(devApp("vuefes")),
  example: noCacheTask(runInPackages("dev", ["./npm/vite-plugin-vize/example"])),
});
