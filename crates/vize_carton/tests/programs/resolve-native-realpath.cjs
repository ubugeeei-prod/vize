const { realpathSync } = require("node:fs");

process.stdout.write(realpathSync.native(process.argv[2]));
