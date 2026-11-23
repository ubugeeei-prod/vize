const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const git = spawnSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" });
fs.writeFileSync(
  process.env.FIXTURE_INSTALL_PROBE,
  JSON.stringify({
    argv: process.argv.slice(2),
    env: {
      GIT_CEILING_DIRECTORIES: process.env.GIT_CEILING_DIRECTORIES,
      HUSKY: process.env.HUSKY,
      SKIP_INSTALL_SIMPLE_GIT_HOOKS: process.env.SKIP_INSTALL_SIMPLE_GIT_HOOKS,
    },
    gitStatus: git.status,
    gitTopLevel: git.stdout.trim(),
  }),
);
