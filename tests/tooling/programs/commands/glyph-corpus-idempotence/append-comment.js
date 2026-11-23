#!/usr/bin/env node
import fs from "node:fs";
const file = "src/App.vue";
fs.appendFileSync(file, "<!-- drift -->\n");
process.stderr.write(
  "Found 1 file(s)\nReformatted: src/App.vue\n\nFormatted 1 file(s)\n  1 file(s) reformatted\n",
);
