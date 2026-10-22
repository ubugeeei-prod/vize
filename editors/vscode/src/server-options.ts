import { type Executable, type ServerOptions, TransportKind } from "vscode-languageclient/node.js";
import { serverProcessEnv } from "./corsa-runtime.js";

export function createServerOptions(
  serverPath: string,
  env: NodeJS.ProcessEnv = process.env,
): ServerOptions {
  const run: Executable = {
    command: serverPath,
    args: ["lsp"],
    transport: TransportKind.stdio,
    options: { env: serverProcessEnv(env, false) },
  };

  const debug: Executable = {
    command: serverPath,
    args: ["lsp", "--debug"],
    transport: TransportKind.stdio,
    options: { env: serverProcessEnv(env, true) },
  };

  return { run, debug };
}
