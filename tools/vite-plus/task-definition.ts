import type { TaskConfig, TaskInput } from "./task-types.ts";
import { withRustTaskEnvironment } from "./task-shell.ts";

/**
 * Creates a cacheable Vite+ task while keeping Rust-specific environment
 * handling transparent to the task catalog.
 */
export const task = (
  command: string,
  options: {
    input?: TaskInput;
  } = {},
): TaskConfig => ({
  command: withRustTaskEnvironment(command),
  ...options,
});

/**
 * Creates an uncached task for commands whose effects are too broad or too
 * stateful to be represented by a stable input list.
 */
export const noCacheTask = (command: string): TaskConfig => ({
  cache: false as const,
  command: withRustTaskEnvironment(command),
});
