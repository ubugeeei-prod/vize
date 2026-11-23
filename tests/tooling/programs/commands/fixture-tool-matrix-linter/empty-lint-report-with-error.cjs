#!/usr/bin/env node
process.stdout.write("[]");
process.stderr.write(
  'No .vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, .tsx, or .tsrx files found matching patterns: ["**/*.vue"]\n',
);
process.exit(2);
