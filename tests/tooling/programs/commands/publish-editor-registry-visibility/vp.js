const fs = require("node:fs");
const args = process.argv.slice(2);
const calls = fs.existsSync(process.env.CALLS_PATH)
  ? JSON.parse(fs.readFileSync(process.env.CALLS_PATH, "utf8"))
  : [];
calls.push(args);
fs.writeFileSync(process.env.CALLS_PATH, JSON.stringify(calls));
if (args[4] === "show") process.exit(1);
if (args[4] === "create-namespace") process.exit(0);
if (args[4] === "publish") process.exit(Number(process.env.PUBLISH_EXIT));
process.exit(1);
