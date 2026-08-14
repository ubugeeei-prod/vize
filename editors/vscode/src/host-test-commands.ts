import { commands, type Disposable } from "vscode";

import { createHostTestCommands, type HostTestLanguageClient } from "./extension-core";

/**
 * Binds the environment-gated host smoke commands to the VS Code command
 * registry. The gate itself lives in `extension-core` so the tooling tests can
 * exercise it without a VS Code host.
 */
export function registerHostTestCommands(
  getClient: () => HostTestLanguageClient | undefined,
  environment: Partial<Record<string, string>> = process.env,
): Disposable[] {
  return createHostTestCommands({ environment, getClient }).map(({ command, handler }) =>
    commands.registerCommand(command, handler),
  );
}
