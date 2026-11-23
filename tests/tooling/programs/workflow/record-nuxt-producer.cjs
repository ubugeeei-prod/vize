const { writeFileSync } = require("node:fs");
writeFileSync(
  process.env.PROBE_LOG,
  JSON.stringify({ producer: process.execPath, args: process.argv.slice(2) }),
);
