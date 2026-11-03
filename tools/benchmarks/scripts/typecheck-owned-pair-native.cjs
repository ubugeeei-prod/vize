/** Record the native image actually loaded by every Node member of this invocation. */
const fs = require("node:fs");
const Module = require("node:module");
const path = require("node:path");
const original = Module._extensions[".node"];
Module._extensions[".node"] = function observeNative(module, filename) {
  original(module, filename);
  const token = module.exports?.getNativeBuildToken;
  if (typeof token !== "function" && !path.basename(filename).startsWith("vize-vitrine.")) return;
  const directory = process.env.VIZE_PAIR_NATIVE_RECEIPTS;
  if (!directory) throw new Error("owned pair native custody directory missing");
  fs.appendFileSync(
    path.join(directory, `${process.pid}.jsonl`),
    JSON.stringify({
      image: fs.realpathSync(filename),
      token: typeof token === "function" ? token() : null,
      kind: "native",
      pid: process.pid,
    }) + "\n",
  );
};
