import { isProgramCommand, invocationWords } from "../../support/ci/automation-commands.ts";
import type { PackageCommand } from "../../support/tasks/package-task.ts";
import type { TaskConfig } from "./task-types.ts";

export type TaskCommand = string | PackageCommand | readonly PackageCommand[];
const plans = new WeakMap<object, readonly PackageCommand[]>();

/** Keep the executable recipe available to the named Vite+ driver and its tests. */
export function steps(...commands: readonly TaskCommand[]): PackageCommand[] {
  return commands.flatMap((command) => {
    if (typeof command !== "string") {
      const commands = "program" in command ? [command] : command;
      for (const step of commands)
        if (!isProgramCommand([step.program, ...(step.arguments ?? [])]))
          throw new Error("Workspace tasks require a named program: " + step.program);
      return [...commands];
    }
    const words = invocationWords(command);
    if (!words || !isProgramCommand(words))
      throw new Error("Expected one workspace program: " + command);
    const [program, ...args] = words;
    return [{ program: program!, arguments: args }];
  });
}

export function attachPlan(config: object, commands: TaskCommand): void {
  plans.set(config, steps(commands));
}

export function workspaceTaskSteps(config: TaskConfig): readonly PackageCommand[] {
  const commands = typeof config === "object" && config !== null ? plans.get(config) : undefined;
  if (!commands) throw new Error("Unregistered workspace task");
  return commands;
}

export function nameWorkspaceTask(name: string, config: TaskConfig): void {
  if (typeof config !== "object" || config === null || !plans.has(config)) return;
  if ("command" in config && config.command === "")
    config.command = "node tools/commands/ci/workspace-task.ts " + name;
}
