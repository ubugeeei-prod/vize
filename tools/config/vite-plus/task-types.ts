import { nameWorkspaceTask } from "./task-plan.ts";
import type { UserConfig } from "vite-plus";

export type TaskMap = NonNullable<NonNullable<UserConfig["run"]>["tasks"]>;
export type TaskConfig = TaskMap[string];
export type CacheableTaskConfig = Extract<TaskConfig, { cache?: true }>;
export type TaskInput = NonNullable<CacheableTaskConfig["input"]>;
export type PackagePath = `./${string}`;

export const defineTasks = <const T extends TaskMap>(tasks: T): T => {
  for (const [name, config] of Object.entries(tasks)) nameWorkspaceTask(name, config);
  return tasks;
};
