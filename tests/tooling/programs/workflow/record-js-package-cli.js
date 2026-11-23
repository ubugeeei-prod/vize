import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
const args = process.argv.slice(2);
const { calls, tsc } = JSON.parse("@@PROGRAM_SETTINGS@@");
fs.appendFileSync(calls, JSON.stringify({ args, cwd: process.cwd() }) + "\n");
if (tsc === undefined) process.exit(0);
if (args[0] === "exec" && args[1] === "napi") {
  const out = args[args.indexOf("--output-dir") + 1];
  fs.mkdirSync(out, { recursive: true });
  for (const name of ["index.js", "index.d.ts"])
    fs.copyFileSync(path.join(process.cwd(), name), path.join(out, name));
} else if (args[0] === "exec" && args[1] === "tsc") {
  process.exit(
    spawnSync(process.execPath, [tsc, ...args.slice(2)], { stdio: "inherit" }).status ?? 1,
  );
} else throw new Error("unexpected CLI");
