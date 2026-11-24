const fs = require("node:fs");
fs.appendFileSync(process.env.FAKE_GH_LOG, JSON.stringify(process.argv.slice(2)) + "\n");
