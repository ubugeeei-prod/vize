// Filesystem/process controls only: the loader below never executes an addon.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { toolingProgram } from "./support/program-path.ts";

const hook = fileURLToPath(
  new URL("../../npm/linter/oxlint/scripts/project-native-custody.cjs", import.meta.url),
);
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const program = toolingProgram("native/observe-oxlint-addon.cjs");

function run(mode: string, selectedHook = hook) {
  const directory = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-custody-control-")),
  );
  const binary = path.join(directory, "source.node");
  const bytes = Buffer.alloc(150_013);
  for (let index = 0; index < bytes.length; index++) bytes[index] = index % 251;
  const original = Buffer.from("<template>unchanged original</template>\n");
  const calls = path.join(directory, "calls.jsonl");
  const source = { head: "authored-process-control", tree: "no-native-qualification" };
  const custody = {
    schema: "vize.oxlint.source-native",
    version: 1,
    source,
    binary: { path: binary, sha256: hash(bytes) },
    calls,
  };
  try {
    fs.writeFileSync(binary, bytes);
    fs.utimesSync(binary, new Date(1_600_000_000_000), new Date(1_600_000_000_000));
    fs.writeFileSync(path.join(directory, "unrelated.node"), "unrelated inert addon bytes\n");
    fs.writeFileSync(path.join(directory, "vize-vitrine.alias.node"), "other requested bytes\n");
    fs.writeFileSync(path.join(directory, "original.vue"), original);
    fs.writeFileSync(path.join(directory, "custody.json"), JSON.stringify(custody));
    const environment = { ...process.env };
    delete environment.NODE_OPTIONS;
    delete environment.VIZE_OXLINT_NATIVE_CONTROL;
    delete environment.VIZE_OXLINT_NATIVE_HINT;
    const result = spawnSync(process.execPath, [program, selectedHook, directory, mode], {
      env: environment,
      timeout: 10_000,
      encoding: "utf8",
      maxBuffer: 1024 * 1024,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stderr, "");
    const packet = JSON.parse(result.stdout);
    const events = fs.existsSync(calls)
      ? fs
          .readFileSync(calls, "utf8")
          .trim()
          .split("\n")
          .map((line) => JSON.parse(line))
      : [];
    if (mode === "valid" || mode === "alias") {
      assert.deepEqual(
        events.map((event) => event.kind),
        ["load", "call"],
      );
      for (const event of events) {
        assert.deepEqual(event.source, source);
        assert.equal(event.binary, binary);
        assert.equal(event.binarySha256, custody.binary.sha256);
        assert.equal(event.pid > 0, true);
        assert.equal(event.control, null);
        assert.equal(event.hasRuleHint, false);
      }
      assert.equal(
        events[0].requested,
        mode === "alias" ? path.join(directory, "vize-vitrine.alias.node") : binary,
      );
      assert.equal(events[1].outcome, "return");
      assert.equal(events[1].originalSha256, hash(original));
      assert.equal(events[1].args[0], original.toString());
      assert.deepEqual(events[1].result, {
        source: original.toString(),
        options: { filename: path.join(directory, "original.vue") },
      });
    } else assert.deepEqual(events, [], "no authenticated Vize-native claim may escape");
    return packet;
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

void test("the complete current image is streamed before loading, with unchanged native receipts", () => {
  const packet = run("valid");
  assert.equal(packet.bytes, 150_013);
  assert.equal(packet.loads, 1);
  assert.equal(run("alias").bytes, 150_013);
});
void test("stock-only registration and unrelated addon loads make no Vize-native claim", () => {
  assert.equal(run("pure").reads, 0);
  assert.equal(run("unrelated").reads, 0);
});
void test("same-size and preserved-mtime mutation after registration refuses before load", () => {
  run("mutated");
});
void test("whole-image open and read failures refuse without a loader or receipt", () => {
  run("missing");
  run("read-error");
});
void test("mutation during hashing and replacement of the physical path both refuse", () => {
  run("during-read");
  run("replace-path");
  run("symlink-read");
});
