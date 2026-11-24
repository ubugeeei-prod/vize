import fs from "node:fs";
import path from "node:path";
import { spawn } from "node:child_process";
const { directory, host } = JSON.parse("@@PROGRAM_SETTINGS@@");
const prefix = path.join(directory, String(process.pid));
const args = process.argv.slice(2);
fs.writeFileSync(
  prefix + ".argv.json",
  JSON.stringify({ argv: [host, ...args], cwd: process.cwd(), parent: process.ppid }),
);
for (const kind of ["stdin", "stdout", "stderr"])
  fs.writeFileSync(prefix + "." + kind + ".raw", "");
const projectAt = args.indexOf("--project");
if (projectAt >= 0) {
  const config = args[projectAt + 1];
  const root = path.dirname(config),
    target = prefix + ".program";
  fs.mkdirSync(target);
  const copy = (from, to) => {
    for (const entry of fs.readdirSync(from, { withFileTypes: true })) {
      if (entry.name === "node_modules" || entry.name.startsWith(".vize-buildinfo")) continue;
      const source = path.join(from, entry.name),
        dest = path.join(to, entry.name);
      if (entry.isDirectory()) {
        fs.mkdirSync(dest, { recursive: true });
        copy(source, dest);
      } else if (entry.isFile() && /\.(?:ts|tsx|json)$/.test(entry.name))
        fs.copyFileSync(source, dest);
    }
  };
  copy(root, target);
}
const child = spawn(host, args, { stdio: ["pipe", "pipe", "pipe"], env: process.env });
child.stdout.on("data", (bytes) => {
  fs.appendFileSync(prefix + ".stdout.raw", bytes);
  process.stdout.write(bytes);
});
child.stderr.on("data", (bytes) => {
  fs.appendFileSync(prefix + ".stderr.raw", bytes);
  process.stderr.write(bytes);
});
process.stdin.on("data", (bytes) => {
  fs.appendFileSync(prefix + ".stdin.raw", bytes);
  child.stdin.write(bytes);
});
process.stdin.on("end", () => child.stdin.end());
child.on("error", (error) => {
  fs.writeFileSync(prefix + ".spawn-error.json", JSON.stringify({ message: error.message }));
  process.exitCode = 1;
});
child.on("close", (code, signal) => {
  fs.writeFileSync(prefix + ".exit.json", JSON.stringify({ code, signal }));
  process.stdin.pause();
  if (signal) process.kill(process.pid, signal);
  else process.exitCode = code ?? 1;
});
