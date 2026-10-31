import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { asArray, asNumber, asRecord, asString, parseJson } from "../../json-value.ts";
import { sha256 } from "../../config/windows_consumer/common.ts";
import type { CliResult } from "../../../benchmarks/scripts/canon-instance-directives-model.ts";
import type { FileImage } from "../computed_event_custody.ts";
import { hook, verifyJournal } from "../editor_vite_stub/custody.ts";
import type { capture } from "./capture.ts";

// Source 815972e4 check/runner/execution.rs emits both progress lines even for
// ordinary JSON. output.rs:112-129 prints this exact shared constant followed by
// every virtual file; output/json.rs emits those same file.content bytes.
// The immutable fixture records preamble.rs blob3bc47842 and all18 include blobs.
export const sharedPreamble = {
  bytes: 43628,
  sha256: "36165195867281533ce9cb085cc646e3dcac6aafe287e9fbc8f9aa71bf48908e",
};
export function verifyCliDebug(stderr: Buffer, result: CliResult, project: string) {
  assert.equal(result.fileCount, 54);
  assert.equal(result.files.length, 54);
  const names = result.files.map((row) => row.file);
  assert.equal(new Set(names).size, 54);
  assert.deepEqual(names, [...names].sort(), "source JSON virtual-file order must stay exact");
  const prefix = Buffer.from(
    `Building Corsa virtual project for 54 files under ${project}...\nRunning Corsa diagnostics for 54 files...\n\n=== __vize_helpers.d.ts ===\n`,
  );
  assert.ok(
    stderr.subarray(0, prefix.length).equals(prefix),
    "exact CLI progress/helper header required",
  );
  const helpers = stderr.subarray(prefix.length, prefix.length + sharedPreamble.bytes);
  assert.equal(helpers.length, sharedPreamble.bytes);
  assert.equal(sha256(helpers), sharedPreamble.sha256, "complete source shared preamble differs");
  const expected = Buffer.concat([
    prefix,
    helpers,
    Buffer.from("\n"),
    ...result.files.map((row) =>
      Buffer.from(`\n=== ${resolve(project, row.file)} ===\n${asString(row.virtualTs)}\n`),
    ),
  ]);
  assert.ok(
    stderr.equals(expected),
    "complete CLI stderr must match every declared virtual file with no remainder",
  );
  return {
    bytes: stderr.length,
    sha256: sha256(stderr),
    sharedPreamble,
    virtualFileSections: 54,
    exactNoRemainder: true,
  };
}
type Child = Pick<ReturnType<typeof capture>, "argv" | "pid" | "status" | "signal" | "error">;
export function verifyNegativeCliJournal(
  journal: string,
  expected: { root: string; cli: string; binary: FileImage; token: string },
  child: Child,
) {
  assert.equal(child.status, 1);
  assert.equal(child.error, null);
  assert.equal(child.signal, null);
  assert.ok(Number.isSafeInteger(child.pid) && child.pid > 0);
  assert.match(expected.token, /^[0-9a-f]{32}$/u);
  assert.equal(child.argv[1], "--require");
  assert.equal(child.argv[2], hook);
  assert.equal(child.argv[3], expected.cli);
  // Normal negative diagnostics call Rust finish_executions->process::exit(1),
  // bypassing Node's exit listener. ChildProcess still captures the real exit.
  // Keep the shared native-load validator's explicit null-Node-exit contract.
  verifyJournal(journal, { binary: expected.binary }, null);
  const raw = readFileSync(journal, "utf8");
  assert.ok(raw.endsWith("\n"));
  const events = raw
    .slice(0, -1)
    .split("\n")
    .map((line) => asRecord(parseJson(line)));
  assert.deepEqual(
    events.map((event) => event.event),
    ["initialized", "attempt", "returned"],
  );
  assert.deepEqual(
    Object.keys(events[0]).sort(),
    ["schema", "pid", "event", "installRoot", "nativePath", "sha256", "node", "argv"].sort(),
  );
  for (const event of events.slice(1)) {
    assert.deepEqual(
      Object.keys(event).sort(),
      ["schema", "pid", "event", "actualPath", "sha256", "expectedNative", "corsaPath"].sort(),
    );
    assert.equal(event.corsaPath, "");
  }
  for (const event of events) assert.equal(asNumber(event.pid), child.pid);
  assert.equal(events[0].installRoot, expected.root);
  assert.equal(events[0].node, child.argv[0]);
  assert.deepEqual(asArray(events[0].argv), [child.argv[0], ...child.argv.slice(3)]);
  return {
    childPid: child.pid,
    capturedChildExitStatus: 1,
    NodeExitCallbackObserved: false,
    installerNativeBuildToken: expected.token,
    nativeImage: expected.binary,
    tokenBindingSource:
      "installer identity and genuine producer receipt share the loaded image SHA; the journal has no token field",
    source: "selected815 check/runner/output.rs finish_executions nonzero Rust process::exit",
  };
}
