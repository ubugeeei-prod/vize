const fs = require("node:fs");
const line = fs
  .readFileSync("/proc/self/status", "utf8")
  .split("\n")
  .find((row) => row.startsWith("Cpus_allowed_list:"));
process.stdout.write(line.split(":")[1].trim().split(/[,-]/)[0]);
