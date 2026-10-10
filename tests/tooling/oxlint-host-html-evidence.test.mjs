import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { createHostCommandRunner } from "../../npm/oxlint/scripts/ci-host-worker.mjs";

void test("HTML children preserve complete transport evidence and retain strict failed child diagnostics", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-host-html-evidence-"));
  const output = path.join(temporary, "output");
  fs.mkdirSync(output);
  const otherRaw = new Map([
    ["install.json", Buffer.from("original install diagnostics\r\n")],
    ["native-calls.jsonl", Buffer.from("original authenticated native calls\n")],
    ["original-project.json", Buffer.from('{"qualified":{"imports":true,"typeAware":true}}\n')],
  ]);
  for (const [name, bytes] of otherRaw) fs.writeFileSync(path.join(output, name), bytes);
  const run = createHostCommandRunner({ temporary, output, version: "controlled-child" });
  const transport = { stdout: " \r\ntransport\r\n\t", stderr: "\n transport stderr \r\n" };
  const html = { stdout: "\r\n HTML output \r\n\n", stderr: " \nHTML stderr\t\r\n" };
  const child = (streams, status = 0) => [
    "-e",
    `process.stdout.write(${JSON.stringify(streams.stdout)}); process.stderr.write(${JSON.stringify(streams.stderr)}); process.exit(${status});`,
  ];
  try {
    run(process.execPath, child(transport));
    const before = fs.readFileSync(path.join(output, "tests.json"));
    run(process.execPath, child(html));
    assert.notDeepEqual(fs.readFileSync(path.join(output, "tests.json")), before);
    assert.equal(JSON.parse(fs.readFileSync(path.join(output, "tests.json"))).stdout, html.stdout);

    run(process.execPath, child(transport));
    const retainedTransport = fs.readFileSync(path.join(output, "tests.json"));
    run(process.execPath, child(html), {}, "html-tests");
    assert.deepEqual(fs.readFileSync(path.join(output, "tests.json")), retainedTransport);
    const successfulHtml = JSON.parse(fs.readFileSync(path.join(output, "html-tests.json")));
    assert.equal(successfulHtml.stdout, html.stdout);
    assert.equal(successfulHtml.stderr, html.stderr);
    assert.equal(successfulHtml.status, 0);
    assert.equal(successfulHtml.signal, null);

    assert.throws(
      () => run(process.execPath, child(html, 7), {}, "html-tests"),
      /Oxlint controlled-child original-project qualification failed/,
    );
    const failedHtml = JSON.parse(fs.readFileSync(path.join(output, "html-tests.json")));
    assert.equal(failedHtml.status, 7);
    assert.equal(failedHtml.signal, null);
    assert.equal(failedHtml.stdout, html.stdout);
    assert.equal(failedHtml.stderr, html.stderr);
    assert.deepEqual(fs.readFileSync(path.join(output, "tests.json")), retainedTransport);
    const originalTransport = JSON.parse(retainedTransport);
    assert.equal(originalTransport.stdout, transport.stdout);
    assert.equal(originalTransport.stderr, transport.stderr);
    for (const [name, bytes] of otherRaw) {
      assert.deepEqual(fs.readFileSync(path.join(output, name)), bytes);
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
