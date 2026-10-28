/** Raw native type calibration; no typed Corsa decoder discards intrinsic metadata. */
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { identify, save } from "./canon-instance-directives-io.ts";
import { isRecord } from "./canon-instance-directives-model.ts";
type JsonObject = Record<string, unknown>;
function object(value: unknown): JsonObject {
  assert.ok(isRecord(value));
  return value;
}
function scalar(value: unknown): string | number {
  assert.ok(typeof value === "string" || typeof value === "number");
  return value;
}
const names = [
  "errorValue",
  "anyValue",
  "unknownValue",
  "neverValue",
  "numberValue",
  "errorResult",
  "anyResult",
  "unknownResult",
  "neverResult",
  "numberResult",
];
export const calibrationSource = [
  "export {};",
  'type IsAny<T>=0 extends 1&T?true:false;type IsNever<T>=[T] extends [never]?true:false;type IsUnknown<T>=IsAny<T> extends true?false:unknown extends T?keyof T extends never?true:false:false;type VizeProbeSlotReport<T>=IsAny<T> extends true?"any":IsNever<T> extends true?"never":IsUnknown<T> extends true?"unknown":T extends number?"number":"other";',
  "// @ts-ignore deliberate calibration error",
  'type ErrorAny={}[ "missing" ];',
  "declare const errorValue:ErrorAny;declare const anyValue:any;declare const unknownValue:unknown;declare const neverValue:never;declare const numberValue:number;",
  'function report<T>(_value:T):VizeProbeSlotReport<T>{throw Error();}function expectNumber(_value:"number"):void{}',
  "const errorResult=report(errorValue), anyResult=report(anyValue), unknownResult=report(unknownValue),neverResult=report(neverValue),numberResult=report(numberValue);",
  "expectNumber(errorResult);expectNumber(anyResult);expectNumber(unknownResult);expectNumber(neverResult);expectNumber(numberResult);",
  "",
].join("\n");
export async function queryCalibration(
  binary: string,
  project: string,
  out: string,
  dialect: "7.0.2" | "7.1.0-beta",
): Promise<JsonObject> {
  fs.mkdirSync(out);
  const file = path.join(project, "Calibration.ts"),
    config = path.join(project, "tsconfig.json");
  fs.writeFileSync(file, calibrationSource);
  save(config, {
    compilerOptions: {
      strict: true,
      skipLibCheck: true,
      noEmit: true,
      types: [],
      target: "ESNext",
    },
    files: ["Calibration.ts"],
  });
  const image = identify(binary),
    argv = ["--api", "--async", "--cwd", project];
  const child = spawn(binary, argv, { cwd: project, env: process.env, stdio: "pipe" });
  const input: Buffer[] = [],
    output: Buffer[] = [],
    errors: Buffer[] = [];
  const requests: JsonObject[] = [],
    responses: JsonObject[] = [];
  const waiting = new Map<
    number,
    { resolve: (value: unknown) => void; reject: (error: Error) => void }
  >();
  let buffered = Buffer.alloc(0),
    failure: string | null = null,
    sequence = 0;
  const rejectAll = (error: Error) => {
    failure ??= error.message;
    for (const pending of waiting.values()) pending.reject(error);
    waiting.clear();
  };
  const closed = new Promise<{ status: number | null; signal: string | null }>((resolve) => {
    child.on("error", rejectAll);
    child.on("close", (status, signal) => {
      if (waiting.size) rejectAll(new Error("native API closed before response"));
      resolve({ status, signal });
    });
  });
  child.stderr.on("data", (chunk: Buffer) => errors.push(Buffer.from(chunk)));
  child.stdout.on("data", (chunk: Buffer) => {
    output.push(Buffer.from(chunk));
    buffered = Buffer.concat([buffered, chunk]);
    try {
      while (buffered.length) {
        const end = buffered.indexOf("\r\n\r\n");
        if (end < 0) return;
        const match = /^Content-Length: (\d+)$/iu.exec(buffered.subarray(0, end).toString("ascii"));
        assert.ok(match && match[1]);
        const length = +match[1],
          start = end + 4;
        assert.ok(Number.isSafeInteger(length) && length <= 16 * 1024 * 1024);
        if (buffered.length < start + length) return;
        const value: unknown = JSON.parse(
          new TextDecoder("utf-8", { fatal: true }).decode(
            buffered.subarray(start, start + length),
          ),
        );
        const response = object(value);
        assert.equal(response.jsonrpc, "2.0");
        assert.ok(typeof response.id === "number");
        const pending = waiting.get(response.id);
        assert.ok(pending, "unexpected or duplicate response ID");
        assert.ok(!Object.hasOwn(response, "error"), JSON.stringify(response));
        assert.ok(Object.hasOwn(response, "result"));
        waiting.delete(response.id);
        responses.push(response);
        buffered = buffered.subarray(start + length);
        pending.resolve(response.result);
      }
    } catch (error) {
      rejectAll(error instanceof Error ? error : new Error(String(error)));
    }
  });
  async function request(method: string, params: unknown): Promise<unknown> {
    assert.equal(failure, null);
    const id = ++sequence,
      request = { jsonrpc: "2.0", id, method, params };
    const body = Buffer.from(JSON.stringify(request));
    const frame = Buffer.concat([Buffer.from("Content-Length: " + body.length + "\r\n\r\n"), body]);
    requests.push(request);
    input.push(frame);
    return await new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        waiting.delete(id);
        reject(new Error("native API request timed out: " + method));
      }, 30_000);
      waiting.set(id, {
        resolve: (value) => {
          clearTimeout(timer);
          resolve(value);
        },
        reject: (error) => {
          clearTimeout(timer);
          reject(error);
        },
      });
      child.stdin.write(frame, (error) => {
        if (error) rejectAll(error);
      });
    });
  }
  const rows: JsonObject[] = [];
  let semantic: unknown = null,
    initialized: unknown = null,
    snapshotResponse: unknown = null;
  try {
    initialized = await request("initialize", null);
    snapshotResponse = await request(dialect === "7.0.2" ? "updateSnapshot" : "createSnapshot", {
      openProjects: [config],
      ...(dialect === "7.1.0-beta" ? { ensurePrograms: true } : {}),
    });
    const snapshot = scalar(object(snapshotResponse).snapshot);
    const projectResponse = object(await request("getDefaultProjectForFile", { snapshot, file }));
    const projectId = scalar(projectResponse.id);
    for (const name of names) {
      const position = calibrationSource.indexOf(name);
      assert.ok(position >= 0 && !/[A-Za-z0-9_]/u.test(calibrationSource[position - 1] ?? ""));
      const symbol = object(
        await request("getSymbolAtPosition", { snapshot, project: projectId, file, position }),
      );
      const handle = dialect === "7.0.2" ? scalar(symbol.id) : object(symbol.reference);
      const type = object(
        await request("getTypeOfSymbol", { snapshot, project: projectId, symbol: handle }),
      );
      rows.push({ name, position, symbol, type });
    }
    semantic = await request("getSemanticDiagnostics", {
      snapshot,
      project: projectId,
      files: [file],
    });
    await request("release", { snapshot });
  } catch (error) {
    failure ??= error instanceof Error ? error.message : String(error);
  }
  child.stdin.end();
  const timeout = setTimeout(() => child.kill("SIGKILL"), 5_000);
  const completion = await closed;
  clearTimeout(timeout);
  const after = identify(binary);
  for (const [channel, chunks] of [
    ["stdin", input],
    ["stdout", output],
    ["stderr", errors],
  ] as const)
    fs.writeFileSync(path.join(out, channel + ".raw"), Buffer.concat(chunks));
  const evidence = {
    schema: "vize.slotErrorAnyCalibration.v1",
    dialect,
    image,
    after,
    argv,
    cwd: project,
    initialized,
    snapshotResponse,
    source: calibrationSource,
    requests,
    responses,
    rows,
    semantic,
    ...completion,
    failure,
    trailingBytes: buffered.length,
    timingUsable: false,
    scope: "one raw native API session, one tiny project; no host rewrites or origin qualification",
  };
  save(path.join(out, "receipt.json"), evidence);
  assert.equal(failure, null);
  assert.equal(completion.status, 0);
  assert.equal(completion.signal, null);
  assert.equal(buffered.length, 0);
  assert.deepEqual(image, after);
  assert.equal(rows.length, names.length);
  assert.deepEqual(
    requests.map((row) => row.id),
    responses.map((row) => row.id),
  );
  return evidence;
}
