// Failure observer laws only; a missing executable cannot qualify source CLI.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { errorPacket } from "./support/n8n-cli-config-oracle.mjs";
import { runCli } from "./support/n8n-cli-config-workspace.mjs";

await test("actual failed spawn retains every own provider field before rejection", () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-failure-custody-"));
  try {
    fs.mkdirSync(path.join(workspace, "owned"));
    const binary = path.join(workspace, "missing-cli");
    const receiptPath = path.join(workspace, "whole-process.json");
    let rejected;
    assert.throws(
      () =>
        runCli({
          binary,
          workspace,
          packageRoot: "owned",
          files: ["Input.vue"],
          config: { path: path.join(workspace, "owned/vize.config.ts") },
          receiptPath,
        }),
      (error) => {
        rejected = error;
        return error instanceof assert.AssertionError && error.actual instanceof Error;
      },
    );
    const actualError = rejected.actual;
    const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
    assert.equal(receipt.error.code, "ENOENT");
    assert.equal(receipt.error.path, binary);
    assert.equal(receipt.error.errno, actualError.errno);
    assert.equal(receipt.error.syscall, actualError.syscall);
    assert.deepEqual(receipt.error.spawnargs, receipt.args);
    assert.deepEqual(receipt.error, {
      name: actualError.name,
      ...Object.fromEntries(
        Object.getOwnPropertyNames(actualError).map((name) => [name, actualError[name]]),
      ),
    });
    assert.equal(receipt.signal, null);
    assert.equal(receipt.status, null);
    assert.equal(receipt.stdout, undefined);
    assert.equal(receipt.stderr, undefined);
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

await test("nested assertion and aggregate causes retain complete Error properties", () => {
  const leaf = Object.assign(new Error("owned leaf"), { errno: -2, path: "owned/source.vue" });
  const outer = new Error("owned outer", { cause: leaf });
  outer.actual = { attempts: [leaf] };
  Object.defineProperty(outer, "nonEnumerableOwner", { value: "owned provider" });
  const failure = new AggregateError([outer], "aggregate", { cause: leaf });
  const packet = JSON.parse(JSON.stringify(errorPacket(failure)));
  assert.equal(packet.name, "AggregateError");
  assert.equal(packet.errors[0].nonEnumerableOwner, "owned provider");
  for (const nested of [
    packet.cause,
    packet.errors[0].cause,
    packet.errors[0].actual.attempts[0],
  ]) {
    assert.deepEqual(nested, {
      name: leaf.name,
      ...Object.fromEntries(Object.getOwnPropertyNames(leaf).map((name) => [name, leaf[name]])),
    });
  }
});
