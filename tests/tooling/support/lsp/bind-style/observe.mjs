// Passive stdio custody for the unchanged original client and authored session.
// Forward every spawn/write argument, receiver and return value unchanged.
import childProcess from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { syncBuiltinESMExports } from "node:module";

const directory = process.env.VIZE_BIND_STYLE_CAPTURE_ROOT;
const binary = process.env.VIZE_BIND_STYLE_BINARY;
let restore = () => {};
export function stopObservation() {
  restore();
}
if (directory && binary) {
  const originalSpawn = childProcess.spawn;
  childProcess.spawn = function (...args) {
    const child = Reflect.apply(originalSpawn, this, args);
    let selected = false;
    try {
      selected = fs.realpathSync(args[0]) === fs.realpathSync(binary);
    } catch {
      /* Other processes retain their exact original route. */
    }
    if (!selected) return child;
    const output = path.join(directory, `process-${child.pid}`);
    fs.mkdirSync(output, { recursive: true });
    const streams = { client: [], server: [], stderr: [] };
    const observed = { client: 0, server: 0, stderr: 0 };
    const record = {
      pid: child.pid,
      binary: fs.realpathSync(binary),
      arguments: args[1],
      cwd: args[2]?.cwd ?? process.cwd(),
      stdio: args[2]?.stdio ?? "pipe",
      state: "running",
      error: null,
      exitCode: null,
      signal: null,
      streams: observed,
    };
    const save = () => {
      for (const stream of Object.keys(streams))
        fs.writeFileSync(path.join(output, `${stream}.bin`), Buffer.concat(streams[stream]));
      fs.writeFileSync(path.join(output, "process.json"), `${JSON.stringify(record, null, 2)}\n`);
    };
    const append = (stream, value, encoding) => {
      const bytes = typeof value === "string" ? Buffer.from(value, encoding) : Buffer.from(value);
      observed[stream] += bytes.length;
      // Overflow is explicitly refused by the oracle; it cannot count as complete custody.
      if (observed[stream] <= 16 * 1024 * 1024) streams[stream].push(bytes);
      save();
    };
    if (child.stdin) {
      const write = child.stdin.write;
      child.stdin.write = function (...values) {
        append("client", values[0], typeof values[1] === "string" ? values[1] : undefined);
        return Reflect.apply(write, this, values);
      };
    }
    child.stdout?.on("data", (bytes) => append("server", bytes));
    child.stderr?.on("data", (bytes) => append("stderr", bytes));
    child.on("error", (error) => {
      record.error = String(error);
      save();
    });
    child.on("close", (code, signal) => {
      record.state = "closed";
      record.exitCode = code;
      record.signal = signal;
      save();
    });
    save();
    return child;
  };
  const ownedSpawn = childProcess.spawn;
  restore = () => {
    if (childProcess.spawn === ownedSpawn) {
      childProcess.spawn = originalSpawn;
      syncBuiltinESMExports();
    }
  };
  syncBuiltinESMExports();
}
