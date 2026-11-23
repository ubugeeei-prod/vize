const { spawnSync } = require("node:child_process");
const result = spawnSync(process.argv[2], process.argv.slice(3), { stdio: "inherit" });
process.exit(result.status ?? 1);
