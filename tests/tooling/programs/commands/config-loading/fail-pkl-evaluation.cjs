#!/usr/bin/env node
if (process.argv.includes("--version")) process.exit(0);
process.stderr.write("synthetic pkl failure");
process.exit(1);
