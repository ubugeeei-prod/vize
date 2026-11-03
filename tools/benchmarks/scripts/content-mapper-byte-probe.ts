/**
 * Transparent content-mapper proxy: `node content-mapper-byte-probe.ts <mapper dir> -- <exec...>`.
 *
 * The host starts it in place of a mapper; it starts the real mapper in the
 * mapper's package directory, forwards every byte both ways, and appends one
 * record per transform response to $VIZE_MAPPER_PROBE_RECORD, so the bytes
 * counted are exactly the virtual TS the host received.
 */

import { spawn } from "node:child_process";
import { appendFileSync } from "node:fs";

import { frameReader, probeRecord } from "./typecheck-corpus-pair-parse.ts";

const [mapperDir, separator, command, ...args] = process.argv.slice(2);
const recordPath = process.env.VIZE_MAPPER_PROBE_RECORD;
if (!mapperDir || separator !== "--" || !command || !recordPath) {
  process.stderr.write(
    "usage: VIZE_MAPPER_PROBE_RECORD=<file> node content-mapper-byte-probe.ts <mapper dir> -- <exec...>\n",
  );
  process.exit(2);
}

const mapper = spawn(command, args, { cwd: mapperDir, stdio: ["pipe", "pipe", "pipe"] });
const fromHost = frameReader();
const fromMapper = frameReader();
const transforms = new Map<string, string>();

process.stdin.on("data", (chunk: Buffer) => {
  for (const message of fromHost(chunk)) {
    const params = message.params as { fileName?: unknown } | undefined;
    if (message.method === "transform" && typeof params?.fileName === "string") {
      transforms.set(JSON.stringify(message.id), params.fileName);
    }
  }
  mapper.stdin.write(chunk);
});
process.stdin.on("end", () => mapper.stdin.end());
// A mapper that already exited reports through "close"; only that write failure is expected.
mapper.stdin.on("error", (error: NodeJS.ErrnoException) => {
  if (error.code !== "EPIPE") throw error;
});
mapper.stdout.on("data", (chunk: Buffer) => {
  for (const message of fromMapper(chunk)) {
    if (Object.hasOwn(message, "method")) continue;
    const key = JSON.stringify(message.id);
    const fileName = transforms.get(key);
    if (fileName == null) continue;
    transforms.delete(key);
    appendFileSync(recordPath, `${JSON.stringify(probeRecord(process.pid, fileName, message))}\n`);
  }
  process.stdout.write(chunk);
});
mapper.stderr.pipe(process.stderr);
for (const signal of ["SIGINT", "SIGTERM"] as const) {
  process.on(signal, () => mapper.kill(signal));
}
mapper.on("close", (code, signal) => process.exit(code ?? (signal ? 1 : 0)));
