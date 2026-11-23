import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
const { runtime, location, moduleUrl } = JSON.parse("@@PROGRAM_SETTINGS@@");
const { captureMembers } = await import(moduleUrl);
const dir = fs.readFileSync(location, "utf8");
const args = process.argv.slice(2);
const result = spawnSync(runtime, args, { env: process.env, maxBuffer: 64 * 1024 * 1024 });
const stem = path.join(dir, String(process.pid));
fs.writeFileSync(stem + ".stdout", result.stdout ?? "");
fs.writeFileSync(stem + ".stderr", result.stderr ?? "");
const projectIndex = args.indexOf("--project");
if (projectIndex < 0 || args.lastIndexOf("--project") !== projectIndex)
  throw Error("unknown project argv");
const configPath = path.resolve(args[projectIndex + 1]);
const configBytes = fs.readFileSync(configPath);
fs.writeFileSync(stem + ".tsconfig.json", configBytes);
const declared = captureMembers(configPath, JSON.parse(configBytes));
for (const [index, member] of declared.members.entries())
  fs.writeFileSync(stem + ".member-" + String(index) + ".txt", fs.readFileSync(member.file));
fs.writeFileSync(
  stem + ".json",
  JSON.stringify({
    runtime,
    args,
    cwd: process.cwd(),
    configPath,
    ...declared,
    status: result.status,
    signal: result.signal,
    error: result.error ? String(result.error) : null,
  }),
);
process.stdout.write(result.stdout ?? "");
process.stderr.write(result.stderr ?? "");
if (result.error) process.exitCode = 125;
else if (result.signal) process.kill(process.pid, result.signal);
else process.exitCode = result.status;
