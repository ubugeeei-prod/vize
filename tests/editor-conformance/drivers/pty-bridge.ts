// Run a terminal editor in a pseudo-terminal for the TS-45 Helix driver.
//
// Keystrokes arrive on this process's stdin (a pipe from the Node driver) and
// are written to the terminal; everything the editor draws is appended to the
// --screen file so a failing CI run can show what the editor displayed. The
// editor's startup terminal queries get the answers a plain VT220-class
// terminal gives (a DA1 reply and nothing else), so it never waits on them.
//
// usage: node pty-bridge.ts --cols N --rows N --screen FILE -- COMMAND [ARG ...]
import fs from "node:fs";
import { StringDecoder } from "node:string_decoder";
import { parseArgs } from "node:util";
import { spawn } from "zigpty";

// Primary device attributes: "VT220 with ANSI color". Editors send DA1 after
// their feature probes and treat the reply as "no more answers are coming".
const da1Query = Buffer.from("\x1b[c");
const da1Reply = "\x1b[?62;22c";

function count(haystack: Buffer, needle: Buffer): number {
  let total = 0;
  for (
    let at = haystack.indexOf(needle);
    at !== -1;
    at = haystack.indexOf(needle, at + needle.length)
  )
    total += 1;
  return total;
}

function dimension(value: string | undefined, name: string): number {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0)
    throw new Error(`--${name} must be a positive integer`);
  return parsed;
}

const { values, positionals } = parseArgs({
  options: {
    cols: { type: "string" },
    rows: { type: "string" },
    screen: { type: "string" },
  },
  allowPositionals: true,
});
const cols = dimension(values.cols, "cols");
const rows = dimension(values.rows, "rows");
if (values.screen === undefined) throw new Error("--screen is required");
if (positionals.length === 0) throw new Error("missing COMMAND");

const env: Record<string, string> = {};
for (const [key, value] of Object.entries(process.env)) if (value !== undefined) env[key] = value;
// A shell is the session leader, as in a real terminal: when the editor itself
// led the session, its exit hung up the terminal and SIGHUP killed the language
// server it had just asked to `exit` before the server could. The shell
// outlives the editor by a grace period, the way a terminal stays open after
// the editor quits.
const terminal = spawn(
  "/bin/sh",
  ["-c", '"$@"; status=$?; sleep 5; exit "$status"', "sh", ...positionals],
  { cols, rows, env, cwd: process.cwd(), encoding: null },
);
const screen = fs.openSync(values.screen, "a");
let tail = Buffer.alloc(0);
terminal.onData((data) => {
  const chunk = typeof data === "string" ? Buffer.from(data) : data;
  fs.writeSync(screen, chunk);
  const window = Buffer.concat([tail, chunk]);
  for (let reply = count(window, da1Query); reply > 0; reply -= 1) terminal.write(da1Reply);
  tail = window.subarray(-(da1Query.length - 1));
});
const keys = new StringDecoder("utf8");
process.stdin.on("data", (chunk: Buffer) => terminal.write(keys.write(chunk)));
terminal.onExit(({ exitCode, signal }) => {
  fs.closeSync(screen);
  process.stdin.destroy();
  terminal.close();
  process.exitCode = signal ? 128 + signal : exitCode;
});
