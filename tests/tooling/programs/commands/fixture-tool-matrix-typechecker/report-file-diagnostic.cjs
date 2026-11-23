#!/usr/bin/env node
process.stdout.write(
  JSON.stringify({
    files: [{ file: "src/App.vue", diagnostics: [] }],
    programs: [{ root: ".", files: ["src/App.vue"] }],
    errorCount: 0,
    warningCount: 0,
    fileCount: 1,
  }),
);
