import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asArray, asRecord, parseJson } from "../../json-value.ts";
import { hook, verifyJournal } from "../editor_vite_stub/custody.ts";
import type { FileImage } from "../computed_event_custody.ts";
import type { capture } from "../constructor_original27_public/capture.ts";

/** Join the hook's real schema to the independent captured spawn PID and exact command. */
export function nativeJournal(
  journal: string,
  installed: { root: string; cli: string; native: FileImage },
  child: Pick<ReturnType<typeof capture>, "pid" | "argv" | "status" | "signal" | "error">,
) {
  assert.equal(child.error, null);
  assert.equal(child.signal, null);
  assert.ok(child.status === 0 || child.status === 1);
  assert.ok(Number.isSafeInteger(child.pid) && child.pid > 0);
  assert.deepEqual(child.argv.slice(0, 4), [process.execPath, "--require", hook, installed.cli]);
  // Normal negative check diagnostics call Rust process::exit(1); that path has no Node exit callback.
  const exit = child.status === 1 ? null : 0;
  verifyJournal(journal, { binary: installed.native }, exit);
  const raw = readFileSync(journal, "utf8");
  assert.ok(raw.endsWith("\n"));
  const rows = raw
    .slice(0, -1)
    .split("\n")
    .map((line) => asRecord(parseJson(line)));
  assert.deepEqual(
    rows.map((row) => row.event),
    ["initialized", "attempt", "returned", ...(exit === null ? [] : ["exit"])],
  );
  assert.deepEqual(
    Object.keys(rows[0]).sort(),
    ["schema", "pid", "event", "installRoot", "nativePath", "sha256", "node", "argv"].sort(),
  );
  assert.equal(rows[0].installRoot, installed.root);
  assert.equal(rows[0].node, child.argv[0]);
  assert.deepEqual(asArray(rows[0].argv), [child.argv[0], ...child.argv.slice(3)]);
  for (const row of rows) assert.equal(row.pid, child.pid);
  for (const row of rows.slice(1, 3)) {
    assert.deepEqual(
      Object.keys(row).sort(),
      ["schema", "pid", "event", "actualPath", "sha256", "expectedNative", "corsaPath"].sort(),
    );
    assert.equal(row.corsaPath, "");
    assert.equal(row.expectedNative, true);
  }
  if (exit !== null)
    assert.deepEqual(Object.keys(rows[3]).sort(), ["schema", "pid", "event", "code"].sort());
  return {
    capturedPid: child.pid,
    status: child.status,
    NodeExitCallbackObserved: exit !== null,
    nativeImage: installed.native,
    tokenBinding:
      "installer/provider receipt authenticates this same image SHA; hook has no token field",
  };
}
