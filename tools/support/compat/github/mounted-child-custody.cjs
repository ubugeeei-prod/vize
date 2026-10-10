// Replay pathname byte snapshots and process maps; historical crash images are unknown.
// Observational preload for a diagnostic run; it does not consume stdin or write
// either protocol stream, install signal handlers, or change the runner's exit.
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const assert = require("node:assert/strict");

const directory = process.env.VIZE_MOUNTED_CHILD_CUSTODY;
if (directory) {
  fs.mkdirSync(path.join(directory, "images"), { recursive: true });
  const mirror = (filename, retained) => {
    const target = path.join(directory, "sysroot", filename.slice(1));
    fs.mkdirSync(path.dirname(target), { recursive: true });
    const relativeImage = path.relative(path.dirname(target), retained);
    try {
      fs.symlinkSync(relativeImage, target);
    } catch (error) {
      if (error.code !== "EEXIST") throw error;
      assert.equal(fs.readlinkSync(target), relativeImage, "Replay pathname snapshot changed");
    }
  };
  const image = (filename) => {
    const bytes = fs.readFileSync(filename);
    const sha256 = crypto.createHash("sha256").update(bytes).digest("hex");
    const retained = path.join(directory, "images", sha256);
    if (!fs.existsSync(retained)) {
      const temporary = `${retained}.${process.pid}.${process.hrtime.bigint()}.tmp`;
      fs.writeFileSync(temporary, bytes, { flag: "wx" });
      try {
        fs.linkSync(temporary, retained);
      } catch (error) {
        if (error.code !== "EEXIST") throw error;
      } finally {
        fs.unlinkSync(temporary);
      }
    }
    const physical = fs.realpathSync(filename);
    const aliases = new Set([filename]);
    let alias = filename;
    while (/\.so\.[0-9.]+$/.test(alias)) {
      alias = alias.slice(0, alias.lastIndexOf("."));
      if (fs.existsSync(alias) && fs.realpathSync(alias) === physical) aliases.add(alias);
    }
    for (const candidate of [...aliases]) {
      if (candidate.startsWith("/usr/lib/")) {
        const legacy = candidate.slice(4);
        if (fs.existsSync(legacy) && fs.realpathSync(legacy) === physical) aliases.add(legacy);
      }
    }
    if (path.basename(filename).startsWith("ld-linux-")) {
      for (const candidate of ["/lib64/", "/usr/lib64/"].map(
        (prefix) => prefix + path.basename(filename),
      ))
        if (fs.existsSync(candidate) && fs.realpathSync(candidate) === physical)
          aliases.add(candidate);
    }
    for (const candidate of aliases) mirror(candidate, retained);
    return { filename, sha256, bytes: bytes.length, retained, aliases: [...aliases] };
  };
  const executable = image(process.execPath);
  let sequence = 0;
  const capture = (event, exitCode = null) => {
    const maps = process.platform === "linux" ? fs.readFileSync("/proc/self/maps", "utf8") : "";
    const libraries = [...new Set(maps.split("\n").map((line) => line.trim().split(/\s+/u).at(-1)))]
      .filter(
        (filename) => filename?.startsWith("/") && /(?:\.node|\.so(?:\.[0-9]+)*)$/.test(filename),
      )
      .map(image);
    const addons = libraries.filter((library) => library.filename.endsWith(".node"));
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
        libraries,
      })}\n`,
    );
  };
  capture("start");
  process.on("beforeExit", (code) => capture("beforeExit", code));
  process.on("exit", (code) => capture("exit", code));
}
