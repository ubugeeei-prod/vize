const { writeFileSync } = require("node:fs");
const [mode, file] = process.argv.slice(2);
if (mode === "change-before-load") writeFileSync(file, "tampered");
if (mode === "observe-failure") {
  try {
    require(file);
  } catch (error) {
    process.stdout.write(error.message);
    process.exitCode = 42;
  }
} else if (file) require(file);
