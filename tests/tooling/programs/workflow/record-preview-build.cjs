const { writeFileSync } = require("node:fs");
writeFileSync(
  process.env.PROBE_OUTPUT,
  JSON.stringify({
    args: process.argv.slice(2),
    cwd: process.cwd(),
    mode: process.env.NODE_ENV,
    fixture: process.env.NUXT_TEST_FIXTURES,
  }),
);
