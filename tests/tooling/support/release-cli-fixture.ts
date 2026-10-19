import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { writeFakeCommand } from "./fake-command.ts";
import { releaseCommandPath, repoRoot, runReleaseCommand } from "./release-command.ts";

export interface DriverEvent {
  kind: "driver" | "authenticate";
  args: string[];
  cargo: string | null;
}

/** Only the mutating Rust release driver is controlled; the release CLI is real. */
export function releaseCliFixture(authenticate = false) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-cli-"));
  const bin = path.join(dir, "bin");
  const log = path.join(dir, "events.jsonl");
  const cargo = '[workspace.package]\nversion = "0.440.0"\n';
  fs.mkdirSync(bin);
  fs.mkdirSync(path.join(dir, "npm"));
  fs.writeFileSync(path.join(dir, "Cargo.toml"), cargo);
  fs.writeFileSync(log, "");
  writeFakeCommand(
    bin,
    "rust-script",
    `
    const fs = require('node:fs');
    const args = process.argv.slice(2);
    const kind = args[1] === 'validate-preparation-target' ? 'authenticate' : 'driver';
    fs.appendFileSync(process.env.RELEASE_EVENTS, JSON.stringify({ kind, args,
      cargo: fs.existsSync('Cargo.toml') ? fs.readFileSync('Cargo.toml', 'utf8') : null }) + '\\n');
    process.exit(${authenticate} && kind === 'authenticate' ? 0 : 83);
  `,
  );
  const env: NodeJS.ProcessEnv = {
    PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
    RELEASE_EVENTS: log,
  };
  const events = (): DriverEvent[] =>
    fs
      .readFileSync(log, "utf8")
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line) as DriverEvent);
  return {
    dir,
    env,
    cargo,
    events,
    run: (args: string[]) => runReleaseCommand("release", args, { cwd: dir, env }),
    prompt: (answer: string, args: string[]) => {
      const childEnv: NodeJS.ProcessEnv = { ...process.env, ...env };
      delete childEnv.NODE_TEST_CONTEXT;
      return spawnSync(
        process.execPath,
        [
          path.join(repoRoot, "tests/tooling/support/pty-command.ts"),
          "Proceed with release? [y/N]",
          answer,
          process.execPath,
          releaseCommandPath("release"),
          ...args,
        ],
        { cwd: dir, env: childEnv, encoding: "utf8", timeout: 30_000 },
      );
    },
    assertUnchanged() {
      assert.equal(fs.readFileSync(path.join(dir, "Cargo.toml"), "utf8"), cargo);
    },
    dispose() {
      fs.rmSync(dir, { recursive: true, force: true });
    },
  };
}
