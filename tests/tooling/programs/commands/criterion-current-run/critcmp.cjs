#!/usr/bin/env node
const fs = require("node:fs");
const path = require("node:path");
const args = process.argv.slice(2);
if (args.includes("--export")) {
  const home = path.join(args[args.indexOf("--target-dir") + 1], "criterion");
  const benchmarks = Object.fromEntries(
    fs.readdirSync(home).map((file) => [
      file.replace(/\.json$/, ""),
      {
        criterion_estimates_v1: {
          median: { point_estimate: JSON.parse(fs.readFileSync(path.join(home, file), "utf8")) },
        },
      },
    ]),
  );
  console.log(JSON.stringify({ name: args[args.indexOf("--export") + 1], benchmarks }));
} else {
  const exports = args.slice(-2).map((file) => JSON.parse(fs.readFileSync(file, "utf8")));
  const names = new Set(exports.flatMap((value) => Object.keys(value.benchmarks)));
  console.log(
    "group base head\n----- ---- ----\n" + [...names].map((name) => name + " 1.00 1.00").join("\n"),
  );
}
