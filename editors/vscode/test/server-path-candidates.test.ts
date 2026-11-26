import assert from "node:assert/strict";
import fs from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import path from "node:path";
import test from "node:test";
import vm from "node:vm";

import { parseVizeVersion, serverMatchesExtension } from "../src/extension-core.ts";
import { trustedServerPath, workspaceDevServerPaths } from "../src/server-trust.ts";

const body = stripTypeScriptTypes(
  fs.readFileSync(new URL("../src/server-discovery.ts", import.meta.url), "utf8"),
)
  .replace(/^import[\s\S]*?;\n/gm, "")
  .replace(/^export /gm, "");

interface Discovery {
  findServerPath(context: object, config: object, generation: number): Promise<string | undefined>;
}

/**
 * Runs the extension's server discovery on a Linux host where every candidate
 * exists, and returns the commands it runs to inspect them, in order.
 */
async function inspected(pathEnv: string): Promise<string[]> {
  const executed: string[] = [];
  const context = vm.createContext({
    path: path.posix,
    process: { platform: "linux", arch: "x64", env: { HOME: "/home/user", PATH: pathEnv } },
    promisify: (run: unknown) => run,
    execFile: async (command: string, args: string[]) => {
      executed.push([command, ...args].join(" "));
      return { stdout: "vize 1.2.3\n" };
    },
    workspace: { workspaceFolders: [], isTrusted: true },
    parseVizeVersion,
    serverMatchesExtension,
    trustedServerPath,
    workspaceDevServerPaths,
    existingPaths: async () => () => true,
    serverInvocation: (serverPath: string) => ({ command: serverPath, args: [], env: {} }),
  });
  new vm.Script(body).runInContext(context);
  const create = context.createServerDiscovery as (...dependencies: unknown[]) => Discovery;
  const discovery = create(
    { configurationSyncGeneration: 1, outputChannel: { appendLine() {} } },
    () => {},
    async () => {},
  );
  const selected = await discovery.findServerPath(
    { extensionPath: "/extension", extension: { packageJSON: { version: "1.2.3" } } },
    { get: () => false, inspect: () => undefined },
    1,
  );
  assert.equal(selected, "/extension/dist/vize");
  return executed;
}

void test("only absolute PATH entries are searched for the server", async () => {
  const withoutPath = await inspected("");
  const absolute = await inspected("/opt/tools/bin");
  assert.deepEqual(
    absolute.filter((command) => !withoutPath.includes(command)),
    ["/opt/tools/bin/vize --version"],
  );
  // An empty or relative entry names a directory under the extension host's
  // working directory, which can be the opened repository.
  assert.deepEqual(
    await inspected(":.:./node_modules/.bin:node_modules/.bin:/opt/tools/bin:bin:"),
    absolute,
  );
});
