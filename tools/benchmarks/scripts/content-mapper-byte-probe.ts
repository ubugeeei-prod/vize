/**
 * Transparent content-mapper proxy: `node content-mapper-byte-probe.ts <mapper dir> -- <exec...>`.
 *
 * The host starts it in place of a mapper; it starts the real mapper in the
 * mapper's package directory, forwards every byte both ways, and appends one
 * record per transform response to $VIZE_MAPPER_PROBE_RECORD, so the bytes
 * counted are exactly the authored SFC, its template content and the virtual
 * TS the host received. With
 * $VIZE_MAPPER_PROBE_DUMP set, each response's text is also written there at
 * its path relative to $VIZE_MAPPER_PROBE_ROOT, with a `.tsx` suffix, and the
 * starts of its spans and directives beside it in `<file>.tsx.shape.json`.
 */

import { spawn } from "node:child_process";
import { appendFileSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative } from "node:path";
import { parse } from "@vue/compiler-sfc";

import { frameReader, probeRecord } from "./typecheck-corpus-pair-parse.ts";
import { shapeSidecar } from "./typecheck-reply-shape.ts";

const [mapperDir, separator, command, ...args] = process.argv.slice(2);
const recordPath = process.env.VIZE_MAPPER_PROBE_RECORD;
const dumpDir = process.env.VIZE_MAPPER_PROBE_DUMP;
const dumpRoot = process.env.VIZE_MAPPER_PROBE_ROOT ?? "/";

function dump(fileName: string, response: Record<string, unknown>): void {
  const text = (response.result as { text?: unknown } | null | undefined)?.text;
  const path = relative(dumpRoot, fileName);
  if (!dumpDir || typeof text !== "string" || path.startsWith("..") || isAbsolute(path)) return;
  const target = join(dumpDir, `${path}.tsx`);
  mkdirSync(dirname(target), { recursive: true });
  writeFileSync(target, text);
  const result = response.result as Record<string, unknown>;
  writeFileSync(`${target}.shape.json`, JSON.stringify(shapeSidecar(result)));
}
if (!mapperDir || separator !== "--" || !command || !recordPath) {
  process.stderr.write(
    "usage: VIZE_MAPPER_PROBE_RECORD=<file> node content-mapper-byte-probe.ts <mapper dir> -- <exec...>\n",
  );
  process.exit(2);
}

const mapper = spawn(command, args, { cwd: mapperDir, stdio: ["pipe", "pipe", "pipe"] });
const fromHost = frameReader();
const fromMapper = frameReader();
type TransformInput = { fileName: string; sourceBytes: number; templateBytes: number };
const transforms = new Map<string, TransformInput>();

function transformInput(fileName: string, source: string): TransformInput {
  const parsed = parse(source, { filename: fileName, sourceMap: false });
  if (parsed.errors.length > 0) {
    throw new Error(`cannot parse authored SFC ${fileName}: ${parsed.errors.join("; ")}`);
  }
  return {
    fileName,
    sourceBytes: Buffer.byteLength(source, "utf8"),
    templateBytes: Buffer.byteLength(parsed.descriptor.template?.content ?? "", "utf8"),
  };
}

process.stdin.on("data", (chunk: Buffer) => {
  for (const message of fromHost(chunk)) {
    const params = message.params as { fileName?: unknown; content?: unknown } | undefined;
    if (message.method === "transform" && typeof params?.fileName === "string") {
      if (typeof params.content !== "string") {
        throw new Error(`transform request has no source text: ${params.fileName}`);
      }
      transforms.set(JSON.stringify(message.id), transformInput(params.fileName, params.content));
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
    const transform = transforms.get(key);
    if (transform == null) continue;
    transforms.delete(key);
    appendFileSync(
      recordPath,
      `${JSON.stringify(probeRecord(process.pid, transform.fileName, transform, message))}\n`,
    );
    dump(transform.fileName, message);
  }
  process.stdout.write(chunk);
});
mapper.stderr.pipe(process.stderr);
for (const signal of ["SIGINT", "SIGTERM"] as const) {
  process.on(signal, () => mapper.kill(signal));
}
mapper.on("close", (code, signal) => process.exit(code ?? (signal ? 1 : 0)));
