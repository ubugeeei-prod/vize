const fs = require("node:fs");
fs.appendFileSync(
  process.env.VIZE_SMOKE_RUNTIME_TEST_LOG,
  JSON.stringify({ tool: "tsc", args: process.argv.slice(2), cwd: process.cwd() }) + "\n",
);
