#!/usr/bin/env node
if (process.argv.includes("-p") && process.argv.includes("tsconfig.error.json")) {
  process.stdout.write("errors/Broken.vue TS2322\n");
  process.stdout.write("errors/JavaScriptConsumer.js TS2322\n");
  process.stdout.write("errors/JsxConsumer.jsx TS2322\n");
  process.stdout.write("src/Unused.vue TS6133\n");
  process.exit(1);
}
process.exit(0);
