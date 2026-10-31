import assert, { AssertionError } from "node:assert/strict";
import { createHash } from "node:crypto";
import { closeSync, openSync, readFileSync, readSync, statSync } from "node:fs";
import { join } from "node:path";

export const DEPTHS = [128, 512, 1100, 2048, 4094] as const;
export const STACKS = Array.from({ length: 10 }, (_, power) => 256 * 1024 * 2 ** power);
export const TIMEOUT_SECONDS = 30;
export const OUTPUT_LIMIT_BYTES = 512 * 1024 ** 2;
export const RSS_LIMIT_BYTES = 2 * 1024 ** 3;
export type Shape = "v-for" | "v-slot";
export type Profile = "ci" | "ci-opt";

export interface Observation {
  returncode: number;
  timedOut: boolean;
  timeoutSeconds: number;
  rssLimitExceeded: boolean;
  peakProcessTreeRssBytes: number;
  wallSeconds: number;
}

export interface TrialResult extends Observation {
  shape: Shape;
  depth: number;
  inheritedMinStackBytes: number;
  stackOverflow: boolean;
  stdinTransport: "file";
  stdinSha256: string;
  stdoutSha256: string;
  stderrSha256: string;
  stdoutBytes: number;
  accepted: boolean;
  virtualTextSha256?: string;
  validationError?: string;
}

export function hashFile(path: string): string {
  const hash = createHash("sha256"),
    buffer = Buffer.allocUnsafe(64 * 1024);
  const descriptor = openSync(path, "r");
  try {
    for (let count = readSync(descriptor, buffer); count > 0; count = readSync(descriptor, buffer))
      hash.update(buffer.subarray(0, count));
    return hash.digest("hex");
  } finally {
    closeSync(descriptor);
  }
}

export function witness(shape: Shape, depth: number): string {
  let source =
    '<script setup lang="ts">\nimport Child from "./Child.vue";\nconst rows = [1];\n</script>\n<template>\n';
  for (let index = 0; index < depth; index++)
    source +=
      shape === "v-for"
        ? `<template v-for="row${index} in rows">\n`
        : `<Child v-slot="slot${index}">\n`;
  source +=
    shape === "v-for"
      ? `<Child :value="row${depth - 1}" />\n`
      : `<Child :value="slot${depth - 1}.value" />\n`;
  return (
    source + (shape === "v-for" ? "</template>\n" : "</Child>\n").repeat(depth) + "</template>\n"
  );
}

export function frames(source: string): Buffer {
  const requests = [
    { id: 1, method: "initialize", params: { positionEncodings: ["utf-8"] } },
    { id: 2, method: "openProject", params: { configFileName: "", projectHandle: "stack" } },
    {
      id: 3,
      method: "transform",
      params: { fileName: "/project/Stack.vue", content: source, projectHandle: "stack" },
    },
  ];
  const parts: Buffer[] = [];
  for (const request of requests) {
    // Match json.dumps(..., ensure_ascii=True, separators=(",", ":")) exactly.
    const text = JSON.stringify({ jsonrpc: "2.0", ...request }).replace(
      /[^\x20-\x7e]/g,
      (character) => `\\u${character.charCodeAt(0).toString(16).padStart(4, "0")}`,
    );
    const body = Buffer.from(text);
    parts.push(Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body);
  }
  return Buffer.concat(parts);
}

class PacketReader {
  private pending: Buffer = Buffer.alloc(0);
  private readonly descriptor: number;
  constructor(descriptor: number) {
    this.descriptor = descriptor;
  }

  line(): Buffer {
    const parts: Buffer[] = [];
    for (;;) {
      const newline = this.pending.indexOf(10);
      if (newline !== -1) {
        parts.push(this.pending.subarray(0, newline + 1));
        this.pending = this.pending.subarray(newline + 1);
        return Buffer.concat(parts);
      }
      if (this.pending.length) parts.push(this.pending);
      const buffer = Buffer.allocUnsafe(64 * 1024),
        count = readSync(this.descriptor, buffer);
      this.pending = buffer.subarray(0, count);
      if (!count) return Buffer.concat(parts);
    }
  }

  bytes(length: number): Buffer {
    const body = Buffer.allocUnsafe(length);
    let count = Math.min(length, this.pending.length);
    this.pending.copy(body, 0, 0, count);
    this.pending = this.pending.subarray(count);
    while (count < length) {
      const read = readSync(this.descriptor, body, count, length - count, null);
      if (!read) break;
      count += read;
    }
    return body.subarray(0, count);
  }
}

function record(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function checkResponse(path: string, shape: Shape, depth: number): string {
  const descriptor = openSync(path, "r"),
    reader = new PacketReader(descriptor);
  let textHash = "";
  try {
    for (const expectedId of [1, 2, 3]) {
      const prefix = Buffer.from("Content-Length: "),
        header = reader.line();
      assert(header.subarray(0, prefix.length).equals(prefix));
      const digits = header
        .subarray(prefix.length)
        .toString("latin1")
        .replace(/^[\t\n\v\f\r ]+|[\t\n\v\f\r ]+$/g, "");
      assert(/^[+-]?[0-9](?:_?[0-9])*$/.test(digits));
      const length = Number(digits.replaceAll("_", ""));
      assert(length > 0 && length <= OUTPUT_LIMIT_BYTES);
      assert(reader.line().equals(Buffer.from("\r\n")));
      const body = reader.bytes(length);
      assert.equal(body.length, length);
      const response: unknown = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(body));
      assert(record(response));
      assert.equal(response.jsonrpc, "2.0");
      // Preserve the historical Python numeric-equality verdict for JSON booleans.
      assert(
        response.id === expectedId ||
          (typeof response.id === "boolean" && Number(response.id) === expectedId),
      );
      assert(!Object.hasOwn(response, "error"));
      if (expectedId === 3) {
        assert(record(response.result) && typeof response.result.text === "string");
        const text = response.result.text,
          marker = `// Component props in ${shape} scope: `;
        let count = 0;
        for (
          let offset = text.indexOf(marker);
          offset !== -1;
          offset = text.indexOf(marker, offset + marker.length)
        )
          count++;
        assert.equal(count, depth);
        assert(text.includes(shape === "v-for" ? `row${depth - 1}` : `slot${depth - 1}.value`));
        assert(!/[\uD800-\uDFFF]/u.test(text), "unpaired surrogate cannot be encoded as UTF-8");
        textHash = createHash("sha256").update(text).digest("hex");
      }
    }
    assert.equal(reader.bytes(1).length, 0, "unexpected trailing response bytes");
    return textHash;
  } finally {
    closeSync(descriptor);
  }
}

export function readTrialResult(
  directory: string,
  shape: Shape,
  depth: number,
  stack: number,
  observation: Observation,
): TrialResult {
  const stdout = join(directory, "stdout.wire"),
    stderr = join(directory, "stderr.txt");
  const result: TrialResult = {
    shape,
    depth,
    inheritedMinStackBytes: stack,
    ...observation,
    stackOverflow: readFileSync(stderr).includes("stack overflow"),
    stdinTransport: "file",
    stdinSha256: hashFile(join(directory, "stdin.wire")),
    stdoutSha256: hashFile(stdout),
    stderrSha256: hashFile(stderr),
    stdoutBytes: statSync(stdout).size,
    accepted: false,
  };
  if (result.returncode === 0) {
    try {
      result.virtualTextSha256 = checkResponse(stdout, shape, depth);
      assert.equal(statSync(stderr).size, 0, "successful mapper wrote stderr");
      result.accepted = true;
    } catch (error) {
      if (
        !(
          error instanceof AssertionError ||
          error instanceof SyntaxError ||
          error instanceof TypeError
        )
      )
        throw error;
      result.validationError = error.message;
    }
  }
  return result;
}

export function requireStackFailure(result: TrialResult): void {
  assert(!result.timedOut, "timeout is not a stack-budget observation");
  assert(!result.rssLimitExceeded, "bounded RSS limit reached; stop the probe");
  assert.notEqual(result.returncode, 0, "semantic refusal is not a stack-budget observation");
  assert(result.stackOverflow, "non-stack failure is not a stack-budget observation");
}
