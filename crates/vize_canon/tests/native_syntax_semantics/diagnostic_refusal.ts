// Failure-only peer: bootstrap succeeds, but no semantic answer is fabricated.
import assert from "node:assert/strict";
import { readSync, writeFileSync, writeSync } from "node:fs";

const [capture, rawCode, ...argv] = process.argv.slice(2);
assert(capture);
const code = Number(rawCode);
assert(code === -32800 || code === -32603);
assert.equal(argv[0], "--api");
const message = code === -32800 ? "context canceled" : "semantic request failed";
const exchanges: unknown[] = [];

// Use the same complete-frame discipline as startup_denial.ts, while allowing
// only the setup requests needed to reach the actual semantic operation.
function request() {
  const bytes: number[] = [];
  const one = Buffer.alloc(1);
  while (!Buffer.from(bytes).subarray(-4).equals(Buffer.from("\r\n\r\n"))) {
    const count = readSync(0, one, 0, 1, null);
    if (count === 0 && bytes.length === 0) return null;
    assert.equal(count, 1, "complete request header required");
    bytes.push(one[0]!);
  }
  const header = Buffer.from(bytes);
  const lengths = header
    .toString("utf8")
    .split("\r\n")
    .filter((line) => /^Content-Length: /i.test(line));
  assert.equal(lengths.length, 1);
  const length = Number(lengths[0]!.slice("Content-Length: ".length));
  assert(Number.isSafeInteger(length) && length > 0);
  const payload = Buffer.alloc(length);
  for (let offset = 0; offset < length;) {
    const count = readSync(0, payload, offset, length - offset, null);
    assert(count > 0, "complete request payload required");
    offset += count;
  }
  const body = JSON.parse(payload.toString("utf8"));
  assert.equal(body.jsonrpc, "2.0");
  assert(typeof body.id === "number" || typeof body.id === "string");
  return { body, frame: [...header, ...payload] };
}

for (;;) {
  const input = request();
  if (!input) break;
  const { body } = input;
  let result: unknown;
  switch (body.method) {
    case "initialize":
      result = { useCaseSensitiveFileNames: true, currentDirectory: process.cwd() };
      break;
    case "updateSnapshot":
      assert.equal(body.params.openProjects.length, 1);
      result = {
        snapshot: "denied-snapshot",
        projects: [
          {
            id: "denied-project",
            configFileName: body.params.openProjects[0],
            compilerOptions: {},
            rootFiles: [],
          },
        ],
      };
      break;
    case "describeCapabilities":
      result = { runtime: { capabilityEndpoint: true }, diagnostics: { project: true } };
      break;
    case "release":
      result = null;
      break;
    case "getDiagnosticsForProject":
      break;
    default:
      assert.fail(`unexpected request: ${JSON.stringify(body)}`);
  }
  const response =
    body.method === "getDiagnosticsForProject"
      ? { jsonrpc: "2.0", id: body.id, error: { code, message } }
      : { jsonrpc: "2.0", id: body.id, result };
  const payload = Buffer.from(JSON.stringify(response));
  const frame = Buffer.concat([Buffer.from(`Content-Length: ${payload.length}\r\n\r\n`), payload]);
  exchanges.push({
    request: body,
    wholeRequestFrame: input.frame,
    response,
    wholeResponseFrame: [...frame],
  });
  if (body.method === "getDiagnosticsForProject") {
    writeFileSync(capture, JSON.stringify({ argv, exchanges, request: body, response }));
  }
  for (let offset = 0; offset < frame.length;) {
    offset += writeSync(1, frame, offset, frame.length - offset);
  }
}
