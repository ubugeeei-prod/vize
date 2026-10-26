import fs from "node:fs";
import path from "node:path";

const POISON_MESSAGE = "outside vize executable reached";

function writeExecutable(filename, source) {
  fs.writeFileSync(filename, source);
  if (process.platform !== "win32") {
    fs.chmodSync(filename, 0o755);
  }
}

/**
 * Places a failing `vize` command before the inherited PATH.
 *
 * Package-manager script runners should still prefer the fresh project's own
 * `node_modules/.bin`; reaching this executable means the release smoke stopped
 * proving the generated command against project-local artifacts.
 */
export function withPoisonedVizePath(projectRoot, env = process.env) {
  const poisonDir = path.join(projectRoot, ".vize-path-poison");
  fs.mkdirSync(poisonDir, { recursive: true });
  writeExecutable(
    path.join(poisonDir, "vize"),
    ["#!/bin/sh", `echo '${POISON_MESSAGE}' >&2`, "exit 42", ""].join("\n"),
  );
  fs.writeFileSync(
    path.join(poisonDir, "vize.cmd"),
    ["@echo off", `echo ${POISON_MESSAGE} 1>&2`, "exit /b 42", ""].join("\r\n"),
  );
  return {
    ...env,
    PATH: [poisonDir, env.PATH ?? ""].filter((segment) => segment.length > 0).join(path.delimiter),
  };
}
