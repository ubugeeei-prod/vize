const fs = require("node:fs");
const { logPath, name, exitCodes, stdout, hook } = JSON.parse("@@PROGRAM_SETTINGS@@");
const previous = fs.existsSync(logPath)
  ? fs
      .readFileSync(logPath, "utf8")
      .split("\n")
      .filter((line) => line && JSON.parse(line).name === name).length
  : 0;
fs.appendFileSync(
  logPath,
  JSON.stringify({ name, argv: process.argv.slice(2), cwd: process.cwd(), env: process.env }) +
    "\n",
);
if (hook) require(hook.program)(...hook.args);
process.stdout.write(stdout);
process.exit(exitCodes[Math.min(previous, exitCodes.length - 1)]);
