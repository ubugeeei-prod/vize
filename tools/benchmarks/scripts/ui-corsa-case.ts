#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { promises as fsp } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

type Row = Record<string, unknown>;
const output = process.env.VIZE_UI_OBSERVATION;
const workspace = process.env.GITHUB_WORKSPACE;
if (!output || !workspace) throw Error("explicit observation/workspace paths required");
const outputDirectory: string = output,
  sourceRoot: string = workspace;
const self = fileURLToPath(import.meta.url);
const proxy = path.join(path.dirname(self), "rpc-proxy.js");
const sourceRevision = "f234b6bff00cf0d44bcec8e8e45eaa98d850b220";
const filename = path.join(
  sourceRoot,
  "npm/ui/src/families/menus/dropdown-menu/dropdown-menu-trigger.vue",
);
const realGo = path.join(sourceRoot, "node_modules/@typescript/typescript-linux-x64/lib/tsc");
const rawFile = path.join(outputDirectory, "native-result.json");
const rpcFile = path.join(outputDirectory, "rpc.jsonl");
const stamp = () => process.hrtime.bigint().toString();
const sha = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
const json = (file: string, row: unknown) =>
  fs.writeFileSync(file, JSON.stringify(row, null, 2) + "\n");
async function identity(file: string) {
  const digest = createHash("sha256");
  for await (const chunk of fs.createReadStream(file)) digest.update(chunk);
  return {
    file,
    realpath: await fsp.realpath(file),
    bytes: (await fsp.stat(file)).size,
    sha256: digest.digest("hex"),
  };
}
function readRows(file: string): Row[] {
  if (!fs.existsSync(file)) return [];
  return fs
    .readFileSync(file, "utf8")
    .split("\n")
    .filter(Boolean)
    .flatMap((line) => {
      try {
        return [JSON.parse(line) as Row];
      } catch {
        return [];
      }
    });
}
async function worker() {
  const eventPath = path.join(outputDirectory, "worker.jsonl");
  const event = (row: Row) =>
    fs.appendFileSync(eventPath, JSON.stringify({ ns: stamp(), pid: process.pid, ...row }) + "\n");
  event({ event: "import-start" });
  const native = (await import(
    pathToFileURL(path.join(sourceRoot, "npm/native/index.js")).href
  )) as {
    lintPatinaSfc(source: string, options: Row): unknown;
  };
  const source = fs.readFileSync(filename, "utf8");
  event({
    event: "native-call-start",
    filename,
    sourceBytes: Buffer.byteLength(source),
    sourceSha256: sha(source),
    options: {
      filename,
      preset: "opinionated",
      typeAware: true,
      helpLevel: "short",
      corsaPath: proxy,
    },
  });
  let result: unknown;
  try {
    result = native.lintPatinaSfc(source, {
      filename,
      preset: "opinionated",
      typeAware: true,
      helpLevel: "short",
      corsaPath: proxy,
    });
    json(rawFile, result);
    const diagnostics =
      result && typeof result === "object" && "diagnostics" in result
        ? (result as { diagnostics?: unknown }).diagnostics
        : undefined;
    event({
      event: "native-call-end",
      diagnosticsCount: Array.isArray(diagnostics) ? diagnostics.length : null,
    });
    process.exitCode = Array.isArray(diagnostics) && diagnostics.length === 0 ? 0 : 1;
  } catch (error) {
    event({
      event: "native-call-threw",
      error: error instanceof Error ? error.stack : String(error),
    });
    process.exitCode = 2;
  } finally {
    const report = process.report.getReport() as { sharedObjects?: string[] };
    const loaded = (report.sharedObjects ?? []).filter((file) => file.endsWith(".node"));
    const images = await Promise.all(loaded.map(identity));
    json(path.join(outputDirectory, "loaded-images.json"), images);
    event({ event: "loaded-images", count: images.length });
  }
}
async function echo() {
  const chunks: Buffer[] = [];
  for await (const chunk of process.stdin) chunks.push(Buffer.from(chunk));
  const input = Buffer.concat(chunks);
  for (let i = 0; i < input.length;) {
    const end = Math.min(input.length, i + 1 + (i % 7919));
    if (!process.stdout.write(input.subarray(i, end)))
      await new Promise<void>((resolve) => process.stdout.once("drain", resolve));
    i = end;
  }
}
async function calibrate(observerPath: string) {
  const wrapper = path.join(outputDirectory, "echo-child");
  fs.writeFileSync(
    wrapper,
    "#!/bin/sh\nexec " +
      JSON.stringify(process.execPath) +
      " " +
      JSON.stringify(observerPath) +
      " --echo\n",
    { mode: 0o755 },
  );
  const bodies = [
    '{"jsonrpc":"2.0","id":9007199254740993,"method":"calibrate","params":{"text":"日本🙂\\r\\n"}}',
    '{"jsonrpc":"2.0","id":"escaped\\\\id","result":{"ok":true}}',
    JSON.stringify({
      jsonrpc: "2.0",
      id: "large",
      method: "large",
      params: { payload: "字".repeat(100_000) },
    }),
  ];
  const frames = bodies.map((body) =>
    Buffer.concat([
      Buffer.from("Content-Length: " + Buffer.byteLength(body) + "\r\n\r\n"),
      Buffer.from(body),
    ]),
  );
  const input = Buffer.concat(frames);
  const log = path.join(outputDirectory, "calibration-rpc.jsonl");
  const child = spawn(process.execPath, [proxy, "--api", "--async", "--cwd", outputDirectory], {
    env: { ...process.env, VIZE_UI_RPC_REAL_EXE: wrapper, VIZE_UI_RPC_LOG: log },
    stdio: ["pipe", "pipe", "pipe"],
  });
  const chunks: Buffer[] = [],
    errors: Buffer[] = [];
  child.stdout.on("data", (chunk: Buffer) => chunks.push(chunk));
  child.stderr.on("data", (chunk: Buffer) => errors.push(chunk));
  const completion = new Promise<void>((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (code, signal) =>
      code === 0 && signal === null ? resolve() : reject(Error("calibration child failed")),
    );
  });
  for (let start = 0; start < input.length;) {
    const end = Math.min(input.length, start + 1 + (start % 4093));
    if (!child.stdin.write(input.subarray(start, end)))
      await new Promise<void>((resolve) => child.stdin.once("drain", resolve));
    start = end;
  }
  child.stdin.end();
  await completion;
  assert.deepEqual(Buffer.concat(chunks), input);
  assert.equal(Buffer.concat(errors).length, 0);
  const rows = readRows(log);
  for (const direction of ["to-go", "from-go"]) {
    const selected = rows.filter((r) => r.event === "frame" && r.direction === direction);
    assert.equal(selected.length, 3);
    for (const [i, row] of selected.entries()) {
      assert.equal(row.sha256, sha(bodies[i]!));
      assert.equal(row.payloadBytes, Buffer.byteLength(bodies[i]!));
    }
    assert.equal(selected[0]!.id, "n:9007199254740993");
    assert.equal(selected[1]!.id, "s:escaped\\id");
    assert.equal(selected[2]!.decoded, false);
  }
  return {
    qualified: true,
    bytesEachDirection: input.length,
    sha256: sha(input),
    framesEachDirection: 3,
    preservedLargeNumericId: "9007199254740993",
    oversizedBodyMetadataDecoded: false,
  };
}

export {
  worker,
  echo,
  calibrate,
  identity,
  readRows,
  stamp,
  sha,
  sleep,
  outputDirectory,
  sourceRoot,
  sourceRevision,
  filename,
  realGo,
  rawFile,
  rpcFile,
};
export type { Row };
