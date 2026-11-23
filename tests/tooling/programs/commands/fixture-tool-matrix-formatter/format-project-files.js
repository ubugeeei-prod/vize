#!/usr/bin/env node
import fs from "node:fs";
fs.writeFileSync("src/App.vue", "<template><main /></template>\n");
process.stderr.write(
  "Found 1 file(s)\nWould reformat: src/App.vue\n\nChecked 1 file(s)\n  1 file(s) would be reformatted\n",
);
process.exit(1);
