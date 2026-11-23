const { spawnSync } = require("node:child_process");
const [command, ...args] = process.argv.slice(2);
const translated = args.map((arg) =>
  arg
    .replace(/^\/etc\/apt\/apt\.conf\.d\//, process.env.PROBE_DIR + "/config/")
    .replace(/^\/var\/cache\/apt\//, process.env.PROBE_DIR + "/cache/"),
);
const result = spawnSync(command, translated, { stdio: "inherit" });
process.exit(result.status ?? 1);
