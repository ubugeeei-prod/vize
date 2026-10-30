import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  OUTPUT_LIMIT_BYTES,
  RSS_LIMIT_BYTES,
  checkResponse,
  frames,
  hashFile,
  readTrialResult,
  requireStackFailure,
  witness,
  type Observation,
  type Shape,
} from "../../tools/benchmarks/scripts/content-mapper-worker-stack-core.ts";
import {
  boundedCommand,
  processTreeRss,
} from "../../tools/benchmarks/scripts/content-mapper-worker-stack.ts";

const sha256 = (body: string | Uint8Array) => createHash("sha256").update(body).digest("hex");
const observation: Observation = {
  returncode: 0,
  timedOut: false,
  timeoutSeconds: 30,
  rssLimitExceeded: false,
  peakProcessTreeRssBytes: 1234,
  wallSeconds: 0.25,
};

function response(
  text: string,
  mutate?: (packets: Array<Record<string, unknown>>) => void,
): Buffer {
  const packets: Array<Record<string, unknown>> = [
    { id: 1, jsonrpc: "2.0", result: {} },
    { id: 2, jsonrpc: "2.0", result: {} },
    { id: 3, jsonrpc: "2.0", result: { text } },
  ];
  mutate?.(packets);
  return Buffer.concat(
    packets.flatMap((packet) => {
      const body = Buffer.from(JSON.stringify(packet));
      return [Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body];
    }),
  );
}

function text(shape: Shape): string {
  return Array.from(
    { length: 3 },
    (_, index) =>
      `// Component props in ${shape} scope: ${shape === "v-for" ? `row${index}` : `slot${index}.value`}\n`,
  ).join("");
}

test("witness and ASCII JSON wire bytes match the retained historical driver", () => {
  // Captured from the unmodified 086fa5204a6bbb89ee79238212251fa263f9130d source.
  assert.equal(
    sha256(frames(witness("v-for", 3))),
    "855f71cae5fe773ea9c9861aef10b9c20b199f6babc3d1432255446770cfe4a5",
  );
  assert.equal(
    sha256(frames(witness("v-slot", 3))),
    "d472f1892261dd7bcb4fbbff0f1fd9695fa267ff603400dfd128a655a3df3014",
  );
  assert.equal(
    sha256(frames("雪🌸\x7f\x00\n")),
    "355a67b07b9ac8a0b48b7e28147fe2e5287754eaf699c180e054da328e1f1a4b",
  );
  assert.equal(
    sha256(frames("lone:\ud800")),
    "f268f74c082c5c813402ce18381481a7a2c6951254f52ed3dfab66edbaf76fb6",
  );
});

test("complete response bytes, decoded text hash and all three ordered frames remain required", (context) => {
  const root = mkdtempSync(join(tmpdir(), "vize-worker-stack-core-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  for (const shape of ["v-for", "v-slot"] as const) {
    const wire = response(text(shape)),
      path = join(root, "response.wire");
    writeFileSync(path, wire);
    assert.equal(checkResponse(path, shape, 3), sha256(text(shape)));
    assert.equal(hashFile(path), sha256(wire));
    for (const malformed of [
      response(text(shape), (rows) => {
        rows[0].id = 2;
      }),
      response(text(shape), (rows) => {
        rows[1].jsonrpc = "1.0";
      }),
      response(text(shape), (rows) => {
        rows[2].error = null;
      }),
      response(text(shape).replace("// Component props", "// changed")),
      response(text(shape).replace(shape === "v-for" ? "row2" : "slot2.value", "missing")),
      wire.subarray(0, wire.length - 1),
      Buffer.concat([wire, Buffer.from("x")]),
      Buffer.from(`Content-Length: ${OUTPUT_LIMIT_BYTES + 1}\r\n\r\n`),
      Buffer.from("Content-Length: 2x\r\n\r\n{}"),
      Buffer.from("Content-Length: 1\r\n\r\n\xff", "latin1"),
    ]) {
      writeFileSync(path, malformed);
      assert.throws(() => checkResponse(path, shape, 3));
    }
    // These are historical Python validator semantics, not product protocol relaxations.
    writeFileSync(
      path,
      response(text(shape), (rows) => {
        rows[0].id = true;
      }),
    );
    assert.equal(checkResponse(path, shape, 3), sha256(text(shape)));
  }
});

test("timeout, RSS, real stack overflow and unrelated failures remain distinct", (context) => {
  const root = mkdtempSync(join(tmpdir(), "vize-worker-stack-verdict-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  writeFileSync(join(root, "stdin.wire"), frames(witness("v-for", 3)));
  writeFileSync(join(root, "stdout.wire"), response(text("v-for")));
  writeFileSync(join(root, "stderr.txt"), "");
  const accepted = readTrialResult(root, "v-for", 3, 262144, observation);
  assert(accepted.accepted);
  writeFileSync(join(root, "stderr.txt"), "unexpected warning");
  const invalid = readTrialResult(root, "v-for", 3, 262144, observation);
  assert(!invalid.accepted && typeof invalid.validationError === "string");
  assert.throws(() => requireStackFailure(invalid), /semantic refusal/);
  writeFileSync(join(root, "stderr.txt"), "fatal runtime error: stack overflow, aborting\n");
  const overflow = readTrialResult(root, "v-for", 3, 262144, { ...observation, returncode: 134 });
  assert(!overflow.accepted && overflow.stackOverflow);
  requireStackFailure(overflow);
  assert.throws(
    () => requireStackFailure({ ...overflow, returncode: -9, timedOut: true }),
    /timeout/,
  );
  assert.throws(
    () => requireStackFailure({ ...overflow, returncode: -9, rssLimitExceeded: true }),
    /RSS/,
  );
  assert.throws(
    () => requireStackFailure({ ...overflow, returncode: 1, stackOverflow: false }),
    /non-stack/,
  );
});

test("Linux resource limits, process-tree RSS and vanished-child handling stay bounded", () => {
  assert.deepEqual(boundedCommand("/source path/vize", "/out path", 90), [
    "--core=0:0",
    `--fsize=${OUTPUT_LIMIT_BYTES}:${OUTPUT_LIMIT_BYTES}`,
    "--cpu=90:91",
    "--",
    "/usr/bin/time",
    "-v",
    "-o",
    "/out path/resources.txt",
    "/source path/vize",
    "content-mapper",
  ]);
  assert.equal(RSS_LIMIT_BYTES, 2147483648);
  const files: Record<string, string> = {
    "/proc/1/statm": "100 2 0",
    "/proc/1/task/1/children": "2 3\n",
    "/proc/2/statm": "100 3 0",
    "/proc/2/task/2/children": "",
  };
  const read = (path: string): string => {
    if (Object.hasOwn(files, path)) return files[path];
    throw Object.assign(new Error("gone"), { code: "ENOENT" });
  };
  assert.equal(processTreeRss(1, 4096, read), 5 * 4096);
  assert.throws(
    () =>
      processTreeRss(1, 4096, () => {
        throw Object.assign(new Error("denied"), { code: "EACCES" });
      }),
    /denied/,
  );
  assert.throws(() => processTreeRss(1, 4096, () => "invalid"), /invalid process RSS/);
  assert.throws(
    () => processTreeRss(1, 4096, (path) => (path.endsWith("statm") ? "100 2 0" : "bad")),
    /invalid child process id/,
  );
});
