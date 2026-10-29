import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { TextDecoder } from "node:util";
import { array, record, run, save, string, type RecordValue } from "./io.ts";
function frames(bytes: Buffer): RecordValue[] {
  const result: RecordValue[] = [];
  let offset = 0;
  while (offset < bytes.length) {
    const end = bytes.indexOf("\r\n\r\n", offset);
    assert.ok(end >= offset);
    const match = /^Content-Length: (\d+)$/iu.exec(bytes.subarray(offset, end).toString("ascii"));
    assert.ok(match);
    const length = Number(string(match[1]));
    assert.ok(Number.isSafeInteger(length) && length >= 0);
    const start = end + 4;
    assert.ok(start + length <= bytes.length);
    const value: unknown = JSON.parse(
      new TextDecoder("utf-8", { fatal: true }).decode(bytes.subarray(start, start + length)),
    );
    const packet = record(value);
    assert.equal(packet.jsonrpc, "2.0");
    assert.ok(!Object.hasOwn(packet, "error"));
    assert.ok(Object.hasOwn(packet, "result"));
    result.push(packet);
    offset = start + length;
  }
  assert.equal(offset, bytes.length);
  return result;
}
export function transform(
  cli: string,
  project: string,
  output: string,
  label: string,
  source: string,
  compilerOptions: RecordValue,
  expectedImport: string,
): { text: string; extension: string; packet: RecordValue; mappings: number[][] } {
  const requests = [
    {
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: { positionEncodings: ["utf-8"], locale: "en-US" },
    },
    {
      jsonrpc: "2.0",
      id: 2,
      method: "openProject",
      params: {
        configFileName: path.join(project, "tsconfig.json"),
        projectHandle: "published-canonical-sdk",
        options: null,
        compilerOptions,
      },
    },
    {
      jsonrpc: "2.0",
      id: 3,
      method: "transform",
      params: {
        fileName: path.join(project, "Probe.vue"),
        content: source,
        projectHandle: "published-canonical-sdk",
      },
    },
    {
      jsonrpc: "2.0",
      id: 4,
      method: "closeProject",
      params: { projectHandle: "published-canonical-sdk" },
    },
  ];
  const input = Buffer.concat(
    requests.map((value) => {
      const body = Buffer.from(JSON.stringify(value));
      return Buffer.concat([Buffer.from("Content-Length: " + body.length + "\r\n\r\n"), body]);
    }),
  );
  const observed = run(
    output,
    label + "/rpc",
    process.execPath,
    [cli, "content-mapper"],
    project,
    input,
  );
  assert.equal(observed.status, 0);
  const responses = frames(observed.stdout);
  assert.deepEqual(
    responses.map((response) => response.id),
    [1, 2, 3, 4],
  );
  const response = responses[2];
  assert.ok(response);
  const packet = record(response.result),
    text = string(packet.text),
    extension = string(packet.extension);
  assert.equal(extension, ".tsx");
  assert.ok(
    text.includes(expectedImport),
    "the actual public mapper must select all 53 shared aliases",
  );
  assert.ok(
    !text.includes("type __EmitShape<"),
    "the actual transform must omit the inline canonical definition",
  );
  assert.deepEqual(array(packet.diagnostics), []);
  const generatedBytes = Buffer.byteLength(text),
    authoredBytes = Buffer.byteLength(source);
  const mappings = array(packet.mappings).map((value) => {
    const span = array(value);
    assert.equal(span.length, 6);
    const tuple = span.map((item) => {
      assert.ok(typeof item === "number" && Number.isSafeInteger(item) && item >= 0);
      return item;
    });
    const [generated, generatedLength, authored, authoredLength] = tuple;
    assert.ok(
      generated !== undefined &&
        generatedLength !== undefined &&
        authored !== undefined &&
        authoredLength !== undefined,
    );
    assert.ok(
      generated + generatedLength <= generatedBytes && authored + authoredLength <= authoredBytes,
    );
    return tuple;
  });
  assert.ok(mappings.length > 0);
  save(path.join(output, label, "requests.json"), requests);
  save(path.join(output, label, "responses.json"), responses);
  fs.writeFileSync(path.join(output, label, "input.vue.txt"), source);
  fs.writeFileSync(path.join(output, label, "virtual" + extension + ".txt"), text);
  return { text, extension, packet, mappings };
}
