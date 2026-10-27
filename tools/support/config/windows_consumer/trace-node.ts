// Observe the actual Node child without reading stdin or writing to its stdout/stderr.
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { createHash } from "node:crypto";

const directory = process.env.VIZE_WINDOWS_CONFIG_TRACE;
if (directory) {
  mkdirSync(directory, { recursive: true });
  const script = process.argv[1];
  const file =
    script && script.endsWith(".mjs") && existsSync(script) ? readFileSync(script) : undefined;
  const initial = {
    pid: process.pid,
    ppid: process.ppid,
    node: process.version,
    execPath: process.execPath,
    cwd: process.cwd(),
    argv: [...process.argv],
    execArgv: [...process.execArgv],
    script: file
      ? {
          path: script,
          bytes: file.length,
          sha256: createHash("sha256").update(file).digest("hex"),
          wholeText: file.toString("utf8"),
        }
      : null,
  };
  writeFileSync(
    join(directory, String(process.pid) + "-start.json"),
    JSON.stringify(initial, null, 2) + "\n",
  );
  process.once("exit", (code) => {
    writeFileSync(
      join(directory, String(process.pid) + "-exit.json"),
      JSON.stringify(
        {
          pid: process.pid,
          node: process.version,
          execPath: process.execPath,
          argv: [...process.argv],
          code,
        },
        null,
        2,
      ) + "\n",
    );
  });
}
