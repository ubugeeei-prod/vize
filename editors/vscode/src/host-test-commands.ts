import { commands, type Disposable } from "vscode";
import { waitForAutoInsertIdle } from "./auto-insert-test-state.js";

import {
  bindHostTestCommands,
  HOST_TEST_COMMAND_ENVIRONMENT_FLAG,
  type HostTestLanguageClient,
  type HostTestServerInfo,
} from "./host-test-core.js";

/**
 * Binds the environment-gated host smoke commands to the VS Code command
 * registry. The gate itself lives in `host-test-core` so the tooling tests can
 * exercise it without a VS Code host.
 */
export function registerHostTestCommands(
  getClient: () => HostTestLanguageClient | undefined,
  environment: Partial<Record<string, string>> = process.env,
  getServerInfo?: () => HostTestServerInfo | undefined,
): Disposable[] {
  const registrations = bindHostTestCommands({
    environment,
    getClient,
    getServerInfo,
    register: (command, handler) => commands.registerCommand(command, handler),
  });
  if (environment[HOST_TEST_COMMAND_ENVIRONMENT_FLAG] === "1") {
    registrations.push(
      commands.registerCommand("vize.test.waitForAutoInsertIdle", waitForAutoInsertIdle),
    );
  }
  return registrations;
}
