import * as fs from "fs";

/** Run npm bin scripts directly: Windows .cmd and pnpm shims need no shell. */
export function serverInvocation(serverPath: string, env: NodeJS.ProcessEnv = process.env) {
  const descriptor = fs.openSync(serverPath, "r");
  const prefix = Buffer.alloc(160);
  let length: number;
  try {
    length = fs.readSync(descriptor, prefix, 0, prefix.length, 0);
  } finally {
    fs.closeSync(descriptor);
  }
  const nodeScript =
    /\.[cm]?js$/i.test(serverPath) ||
    /^#![^\r\n]*\bnode(?:\s|$)/.test(prefix.subarray(0, length).toString("utf8"));
  return nodeScript
    ? {
        command: process.execPath,
        args: [serverPath],
        env: { ...env, ELECTRON_RUN_AS_NODE: "1" },
      }
    : { command: serverPath, args: [] as string[], env };
}
