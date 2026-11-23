const { readFileSync } = require("node:fs");

process.stdout.write(readFileSync(__filename + ".stdout"));
