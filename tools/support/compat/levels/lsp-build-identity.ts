#!/usr/bin/env node
// Bind resource results to the actual binary, features, toolchain and Linux ABI.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { isRecord } from "../../json-value.ts";

const [serverArgument, profile, buildLog, out] = process.argv.slice(2);
assert.ok(serverArgument && buildLog && out, "server profile cargo-json-log output are required");
assert.ok(profile === "ci-opt" || profile === "release", "record a known Cargo profile");
const server = fs.realpathSync(serverArgument);
const command = (name: string, args: string[]) =>
  execFileSync(name, args, { encoding: "utf8" }).trim();
const artifacts: unknown[] = fs
  .readFileSync(buildLog, "utf8")
  .split("\n")
  .filter(Boolean)
  .map((line: string): unknown => JSON.parse(line));
const artifact = artifacts.find(
  (entry): entry is Record<string, unknown> =>
    isRecord(entry) &&
    entry.reason === "compiler-artifact" &&
    typeof entry.executable === "string" &&
    path.resolve(entry.executable) === server,
);
assert.ok(artifact, "Cargo reports the exact measured executable and enabled features");
const binary = fs.readFileSync(server);
const manifest = fs.readFileSync("Cargo.toml", "utf8");
const section = (name: string) => {
  const header = `[profile.${name}]`;
  const start = manifest.indexOf(header);
  assert.ok(start >= 0, `Cargo manifest records ${name}`);
  const end = manifest.indexOf("\n[", start + header.length);
  return manifest.slice(start, end < 0 ? undefined : end).trim();
};
const measurement = {
  source: {
    revision: command("git", ["rev-parse", "HEAD"]),
    tree: command("git", ["rev-parse", "HEAD^{tree}"]),
    github_run: process.env.GITHUB_RUN_ID
      ? `https://github.com/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}`
      : undefined,
  },
  build: {
    cargo_profile: profile,
    target: "x86_64-unknown-linux-gnu",
    features: artifact.features,
    artifact_profile: artifact.profile,
    profile_manifest: [
      section("release"),
      section("release.package.vize"),
      ...(profile === "ci-opt" ? [section(profile), section("ci-opt.package.vize")] : []),
    ],
    rustc: command("rustc", ["--version", "--verbose"]),
    cargo: command("cargo", ["--version", "--verbose"]),
    default_linker: "rust-lld; no Wild or linker override in this workflow",
  },
  binary: {
    path: server,
    bytes: binary.byteLength,
    sha256: createHash("sha256").update(binary).digest("hex"),
    elf_header: command("readelf", ["--file-header", server]),
    elf_dynamic: command("readelf", ["--dynamic", server]),
    elf_symbol_versions: command("readelf", ["--version-info", server]),
  },
};
fs.writeFileSync(out, `${JSON.stringify(measurement, null, 2)}\n`);
console.log(`${profile}: ${measurement.binary.bytes} bytes; SHA256 ${measurement.binary.sha256}`);
