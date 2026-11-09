import assert from "node:assert/strict";
import { relative, sep } from "node:path";
import { sixElementTuples } from "./wire-tuples.ts";

export type Packet = Record<string, unknown>;
export function hostConfig(compilerOptions: Packet) {
  assert.ok(
    !Object.hasOwn(compilerOptions, "contentMappers"),
    "contentMappers belongs at the project root",
  );
  return {
    compilerOptions,
    contentMappers: [{ package: "vize", extensions: [".vue"] }],
    include: ["src/**/*.vue"],
  };
}
export function object(value: unknown): Packet {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Packet;
}
export function fixture(invalid: boolean, pipeCapacity: number): string {
  assert.ok(
    Number.isSafeInteger(pipeCapacity) && pipeCapacity > 0 && pipeCapacity <= 1_048_576,
    "bounded Linux pipe capacity required",
  );
  return `<script setup lang="ts">\nconst value: number = ${invalid ? '"wrong"' : "1"};\nconst padding: string = "雪🌸${"x".repeat(pipeCapacity * 2)}";\n</script>\n<template><div>{{ value }}{{ padding }}</div></template>\n`;
}
export function frames(bytes: Buffer): { packet: Packet; bytes: number }[] {
  const rows: { packet: Packet; bytes: number }[] = [];
  let offset = 0;
  while (offset < bytes.length) {
    const end = bytes.indexOf("\r\n\r\n", offset);
    assert.ok(end >= offset, "incomplete protocol header");
    const header = bytes.subarray(offset, end).toString("ascii");
    const match = /^Content-Length: ([0-9]+)$/u.exec(header);
    assert.ok(match, "unexpected protocol header");
    const length = Number(match[1]);
    assert.ok(
      Number.isSafeInteger(length) && length > 0 && length <= bytes.length - end - 4,
      "incomplete protocol body",
    );
    const text = new TextDecoder("utf-8", { fatal: true }).decode(
      bytes.subarray(end + 4, end + 4 + length),
    );
    rows.push({ packet: object(sixElementTuples(JSON.parse(text))), bytes: length });
    offset = end + 4 + length;
  }
  return rows;
}
export function qualifyWire(
  input: Buffer,
  output: Buffer,
  expected: Map<string, string>,
  pipeCapacity: number,
) {
  const requests = frames(input),
    responses = frames(output);
  assert.ok(requests.length > 0, "mapper requests absent");
  assert.deepEqual(
    responses.map((row) => row.packet.id),
    requests.map((row) => row.packet.id),
    "complete response IDs must preserve request order",
  );
  assert.equal(
    new Set(requests.map((row) => JSON.stringify(row.packet.id))).size,
    requests.length,
    "duplicate request ID",
  );
  const seen = new Set<string>();
  const transforms: Packet[] = [];
  let initialized = false;
  const opened = new Set<string>();
  for (const [index, row] of requests.entries()) {
    if (Object.hasOwn(row.packet, "jsonrpc")) assert.equal(row.packet.jsonrpc, "2.0");
    const response = responses[index].packet;
    assert.equal(response.jsonrpc, "2.0");
    assert.ok(
      !Object.hasOwn(response, "error") && Object.hasOwn(response, "result"),
      "mapper returned an error or omitted its result",
    );
    assert.ok(
      ["initialize", "openProject", "transform", "closeProject"].includes(
        String(row.packet.method),
      ),
      "unexpected mapper operation",
    );
    const params = object(row.packet.params);
    if (row.packet.method === "initialize") {
      assert.ok(
        Array.isArray(params.positionEncodings) && params.positionEncodings.includes("utf-8"),
      );
      assert.equal(
        object(response.result).positionEncoding,
        "utf-8",
        "initialize did not negotiate UTF8",
      );
      initialized = true;
      continue;
    }
    assert.ok(initialized, "mapper operation preceded successful initialize");
    assert.equal(typeof params.projectHandle, "string", "explicit project handle required");
    const handle = params.projectHandle as string;
    if (row.packet.method === "openProject") {
      const result = object(response.result);
      assert.equal(typeof result.configIdentity, "string");
      if (Object.hasOwn(result, "optionDiagnostics"))
        assert.deepEqual(result.optionDiagnostics, []);
      opened.add(handle);
      continue;
    }
    if (row.packet.method === "closeProject") {
      assert.equal(response.result, null);
      opened.delete(handle);
      continue;
    }
    assert.ok(opened.has(handle), "transform references a project that was not opened");
    assert.equal(typeof params.fileName, "string");
    assert.equal(typeof params.content, "string");
    const name = params.fileName as string;
    assert.ok(expected.has(name) && !seen.has(name), "unexpected or duplicate authored transform");
    assert.ok(
      params.content === expected.get(name),
      "mapper request changed authored source bytes: " + name,
    );
    seen.add(name);
    const result = object(response.result);
    assert.equal(typeof result.text, "string");
    assert.ok(
      Array.isArray(result.mappings) && result.mappings.length > 0,
      "mapping coverage absent",
    );
    assert.deepEqual(result.diagnostics, [], "unexpected mapper diagnostics");
    for (const mapping of result.mappings) {
      assert.ok(Array.isArray(mapping) && mapping.length === 6);
      assert.ok(mapping.every((value) => Number.isSafeInteger(value) && value >= 0));
      assert.ok(mapping[0] + mapping[1] <= Buffer.byteLength(result.text as string));
      assert.ok(mapping[2] + mapping[3] <= Buffer.byteLength(params.content as string));
    }
    assert.ok(
      row.bytes > pipeCapacity && responses[index].bytes > pipeCapacity,
      "each authored request/result must exceed the recorded kernel pipe upper bound",
    );
    transforms.push({ request: row.packet, response });
  }
  assert.deepEqual([...seen].sort(), [...expected.keys()].sort(), "missing authored transform");
  assert.ok(
    requests.some((row) => row.bytes > pipeCapacity),
    "no individual request exceeds pipe capacity",
  );
  assert.ok(
    responses.some((row) => row.bytes > pipeCapacity),
    "no individual response exceeds pipe capacity",
  );
  return { requests, responses, transforms };
}
export function diagnosticPacket(
  stdout: string,
  stderr: string,
  project: string,
  expectedErrors: readonly number[],
) {
  assert.equal(stderr, "", "host stderr must be empty");
  const lines = stdout.replaceAll("\r\n", "\n").split("\n").filter(Boolean);
  const rows = lines.map((line) => {
    const match = /^(.*\.vue)\(([0-9]+),([0-9]+)\): error TS([0-9]+): (.*)$/u.exec(line);
    assert.ok(match, "unexpected diagnostic output: " + line);
    const file = match[1].startsWith(project + sep) ? relative(project, match[1]) : match[1];
    return {
      file: file.replaceAll("\\", "/"),
      line: Number(match[2]),
      column: Number(match[3]),
      code: Number(match[4]),
      message: match[5],
      end: null,
      related: null,
    };
  });
  const expected = expectedErrors.map((index) => ({
    file: `src/File${index}.vue`,
    line: 2,
    column: 7,
    code: 2322,
    message: "Type 'string' is not assignable to type 'number'.",
    end: null,
    related: null,
  }));
  assert.deepEqual(rows, expected, "whole authored diagnostic population changed");
  return rows;
}
export function semanticPacket(
  status: number | null,
  stdout: string,
  stderr: string,
  project: string,
  expectedErrors: readonly number[],
) {
  assert.equal(status, expectedErrors.length ? 2 : 0, "pinned noEmit semantic status changed");
  return diagnosticPacket(stdout, stderr, project, expectedErrors);
}
/** Host disposal can kill the tee before it commits a child-exit receipt. */
export function mapperExitReceipt(value: unknown) {
  if (value === undefined) return { exitObserved: false, status: null, signal: null };
  const result = object(value);
  assert.equal(result.status, 0, "observed mapper child exit failed");
  assert.equal(result.signal, null, "observed mapper child exit was signaled");
  assert.equal(result.error, null, "observed mapper child spawn or pipe failed");
  return { exitObserved: true, status: 0, signal: null };
}
