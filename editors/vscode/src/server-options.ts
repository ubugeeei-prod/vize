import { type Executable, type ServerOptions, TransportKind } from "vscode-languageclient/node.js";
import { serverProcessEnv } from "./corsa-runtime.js";
import { serverInvocation } from "./server-command.js";

export function createServerOptions(
  serverPath: string,
  env: NodeJS.ProcessEnv = process.env,
): ServerOptions {
  const invocation = serverInvocation(serverPath, env);
  const run: Executable = {
    command: invocation.command,
    args: [...invocation.args, "lsp"],
    transport: TransportKind.stdio,
    options: { env: serverProcessEnv(invocation.env, false) },
  };

  const debug: Executable = {
    command: invocation.command,
    args: [...invocation.args, "lsp", "--debug"],
    transport: TransportKind.stdio,
    options: { env: serverProcessEnv(invocation.env, true) },
  };

  return { run, debug };
}
