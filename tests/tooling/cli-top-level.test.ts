import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function resolveVizeCommand(): { command: string; prefix: string[] } {
  const candidates = [
    path.join(root, "target/ci/vize"),
    path.join(root, "target/release/vize"),
    path.join(root, "target/debug/vize"),
    "vize",
  ];
  for (const candidate of candidates) {
    const probe = spawnSync(candidate, ["--version"], { cwd: root, encoding: "utf8" });
    if (probe.status === 0) {
      return { command: candidate, prefix: [] };
    }
  }
  return { command: "cargo", prefix: ["run", "-q", "-p", "vize", "--"] };
}

const VIZE = resolveVizeCommand();

function runVize(args: string[], cwd: string = root) {
  const result = spawnSync(VIZE.command, [...VIZE.prefix, ...args], { cwd, encoding: "utf8" });
  if (result.error) {
    throw result.error;
  }
  return result;
}

function workspaceVersion(): string {
  const version = fs
    .readFileSync(path.join(root, "Cargo.toml"), "utf-8")
    .match(/^version = "(.+)"$/m)?.[1];
  assert.ok(version, "Cargo.toml should declare a workspace version");
  return version;
}

test("vize --version matches the workspace Cargo.toml version", () => {
  const result = runVize(["--version"]);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.trim(), `vize ${workspaceVersion()}`);
});

test("vize --help advertises the documented subcommands and their aliases", () => {
  const result = runVize(["--help"]);
  assert.equal(result.status, 0, result.stderr);
  const help = result.stdout;

  for (const subcommand of ["build", "fmt", "lint", "check", "clean", "lsp", "ready"]) {
    assert.match(help, new RegExp(`\\b${subcommand}\\b`), `help should list ${subcommand}`);
  }

  // The artistic command aliases are part of the published CLI surface.
  for (const [command, alias] of [
    ["build", "atelier"],
    ["fmt", "glyph"],
    ["lint", "patina"],
    ["lsp", "maestro"],
  ]) {
    assert.match(
      help,
      new RegExp(`${command}[\\s\\S]*?${alias}`),
      `${command} should alias ${alias}`,
    );
  }
});

test("vize rejects an unknown subcommand with a clap usage error", () => {
  const result = runVize(["definitely-not-a-command"]);
  assert.equal(result.status, 2, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stderr, /unrecognized subcommand/);
});

test("vize clean --help documents the optional project root argument", () => {
  const result = runVize(["clean", "--help"]);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /ROOT/);
});

test("vize clean exits 0 with a clear notice when no artifacts exist", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cli-clean-"));
  try {
    const result = runVize(["clean"], dir);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.match(`${result.stdout}${result.stderr}`, /No managed Vize artifacts found/);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});
