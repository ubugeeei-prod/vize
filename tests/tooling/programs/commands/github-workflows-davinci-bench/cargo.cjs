#!/usr/bin/env node
const fs = require("node:fs");
const input = process.env.VIZE_SCRIPT_QUERY_INPUTS;
const manifest = input && fs.readFileSync(input, "utf8").split("\0");
fs.appendFileSync(
  process.env.CARGO_INVOCATIONS,
  JSON.stringify({ args: process.argv.slice(2), manifest }) + "\n",
);
