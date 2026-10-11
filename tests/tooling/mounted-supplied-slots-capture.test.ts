import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const helper = fileURLToPath(
  new URL("../../tools/support/compat/github/mounted-supplied-slots-custody.py", import.meta.url),
);
const before = fileURLToPath(
  new URL("../_fixtures/tooling/mounted-supplied-slots-capture-before.py.bin", import.meta.url),
);
const input = Buffer.from(Array.from({ length: 4096 }, (_, index) => index % 256));
const expectedOutput = Buffer.from(input).reverse();
const expectedError = Buffer.from([0, 115, 116, 100, 101, 114, 114, 255]);
const worker = String.raw`
import io, json, os, pathlib, subprocess, sys
source = pathlib.Path(sys.argv[1])
namespace = {"__name__": "mounted_capture_control", "__file__": str(source)}
exec(compile(source.read_bytes(), str(source), "exec"), namespace)
directory = pathlib.Path(sys.argv[2])
supplied = bytes(index % 256 for index in range(4096))
stdout, stderr = io.BytesIO(), io.BytesIO()
program = "import sys; data=sys.stdin.buffer.read(); sys.stdout.buffer.write(data[::-1]); sys.stderr.buffer.write(bytes([0,115,116,100,101,114,114,255]))"
packet = namespace["forward"]([sys.executable, "-c", program], os.environ,
                              directory, io.BytesIO(supplied), stdout, stderr)
print(json.dumps({"packet": packet, "stdout": stdout.getvalue().hex(),
                  "stderr": stderr.getvalue().hex()}))
`;

test("the retained original helper reproduces lost capture under Python optimization", () => {
  assert.equal(
    createHash("sha256").update(fs.readFileSync(before)).digest("hex"),
    "7ee19082394b30e6f3415e648c3afdfaba7b5ba92de24d56f59875318d8bb009",
  );
  run(before, false, true);
  run(before, true, false);
});

test("retained writes preserve every binary stream in normal and optimized Python", () => {
  run(helper, false, true);
  run(helper, true, true);
});

function run(source: string, optimized: boolean, retained: boolean) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-mounted-capture-control-"));
  try {
    const result = spawnSync(
      "python3",
      [...(optimized ? ["-O"] : []), "-c", worker, source, directory],
      {
        encoding: "utf8",
        timeout: 10_000,
      },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stderr, "");
    const observation = JSON.parse(result.stdout);
    assert.equal(observation.packet.returncode, 0);
    assert.equal(observation.packet.signal, null);
    assert.deepEqual(observation.packet.relayErrors, []);
    assert.equal(observation.stdout, expectedOutput.toString("hex"));
    assert.equal(observation.stderr, expectedError.toString("hex"));
    const streams = [
      ["stdin.bin", input],
      ["stdout.bin", expectedOutput],
      ["stderr.bin", expectedError],
    ] as const;
    for (const [name, bytes] of streams) {
      const captured = fs.readFileSync(path.join(directory, name));
      assert.deepEqual(captured, retained ? bytes : Buffer.alloc(0));
      assert.deepEqual(observation.packet.raw[name], {
        bytes: captured.length,
        sha256: createHash("sha256").update(captured).digest("hex"),
      });
    }
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(directory, "wait.json"), "utf8")),
      observation.packet,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
