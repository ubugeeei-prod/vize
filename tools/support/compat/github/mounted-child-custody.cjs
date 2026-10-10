// Observational preload for a diagnostic run; it does not consume stdin or write
// either protocol stream, install signal handlers, or change the runner's exit.
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");

const directory = process.env.VIZE_MOUNTED_CHILD_CUSTODY;
if (directory) {
  fs.mkdirSync(path.join(directory, "images"), { recursive: true });
  const image = (filename) => {
    const bytes = fs.readFileSync(filename);
    const sha256 = crypto.createHash("sha256").update(bytes).digest("hex");
    const retained = path.join(directory, "images", sha256);
    try {
      fs.copyFileSync(filename, retained, fs.constants.COPYFILE_EXCL);
    } catch (error) {
      if (error.code !== "EEXIST") throw error;
    }
    return { filename, sha256, bytes: bytes.length, retained };
  };
  const executable = image(process.execPath);
  let sequence = 0;
  const capture = (event, exitCode = null) => {
    const maps = process.platform === "linux" ? fs.readFileSync("/proc/self/maps", "utf8") : "";
    const addons = [...new Set(maps.split("\n").map((line) => line.trim().split(/\s+/u).at(-1)))]
      .filter((filename) => filename?.endsWith(".node"))
      .map(image);
    const stem = path.join(directory, `${process.pid}-${sequence++}-${event}`);
    fs.writeFileSync(`${stem}.maps`, maps);
    fs.writeFileSync(
      `${stem}.json`,
      `${JSON.stringify({
        event,
        exitCode,
        pid: process.pid,
        ppid: process.ppid,
        executable,
        version: process.version,
        versions: process.versions,
        argv: process.argv,
        cwd: process.cwd(),
        sourceSha: process.env.VIZE_MOUNTED_CHILD_SOURCE_SHA,
        role: process.env.VIZE_MOUNTED_CHILD_ROLE ?? "original-mounted-tests",
        addons,
      })}\n`,
    );
  };
  capture("start");
  process.on("beforeExit", (code) => capture("beforeExit", code));
  process.on("exit", (code) => capture("exit", code));
}
