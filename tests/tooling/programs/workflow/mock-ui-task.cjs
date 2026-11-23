const fs = require("node:fs");
const first = process.argv[4] === "./npm/native";
const snapshot = fs.existsSync(process.env.PHASE_OUTCOME)
  ? JSON.parse(fs.readFileSync(process.env.PHASE_OUTCOME, "utf8"))
  : null;
fs.appendFileSync(
  "calls.jsonl",
  JSON.stringify({ argv: process.argv.slice(2), cwd: process.cwd(), env: process.env, snapshot }) +
    "\n",
);
fs.writeSync(1, first ? "build stdout\n" : "ui stdout\n");
fs.writeSync(2, first ? "build stderr\n" : "ui stderr\n");
const result = first ? process.env.UI_BUILD_OUTCOME : process.env.UI_CHECK_OUTCOME;
if (result.startsWith("SIG")) process.kill(process.pid, result);
else process.exit(Number(result));
