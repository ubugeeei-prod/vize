import { Buffer } from "node:buffer";
import { spawnSync } from "node:child_process";

import type { CommandResult } from "../../../_helpers/realworld-typecheck.ts";

export type RecordObservation = (value: unknown) => void;
type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };

/** Include non-enumerable error fields, nested causes and cycles without JSON loss. */
export function ownError(error: unknown): JsonValue {
  const objects = new Map<object, number>();
  const symbols = new Map<symbol, number>();
  function key(value: string | symbol): JsonValue {
    if (typeof value === "string") return { type: "string", value };
    if (!symbols.has(value)) symbols.set(value, symbols.size + 1);
    return {
      type: "symbol",
      id: symbols.get(value)!,
      description: value.description ?? null,
      globalKey: Symbol.keyFor(value) ?? null,
    };
  }
  function serialize(value: unknown): JsonValue {
    if (value === null || typeof value === "string" || typeof value === "boolean") return value;
    if (typeof value === "undefined") return { type: "undefined" };
    if (typeof value === "bigint") return { type: "bigint", value: String(value) };
    if (typeof value === "symbol") return key(value);
    if (typeof value === "number") {
      return Number.isFinite(value) && !Object.is(value, -0)
        ? value
        : { type: "number", value: Object.is(value, -0) ? "-0" : String(value) };
    }
    const object = value as object;
    const previous = objects.get(object);
    if (previous !== undefined) return { reference: previous };
    const id = objects.size + 1;
    objects.set(object, id);
    if (Buffer.isBuffer(object)) {
      const bytes = object as Uint8Array;
      return {
        id,
        type: "buffer",
        bytes: bytes.byteLength,
        base64: Buffer.from(bytes).toString("base64"),
      };
    }
    const properties = Reflect.ownKeys(object).map((name): JsonValue => {
      const descriptor = Object.getOwnPropertyDescriptor(object, name)!;
      const property: { [key: string]: JsonValue } = {
        key: key(name),
        enumerable: descriptor.enumerable ?? false,
        configurable: descriptor.configurable ?? false,
      };
      if (Object.hasOwn(descriptor, "value")) {
        property.writable = descriptor.writable ?? false;
        property.value = serialize(descriptor.value);
      } else {
        property.get = serialize(Reflect.get(descriptor, "get"));
        property.set = serialize(Reflect.get(descriptor, "set"));
      }
      return property;
    });
    const snapshot: { [key: string]: JsonValue } = {
      id,
      type:
        typeof value === "function"
          ? "function"
          : Array.isArray(object)
            ? "array"
            : object instanceof Error
              ? "error"
              : "object",
      properties,
    };
    if (object instanceof Error) {
      let prototype: object | null = object;
      while (prototype !== null) {
        const descriptor = Object.getOwnPropertyDescriptor(prototype, "name");
        if (descriptor) {
          if (typeof descriptor.value === "string") snapshot.name = descriptor.value;
          break;
        }
        prototype = Object.getPrototypeOf(prototype) as object | null;
      }
    }
    if (typeof value === "function") snapshot.source = Function.prototype.toString.call(value);
    return snapshot;
  }
  return serialize(error);
}

function outputBytes(value: Uint8Array | null | undefined) {
  if (value === undefined) return { type: "undefined" };
  return value === null
    ? null
    : { bytes: value.byteLength, base64: Buffer.from(value).toString("base64") };
}

/** Keep exact process output before throwing or allowing any caller to parse it. */
export function rawCommand(
  command: string,
  args: string[],
  cwd: string,
  record: RecordObservation,
  context: Record<string, unknown>,
): CommandResult {
  const options = {
    cwd,
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 64 * 1024 * 1024,
    timeout: 120_000,
  };
  const invocation = {
    ...context,
    command,
    args,
    cwd,
    locale: { LANG: options.env.LANG, LC_ALL: options.env.LC_ALL },
    environment: "inherited process.env with the recorded locale overrides",
    maxBuffer: options.maxBuffer,
    timeout: options.timeout,
    encoding: "buffer",
  };
  record({ ...invocation, observation: "process-start" });
  let result;
  try {
    result = spawnSync(command, args, options);
  } catch (error) {
    record({
      ...invocation,
      observation: "process-threw",
      status: null,
      signal: null,
      stdout: null,
      stderr: null,
      error: ownError(error),
    });
    throw error;
  }
  record({
    ...invocation,
    observation: "process-result",
    pid: result.pid,
    status: result.status,
    signal: result.signal,
    stdout: outputBytes(result.stdout),
    stderr: outputBytes(result.stderr),
    output: result.output?.map(outputBytes) ?? null,
    error: result.error === undefined ? null : ownError(result.error),
  });
  if (result.error !== undefined) throw result.error;
  return {
    status: result.status,
    stderr: result.stderr.toString("utf8"),
    stdout: result.stdout.toString("utf8"),
  };
}
