import assert from "node:assert/strict";
import fs from "node:fs";
import { syncBuiltinESMExports } from "node:module";

const fixture = JSON.parse(process.argv[2]);

const target = fixture.modulesPath,
  link = fixture.linkPath;
if (fixture.mode === "link-revision") {
  const read = fs.readlinkSync;
  let changed = false;
  fs.readlinkSync = (...args) => {
    const raw = read(...args);
    if (args[0] === link && !changed) {
      changed = true;
      fs.lutimesSync(link, new Date(10000), new Date(20000));
    }
    return raw;
  };
} else {
  const open = fs.openSync;
  let changed = false;
  fs.openSync = (...args) => {
    const fd = open(...args);
    if (args[0] === target && !changed) {
      changed = true;
      fs.renameSync(target, fixture.previousModules);
      fs.mkdirSync(target);
    }
    return fd;
  };
}
syncBuiltinESMExports();
const { captureSourceCustody } = await import(fixture.custodyModule);
assert.throws(() => captureSourceCustody(fixture.options), /changed during custody/);
