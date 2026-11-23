const fs = require("node:fs");
fs.appendFileSync(process.env.SEMVER_HELPER_LOG, JSON.stringify(process.argv.slice(2)) + "\n");
if (process.env.SEMVER_TEST_FAIL) process.exit(1);
if (process.env.SEMVER_TEST_ROOT) process.stdout.write(process.env.SEMVER_TEST_ROOT + "\n");
