#!/usr/bin/env node
require("node:fs").writeFileSync(process.env.ARGS_FILE, JSON.stringify(process.argv.slice(2)));
process.exit(37);
