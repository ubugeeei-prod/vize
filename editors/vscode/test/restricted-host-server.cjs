#!/usr/bin/env node
const fs = require("node:fs");

fs.appendFileSync(
  process.env.VIZE_TEST_FORBIDDEN_SERVER_MARKER,
  JSON.stringify({ argv: process.argv.slice(2) }) + "\n",
);
if (process.argv.includes("--version")) console.log("vize 9.9.9");
