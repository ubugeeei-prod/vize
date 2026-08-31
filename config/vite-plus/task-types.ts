import type { UserConfig } from "vite-plus";

export type TaskMap = NonNullable<NonNullable<UserConfig["run"]>["tasks"]>;
export type TaskConfig = TaskMap[string];
export type CacheableTaskConfig = Extract<TaskConfig, { cache?: true }>;
export type TaskInput = NonNullable<CacheableTaskConfig["input"]>;
export type PackagePath = `./${string}`;

/**
 * Preserves the exact task object shape while letting TypeScript validate every
 * task against Vite+'s public configuration type.
 *
 * Keeping this helper small but explicit gives each task group precise literal
 * keys without falling back to a broad `Record<string, unknown>` style. That is
 * important for the root config because the task catalog is assembled from many
 * modules and should fail at compile time when Vite+ changes its task schema.
 */
export const defineTasks = <const T extends TaskMap>(tasks: T): T => tasks;
