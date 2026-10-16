import { appendFileSync, readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { threadId } from "node:worker_threads";

const directory = process.env.VIZE_MUSEA_HTTP_DIAGNOSTICS;
if (directory) {
  const file = `${directory}/native-${process.pid}-${threadId}.jsonl`;
  const record = (event, details = {}) =>
    appendFileSync(file, `${JSON.stringify({ event, pid: process.pid, threadId, ...details })}\n`);
  record("started", { node: process.version, executable: process.execPath });
  const original = process.dlopen;
  process.dlopen = function (module, filename, ...options) {
    record("dlopen:start", {
      filename,
      sha256: createHash("sha256").update(readFileSync(filename)).digest("hex"),
    });
    const result = Reflect.apply(original, this, [module, filename, ...options]);
    record("dlopen:finished", { filename });
    return result;
  };
  process.on("beforeExit", (code) => record("beforeExit", { code }));
  process.on("exit", (code) => record("exit", { code }));
}
