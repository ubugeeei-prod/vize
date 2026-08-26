import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, toValue } from "vue";

import type {
  CommandDefinition,
  CommandDispatch,
  CommandDispatchSource,
  CommandExecution,
  CommandInfo,
  CommandRouter,
  CommandRouterOptions,
} from "./command-types.ts";

const disposedDiagnostic = "VIZE_UI_COMMAND_DISPOSED";
const setupDiagnostic = "VIZE_UI_COMMAND_SETUP";
const optionDiagnostic = "VIZE_UI_COMMAND_OPTION";
const conflictDiagnostic = "VIZE_UI_COMMAND_CONFLICT";
const dispatchSources = new Set<CommandDispatchSource>([
  "imperative",
  "menu",
  "palette",
  "shortcut",
]);

function readBoolean(source: CommandDefinition["when"], name: string, fallback: boolean): boolean {
  const value = toValue(source);
  if (value === undefined) return fallback;
  if (typeof value !== "boolean") {
    throw new TypeError(`${optionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return value;
}

function validateDefinition<Id extends string>(command: CommandDefinition<Id>): void {
  if (typeof command.id !== "string" || command.id === "") {
    throw new TypeError(`${optionDiagnostic}: id must be a non-empty string`);
  }
  if (typeof command.run !== "function") {
    throw new TypeError(`${optionDiagnostic}: run must be a function`);
  }
  for (const name of ["title", "description", "group"] as const) {
    if (command[name] !== undefined && typeof command[name] !== "string") {
      throw new TypeError(`${optionDiagnostic}: ${name} must be a string`);
    }
  }
  if (
    command.keywords !== undefined &&
    (!Array.isArray(command.keywords) ||
      command.keywords.some((keyword) => typeof keyword !== "string"))
  ) {
    throw new TypeError(`${optionDiagnostic}: keywords must be an array of strings`);
  }
}

/**
 * Create a typed command router with enablement and help metadata.
 *
 * The router owns no DOM and is safe to create during server rendering.
 * Registering an identifier twice throws a conflict diagnostic so palettes,
 * menus, and shortcut layers cannot silently shadow one another. Call
 * {@link CommandRouter.dispose} when using this factory outside a Vue effect
 * scope.
 */
export function createCommandRouter<Id extends string = string>(
  options: CommandRouterOptions<Id> = {},
): CommandRouter<Id> {
  if (options.onDidExecute !== undefined && typeof options.onDidExecute !== "function") {
    throw new TypeError(`${optionDiagnostic}: onDidExecute must be a function`);
  }
  readBoolean(options.isDisabled, "isDisabled", false);

  const registrations = new Map<Id, CommandDefinition<Id>>();
  const commands = shallowRef<readonly CommandInfo<Id>[]>(Object.freeze([]));
  let disposed = false;

  const assertActive = () => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the router has been disposed`);
  };
  const readEnabled = (command: CommandDefinition<Id>): boolean =>
    !readBoolean(options.isDisabled, "isDisabled", false) &&
    readBoolean(command.when, "when", true);
  const publishCommands = () => {
    commands.value = Object.freeze(
      [...registrations.values()].map((command) =>
        Object.freeze({
          id: command.id,
          title: command.title ?? null,
          description: command.description ?? null,
          keywords: Object.freeze([...(command.keywords ?? [])]),
          group: command.group ?? null,
          isEnabled: () =>
            !disposed && registrations.get(command.id) === command && readEnabled(command),
        }),
      ),
    );
  };

  const execute = (
    id: Id,
    payload?: unknown,
    executeOptions: { readonly source?: CommandDispatchSource } = {},
  ): CommandDispatch<Id> => {
    assertActive();
    const source = executeOptions.source ?? "imperative";
    if (!dispatchSources.has(source)) {
      throw new TypeError(
        `${optionDiagnostic}: source must be imperative, menu, palette, or shortcut`,
      );
    }
    const command = registrations.get(id);
    const finish = (status: CommandDispatch<Id>["status"], value: unknown) => {
      const dispatch: CommandDispatch<Id> = Object.freeze({ id, status, value, source });
      if (command !== undefined) options.onDidExecute?.(dispatch);
      return dispatch;
    };
    if (command === undefined) return finish("not-found", undefined);
    if (!readEnabled(command)) return finish("disabled", undefined);
    const execution: CommandExecution<Id> = Object.freeze({ id, payload, source });
    return finish("executed", command.run(execution));
  };

  return Object.freeze({
    commands: shallowReadonly(commands),
    register(command: CommandDefinition<Id>) {
      assertActive();
      validateDefinition(command);
      const existing = registrations.get(command.id);
      if (existing !== undefined) {
        throw new TypeError(
          `${conflictDiagnostic}: "${command.id}" is already registered` +
            (existing.title ? ` as "${existing.title}"` : ""),
        );
      }
      registrations.set(command.id, command);
      publishCommands();
      return () => {
        if (registrations.get(command.id) !== command) return;
        registrations.delete(command.id);
        if (!disposed) publishCommands();
      };
    },
    has(id: Id) {
      assertActive();
      return registrations.has(id);
    },
    isEnabled(id: Id) {
      assertActive();
      const command = registrations.get(id);
      return command !== undefined && readEnabled(command);
    },
    execute,
    dispose: () => {
      if (disposed) return;
      disposed = true;
      registrations.clear();
      commands.value = Object.freeze([]);
    },
  });
}

/** Create a command router disposed with the current Vue effect scope. */
export function useCommandRouter<Id extends string = string>(
  options: CommandRouterOptions<Id> = {},
): CommandRouter<Id> {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const router = createCommandRouter(options);
  onScopeDispose(router.dispose);
  return router;
}

export type {
  CommandDefinition,
  CommandDispatch,
  CommandDispatchSource,
  CommandExecution,
  CommandInfo,
  CommandRouter,
  CommandRouterOptions,
} from "./command-types.ts";
