import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { createCaptureOutput } from "./support/lsp/bind-style/capture-output.ts";

test("consecutive bind-style captures preserve cached records and count only their own processes", (t) => {
  const parent = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bind-style-capture-"));
  t.after(() => fs.rmSync(parent, { recursive: true, force: true }));
  const cached = path.join(parent, "literal-original");
  const records = new Map([
    ["process-17", Buffer.from('{"cwd":"previous-project","pid":17}\n')],
    ["process-19", Buffer.from('{"cwd":"other-project","pid":19}\n')],
  ]);
  for (const [name, bytes] of records) {
    fs.mkdirSync(path.join(cached, name), { recursive: true });
    fs.writeFileSync(path.join(cached, name, "process.json"), bytes);
  }
  const priorResult = Buffer.from('{"status":"previous result"}\n');
  fs.writeFileSync(path.join(parent, "result.json"), priorResult);
  const processes = (directory: string) =>
    fs.readdirSync(directory).filter((name) => name.startsWith("process-"));
  // The previous fixed leaf selects both cached records for its one-process oracle.
  assert.deepEqual(processes(cached), ["process-17", "process-19"]);

  const first = createCaptureOutput(parent, parent);
  const second = createCaptureOutput(parent, parent);
  assert.notEqual(first, second);
  for (const output of [first, second]) {
    assert.equal(path.dirname(output), parent);
    assert.deepEqual(fs.readdirSync(output), []);
    for (const leaf of ["literal-original", "authored-positive"]) {
      const capture = path.join(output, leaf);
      fs.mkdirSync(path.join(capture, "process-17"), { recursive: true });
      fs.writeFileSync(path.join(capture, "process-17", "client.bin"), output);
      assert.deepEqual(processes(capture), ["process-17"]);
      assert.equal(fs.readFileSync(path.join(capture, "process-17", "client.bin"), "utf8"), output);
    }
  }
  for (const [name, bytes] of records)
    assert.deepEqual(fs.readFileSync(path.join(cached, name, "process.json")), bytes);
  assert.deepEqual(fs.readFileSync(path.join(parent, "result.json")), priorResult);
  assert.deepEqual(processes(cached), ["process-17", "process-19"]);
  const defaultOutput = createCaptureOutput(parent);
  assert.equal(
    path.dirname(defaultOutput),
    path.join(parent, "target/differential/lsp-bind-style-code-actions"),
  );
  assert.deepEqual(fs.readdirSync(defaultOutput), []);
});
