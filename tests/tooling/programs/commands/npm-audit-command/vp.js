const fs = require("node:fs");
const countFile = process.env.TEST_COUNTER_FILE;
const argsFile = process.env.TEST_ARGS_FILE;
const count = fs.existsSync(countFile) ? Number(fs.readFileSync(countFile, "utf8")) : 0;
fs.writeFileSync(countFile, String(count + 1));
fs.appendFileSync(argsFile, JSON.stringify(process.argv.slice(2)) + "\n");
if (process.env.TEST_AUDIT_MODE === "signal") process.kill(process.pid, "SIGTERM");
if (process.env.TEST_AUDIT_MODE === "eventual-success" && count > 0) {
  process.exit(0);
}
process.exit(Number(process.env.TEST_AUDIT_STATUS ?? "1"));
