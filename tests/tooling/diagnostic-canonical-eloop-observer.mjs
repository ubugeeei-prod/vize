// Diagnostic preload for one explicitly filtered child. No retries or lookup changes.
import fs from "node:fs";
import { syncBuiltinESMExports } from "node:module";
import { join } from "node:path";

const directory = process.env.VIZE_ELOOP_OBSERVER_DIRECTORY;
if (!directory) throw new Error("Missing isolated ELOOP observation directory");
const rows = [];
const observationErrors = [];
const selected = (value) =>
  typeof value === "string" && value.includes("canonical-current-native-walk-");
const errorFields = (error) => ({
  name: error?.name,
  message: error?.message,
  code: error?.code,
  errno: error?.errno,
  syscall: error?.syscall,
  path: error?.path,
  stack: error?.stack,
});
const record = (method, args, result, error) => {
  if (!selected(args[0])) return;
  try {
    const row = { sequence: rows.length, method, path: args[0] };
    if (error) row.error = errorFields(error);
    else if (method === "statSync" || method === "lstatSync")
      row.result = result && {
        isDirectory: result.isDirectory(),
        isSymbolicLink: result.isSymbolicLink(),
        mode: result.mode,
        dev: result.dev,
        ino: result.ino,
        size: result.size,
        mtimeMs: result.mtimeMs,
        ctimeMs: result.ctimeMs,
      };
    else if (Buffer.isBuffer(result)) row.result = { base64: result.toString("base64") };
    else if (method === "readdirSync") row.result = [...result];
    else row.result = result;
    rows.push(row);
  } catch (failure) {
    // Observer failures do not replace a filesystem return value or exception.
    observationErrors.push(errorFields(failure));
  }
};

for (const method of ["statSync", "lstatSync", "readlinkSync", "realpathSync", "readdirSync"]) {
  const original = fs[method];
  fs[method] = new Proxy(original, {
    apply(target, receiver, args) {
      let value;
      try {
        value = Reflect.apply(target, receiver, args);
      } catch (error) {
        record(method, args, undefined, error);
        throw error; // The identical native exception, with no retry.
      }
      record(method, args, value, undefined);
      return value; // The identical object or primitive.
    },
  });
}
syncBuiltinESMExports();

process.on("exit", (code) => {
  const permittedEnvironment = {};
  for (const name of ["LD_PRELOAD", "LD_AUDIT", "NODE_OPTIONS", "RUSTUP_TOOLCHAIN"])
    if (process.env[name] !== undefined) permittedEnvironment[name] = process.env[name];
  fs.writeFileSync(
    join(directory, `observer.${process.pid}.json`),
    JSON.stringify(
      {
        schema: "vize.diagnostic-canonical-eloop-observation",
        sourceSha: process.env.VIZE_ELOOP_SOURCE_SHA,
        role: process.env.VIZE_ELOOP_ROLE,
        pid: process.pid,
        ppid: process.ppid,
        node: process.version,
        executable: process.execPath,
        argv: process.argv,
        execArgv: process.execArgv,
        cwd: process.cwd(),
        exitCode: code,
        permittedEnvironment,
        processMaps: fs.readFileSync("/proc/self/maps", "utf8"),
        observationErrors,
        rows,
      },
      (_, value) => (typeof value === "bigint" ? value.toString() : value),
      2,
    ) + "\n",
  );
});
