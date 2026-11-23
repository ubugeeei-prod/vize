import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
  symlinkSync,
} from "node:fs";
import { spawn } from "node:child_process";
if (process.argv[2] === "--wait-for-release") {
  writeFileSync(process.env.FAKE_CHILD_READY, String(process.pid));
  await new Promise((resolve) => {
    const timer = setInterval(() => {
      if (existsSync(process.env.FAKE_RELEASE)) {
        clearInterval(timer);
        writeFileSync(process.env.FAKE_LATE, "late child wrote");
        resolve();
      }
    }, 10);
  });
  process.exit(0);
}
const args = process.argv.slice(2),
  input = args.at(-1),
  mode = process.env.FAKE_MODE;
const tufRoot = process.env.TUF_ROOT;
const event = (type) =>
  appendFileSync(
    process.env.FAKE_EVENTS,
    JSON.stringify({
      type,
      input,
      pid: process.pid,
      args,
      yes: process.env.COSIGN_YES,
      tufRoot,
      cacheExists: existsSync(tufRoot),
      cacheMode: statSync(tufRoot).mode & 0o777,
    }) + "\n",
  );
let cacheConflict = false;
if (mode === "tuf-lock") {
  try {
    mkdirSync(tufRoot + "/LOCK");
  } catch {
    cacheConflict = true;
  }
}
event("start");
if (mode === "fail" && input === process.env.FAKE_TARGET) process.exit(9);
if (mode === "descendant" && input === process.env.FAKE_TARGET) {
  const child = spawn(process.execPath, [process.argv[1], "--wait-for-release"], {
    stdio: "ignore",
  });
  const ready = setInterval(() => {
    if (existsSync(process.env.FAKE_CHILD_READY)) {
      clearInterval(ready);
      child.unref();
      process.exit(0);
    }
  }, 10);
} else {
  const wait = setInterval(() => {
    if (!existsSync(process.env.FAKE_RELEASE)) return;
    clearInterval(wait);
    if (cacheConflict) {
      console.error("creating cached local store: resource temporarily unavailable");
      process.exit(9);
    }
    if (mode === "mutate" && input === process.env.FAKE_TARGET)
      writeFileSync(input, "changed input");
    for (const option of ["--bundle", "--output-certificate", "--output-signature"]) {
      const output = args[args.indexOf(option) + 1];
      if (input === process.env.FAKE_TARGET && option === "--output-signature") {
        if (mode === "missing") continue;
        if (mode === "empty") {
          writeFileSync(output, "");
          continue;
        }
        if (mode === "symlink") {
          symlinkSync(input.split("/").at(-1), output);
          continue;
        }
      }
      writeFileSync(output, option + " " + input + " " + readFileSync(input).toString());
    }
    if (mode === "extra" && input === process.env.FAKE_TARGET)
      writeFileSync(input + ".foreign.sig", "foreign");
    if (mode === "tuf-lock") rmSync(tufRoot + "/LOCK", { recursive: true });
    event("finish");
  }, 10);
}
