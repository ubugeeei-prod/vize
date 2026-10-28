#!/usr/bin/env node
/** Test-only exec carrier. setpriv installs PDEATHSIG before this entry runs. */
import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { join } from "node:path";

const backend = process.env.VIZE_ORIGIN_API_BACKEND;
const output = process.env.VIZE_ORIGIN_API_CAPTURE;
const parent = process.env.VIZE_ORIGIN_API_PARENT_PID;
assert.ok(backend !== undefined && output !== undefined && parent !== undefined);
assert.match(parent, /^[1-9][0-9]*$/u);
const expectedParent = Number(parent);
assert.ok(Number.isSafeInteger(expectedParent));
assert.equal(process.ppid, expectedParent, "native owner disappeared during guard setup");
const argv = [backend, ...process.argv.slice(2)];
writeFileSync(
  join(output, "actual-launch.json"),
  JSON.stringify({
    pid: process.pid,
    ppid: process.ppid,
    expectedParent,
    argv,
    transport: "direct-stdio",
    parentDeathGuard: "setpriv --pdeathsig KILL",
  }),
  { flag: "wx" },
);
assert.equal(typeof process.execve, "function");
if (process.execve === undefined) throw new Error("Node must expose execve");
process.execve(backend, argv, process.env);
throw new Error("execve unexpectedly returned");
