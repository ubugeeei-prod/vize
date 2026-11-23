const { spawn } = require("node:child_process");
const grandchild = spawn(
  process.execPath,
  [require("node:path").join(__dirname, "hold-config-process.cjs")],
  { stdio: "inherit" },
);
require("node:fs").writeFileSync(process.argv[2], String(grandchild.pid));
process.stderr.write("child-specificreason\n");
process.stdout.write('{"value":2}\n', () => process.exit(3));
