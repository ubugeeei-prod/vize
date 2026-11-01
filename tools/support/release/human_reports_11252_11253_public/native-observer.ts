// Companion-only observer. It preserves the native loader's this, arguments,
// return value and exception. Original recipe commands run without this preload.
import { createHash } from "node:crypto";
import { closeSync, openSync, readFileSync, realpathSync, writeSync } from "node:fs";
import { createRequire, syncBuiltinESMExports } from "node:module";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import type { NativeEvent } from "./model.ts";
import { observeLoader, type Loader } from "./loader-law.ts";
const require = createRequire(import.meta.url);
const config: unknown = JSON.parse(process.env.VIZE_HUMAN_REPORT_NATIVE_OBSERVER ?? "null");
if (!config || typeof config !== "object")
  throw new Error("Missing companion observer configuration");
const c = config as Record<string, unknown>;
if (
  c.schema !== "vize.humanReportNativeObserver.v1" ||
  typeof c.nativePath !== "string" ||
  typeof c.nativeSha256 !== "string" ||
  typeof c.journalRoot !== "string" ||
  typeof c.commandId !== "string" ||
  !/^[a-z][a-z0-9-]*$/u.test(c.commandId) ||
  typeof c.rootPid !== "number" ||
  !Number.isSafeInteger(c.rootPid) ||
  c.rootPid <= 0
)
  throw new Error("Incomplete finite companion observer configuration");
const expectedPath = realpathSync(c.nativePath),
  expectedSha = c.nativeSha256;
const sha = (filename: string) => createHash("sha256").update(readFileSync(filename)).digest("hex");
if (sha(expectedPath) !== expectedSha)
  throw new Error("Installed native image changed before companion");
const ancestry: NonNullable<NativeEvent["ancestry"]> = [
  { pid: process.pid, parentPid: process.ppid, source: "process.ppid" },
];
let parent = process.ppid;
for (let count = 0; count < 16 && parent !== c.rootPid && parent > 1; count++) {
  // These are observed process metadata edges, not syscall clocks or an
  // assertion that a particular checker executable/backend was selected.
  const row = spawnSync("ps", ["-p", String(parent), "-o", "ppid="], {
    encoding: "utf8",
    timeout: 2000,
    maxBuffer: 4096,
  });
  if (row.error || row.status !== 0 || !/^\s*[1-9]\d*\s*$/u.test(row.stdout)) break;
  const next = Number(row.stdout.trim());
  ancestry.push({ pid: parent, parentPid: next, source: "ps" });
  parent = next;
}
const journal = openSync(join(realpathSync(c.journalRoot), `${process.pid}.jsonl`), "wx", 0o600);
const base = {
  commandId: c.commandId,
  pid: process.pid,
  parentPid: process.ppid,
  argv: process.argv,
  nativePath: expectedPath,
  sha256: expectedSha,
  ancestry,
};
const emit = (event: NativeEvent["event"]) =>
  writeSync(journal, JSON.stringify({ ...base, event }) + "\n");
emit("initialized");
process.on("exit", () => {
  emit("exit");
  closeSync(journal);
});
const Module = require("node:module") as {
  _extensions: Record<string, Loader>;
};
const original = Module._extensions[".node"];
if (typeof original !== "function") throw new Error("Original Node native loader is unavailable");
Module._extensions[".node"] = observeLoader(
  original,
  (args) => {
    if (typeof args[1] !== "string") return false;
    const actual = realpathSync(args[1]);
    if (actual !== expectedPath) {
      if (/(?:vize[-_](?:native|vitrine)|@vizejs[/\\]native)/u.test(actual))
        throw new Error("Companion Vize native loader escaped authenticated installed image");
      return false;
    }
    return true;
  },
  emit,
  () => {
    if (sha(expectedPath) !== expectedSha)
      throw new Error("Native payload changed around actual load");
  },
);
syncBuiltinESMExports();
