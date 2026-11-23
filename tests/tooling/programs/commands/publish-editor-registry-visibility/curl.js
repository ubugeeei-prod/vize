const fs = require("node:fs");
const args = process.argv.slice(2);
const calls = fs.existsSync(process.env.CALLS_PATH)
  ? JSON.parse(fs.readFileSync(process.env.CALLS_PATH, "utf8"))
  : [];
calls.push(["curl", ...args]);
fs.writeFileSync(process.env.CALLS_PATH, JSON.stringify(calls));
process.exit(22); // curl --fail on HTTP 404
