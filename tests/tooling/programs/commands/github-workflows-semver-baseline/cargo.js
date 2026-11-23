const fs = require("node:fs");
fs.appendFileSync(process.env.SEMVER_TEST_LOG, JSON.stringify(process.argv.slice(2)) + "\n");
