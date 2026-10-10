// Supplement the unchanged pathname collector; never claim historical images.
require("./mounted-child-custody.cjs");
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const assert = require("node:assert/strict");

const directory = process.env.VIZE_MOUNTED_CHILD_CUSTODY;
const packet = process.env.VIZE_SUPPLIED_SLOTS_PACKET;
assert(directory && packet && process.platform === "linux", "Missing Linux PID custody");
const statValue = (stat) => ({
  dev: stat.dev.toString(),
  ino: stat.ino.toString(),
  size: stat.size.toString(),
  mtimeNs: stat.mtimeNs.toString(),
  ctimeNs: stat.ctimeNs.toString(),
});
const device = (dev) => [
  ((dev >> 8n) & 0xfffn) | ((dev >> 32n) & 0xfffff000n),
  (dev & 0xffn) | ((dev >> 12n) & 0xffffff00n),
];
const cached = new Map();
const snapshot = (filename) => {
  const fd = fs.openSync(filename, "r");
  try {
    const before = fs.fstatSync(fd, { bigint: true });
    const stat = statValue(before);
    const key = JSON.stringify(stat);
    let retained = cached.get(key);
    if (!retained) {
      const bytes = fs.readFileSync(fd);
      const sha256 = crypto.createHash("sha256").update(bytes).digest("hex");
      const image = path.join(directory, "images", sha256);
      try {
        fs.writeFileSync(image, bytes, { flag: "wx" });
      } catch (error) {
        if (error.code !== "EEXIST") throw error;
      }
      retained = { sha256, retained: image, bytes: bytes.length };
      cached.set(key, retained);
    }
    assert.deepEqual(
      statValue(fs.fstatSync(fd, { bigint: true })),
      stat,
      "Image changed during capture",
    );
    return { filename, realpath: fs.realpathSync(filename), stat, ...retained };
  } finally {
    fs.closeSync(fd);
  }
};
const startTicks = () => {
  const text = fs.readFileSync("/proc/self/stat", "utf8");
  return text
    .slice(text.lastIndexOf(") ") + 2)
    .trim()
    .split(/\s+/u)[19];
};
const mirror = (image) => {
  const target = path.join(directory, "sysroot", image.filename.slice(1));
  fs.mkdirSync(path.dirname(target), { recursive: true });
  const relative = path.relative(path.dirname(target), image.retained);
  try {
    fs.symlinkSync(relative, target);
  } catch (error) {
    if (error.code !== "EEXIST") throw error;
    assert.equal(fs.readlinkSync(target), relative, "Mapped pathname snapshot changed");
  }
};
let sequence = 0;
const capture = (event) => {
  const executable = { procLink: "/proc/self/exe", ...snapshot("/proc/self/exe") };
  const maps = fs.readFileSync("/proc/self/maps", "utf8");
  const mappings = maps
    .trim()
    .split("\n")
    .map((line) => {
      const match = /^(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s*(.*)$/u.exec(line);
      assert(match, "Malformed actual process mapping");
      return {
        address: match[1],
        permissions: match[2],
        offset: match[3],
        device: match[4],
        inode: match[5],
        filename: match[6],
      };
    });
  const names = [
    ...new Set(
      mappings
        .filter((map) => map.permissions.includes("x") && map.filename.startsWith("/"))
        .map((map) => map.filename),
    ),
  ];
  const images = names.map((filename) => {
    const image = snapshot(filename);
    const [major, minor] = device(BigInt(image.stat.dev));
    const mappedIdentityEqual = mappings
      .filter((map) => map.filename === filename)
      .every((map) => {
        const parts = map.device.split(":").map((part) => BigInt(`0x${part}`));
        return (
          parts[0] === major && parts[1] === minor && BigInt(map.inode) === BigInt(image.stat.ino)
        );
      });
    assert(mappedIdentityEqual, "Mapped backing descriptor does not match device/inode");
    mirror(image);
    return { ...image, mappedIdentityEqual };
  });
  const stem = path.join(packet, `identity-${String(sequence++).padStart(3, "0")}-${event}`);
  fs.writeFileSync(`${stem}.maps`, maps);
  fs.writeFileSync(
    `${stem}.json`,
    `${JSON.stringify({ event, pid: process.pid, ppid: process.ppid, startTicks: startTicks(), sourceSha: process.env.VIZE_MOUNTED_CHILD_SOURCE_SHA, executable, mappings, images })}\n`,
  );
};
capture("start");
// Preserve the original write and bytes. This captures mappings after authored
// cleanup returns, before its final protocol publication; timing is diagnostic.
const write = Reflect.get(process.stdout, "write");
process.stdout.write = function (...args) {
  capture("stdout");
  return Reflect.apply(write, this, args);
};
process.on("beforeExit", () => capture("beforeExit"));
process.on("exit", () => capture("exit"));
