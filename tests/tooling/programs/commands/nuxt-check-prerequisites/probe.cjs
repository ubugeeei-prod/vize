#!/usr/bin/env node
const fs = require("node:fs");
const { execFileSync } = require("node:child_process");
function identity() {
  return { execPath: process.execPath, version: process.version };
}
if (process.argv[2] === "--identity") {
  console.log(JSON.stringify(identity()));
} else {
  fs.writeFileSync(
    process.argv[2],
    JSON.stringify({
      parent: identity(),
      child: JSON.parse(execFileSync("node", [__filename, "--identity"], { encoding: "utf8" })),
    }),
  );
}
