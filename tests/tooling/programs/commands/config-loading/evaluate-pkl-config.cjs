#!/usr/bin/env node
const fs = require("node:fs");
if (process.argv.includes("--version")) process.exit(0);
const modulePath = process.argv[process.argv.length - 1];
const source = fs.readFileSync(modulePath, "utf-8");
if (!source.includes("file://") || !source.includes("/pkl/vize.pkl")) {
  process.stderr.write(source);
  process.exit(1);
}
process.stdout.write(
  JSON.stringify({
    compiler: { sourceMap: true },
    vite: { scanPatterns: ["app/**/*.vue"] },
  }),
);
