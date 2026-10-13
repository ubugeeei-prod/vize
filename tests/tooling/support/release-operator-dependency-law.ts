import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { writeFakeCommand } from "./fake-command.ts";

/** Models capture argv only; no Rust, Moon or publication command is executed. */
export function dependencyCommands(bin: string, capture: string, dependencies: string) {
  for (const command of ["vp", "moon", "rust-script"])
    writeFakeCommand(
      bin,
      command,
      `
      const fs = require('node:fs'), args = process.argv.slice(2), command = ${JSON.stringify(command)};
      if (command === 'moon' || args[0] === 'install') {
        fs.appendFileSync(${JSON.stringify(dependencies)}, JSON.stringify([command, ...args]) + '\\n');
        process.exit(Number(process.env.TEST_FROZEN_EXIT ?? 0));
      }
      fs.writeFileSync(${JSON.stringify(capture)}, JSON.stringify(command === 'rust-script' ? [command, ...args] : args));
    `,
    );
}

/** Execute the actual YAML dependency shell for every operation and frozen-install refusal. */
export function dependencyShell(
  run: string,
  root: string,
  bin: string,
  capture: string,
  dependencyCapture: string,
) {
  const setup = (operation: string, status = "0") =>
    spawnSync("bash", ["-e", "-c", run], {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
        RELEASE_OPERATION: operation,
        TEST_FROZEN_EXIT: status,
      },
    });
  const install = ["vp", "install", "--frozen-lockfile"];
  const calls = () =>
    fs
      .readFileSync(dependencyCapture, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
  for (const operation of ["start", "patch", "resume", "retire"]) {
    const result = setup(operation);
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(calls(), operation === "retire" ? [install] : [install, ["moon", "update"]]);
    fs.rmSync(dependencyCapture);
    assert.equal(fs.existsSync(capture), false);
  }
  for (const operation of ["start", "patch", "resume", "retire"]) {
    assert.equal(setup(operation, "17").status, 17);
    assert.deepEqual(calls(), [install]);
    assert.equal(fs.existsSync(capture), false);
    fs.rmSync(dependencyCapture);
  }
}
