#!/usr/bin/env node
const fs = require("node:fs");
const path = require("node:path");
const home = process.env.CRITERION_HOME;
if (home !== path.join(process.env.CARGO_TARGET_DIR, "criterion")) process.exit(2);
fs.mkdirSync(home, { recursive: true });
if (!process.env.VIZE_TEST_EMPTY_CRITERION) {
  fs.writeFileSync(path.join(home, "current-benchmark.json"), "100");
}
