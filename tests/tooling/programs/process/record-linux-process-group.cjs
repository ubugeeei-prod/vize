const fs = require("node:fs");
process.on("SIGTERM", () => {});
const stat = fs.readFileSync("/proc/self/stat", "utf8");
const fields = stat
  .slice(stat.lastIndexOf(")") + 1)
  .trim()
  .split(/\s+/u);
fs.writeFileSync(
  process.argv[2],
  JSON.stringify({
    pid: process.pid,
    parentPid: process.ppid,
    group: Number(fields[2]),
    session: Number(fields[3]),
  }),
);
process.stdout.write("mutation-child-started\n");
process.stderr.write("mutation-child-evidence\n");
setInterval(() => {}, 1000);
