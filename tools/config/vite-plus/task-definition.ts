import { attachPlan, type TaskCommand } from "./task-plan.ts";
import { rustEnvironmentKeys } from "./task-shell.ts";
import type { TaskConfig, TaskInput } from "./task-types.ts";

const executorInputs = [
  "tools/commands/ci/workspace-task.ts",
  "tools/config/vite-plus/**",
  "tools/support/ci/automation-commands.ts",
  "tools/support/tasks/**",
  "tools/support/command-invocation.ts",
];

export function task(
  commands: TaskCommand,
  options: { env?: string[]; input?: TaskInput; cache?: false } = {},
): TaskConfig {
  const rustEnvironment = Boolean(process.env.VIZE_DARWIN_LIBICONV_LIB);
  const direct = typeof commands === "string" && !rustEnvironment;
  const { input, ...metadata } = options;
  const config = {
    command: direct ? commands : "",
    ...metadata,
    ...(input ? { input: direct ? input : [...input, ...executorInputs] } : {}),
    ...(rustEnvironment && options.cache !== false
      ? { env: [...(options.env ?? []), ...rustEnvironmentKeys] }
      : {}),
  };
  attachPlan(config, commands);
  return config;
}

export const noCacheTask = (commands: TaskCommand): TaskConfig => task(commands, { cache: false });
