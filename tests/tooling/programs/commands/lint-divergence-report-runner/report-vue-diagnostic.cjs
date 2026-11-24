#!/usr/bin/env node
if (process.argv[2] !== "lint") process.exit(2);
process.stdout.write(JSON.stringify([{ file: "App.vue", messages: [] }]));
