#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const [packageName, binName, ...args] = process.argv.slice(2);

if (packageName == null || binName == null) {
  console.error("Usage: run-package-bin.mjs <package-name> <bin-name> [args...]");
  process.exit(1);
}

const cwd = process.cwd();
const nodeModules = path.join(cwd, "node_modules");
const packageRoot = resolvePackageRoot(packageName);
const packageJson = readJson(path.join(packageRoot, "package.json"));
const relativeBin = resolveBinPath(packageJson, binName);
const binPath = path.resolve(packageRoot, relativeBin);

if (!fs.existsSync(binPath)) {
  console.error(`Package bin "${binName}" for ${packageName} does not exist at ${binPath}`);
  process.exit(1);
}

const runWithNode = process.platform === "win32" && !/\.(?:bat|cmd|exe)$/i.test(binPath);
const command = runWithNode ? process.execPath : binPath;
const commandArgs = runWithNode ? [binPath, ...args] : args;
const result = spawnSync(command, commandArgs, {
  cwd,
  env: process.env,
  stdio: "inherit",
});

if (result.error != null) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);

function resolvePackageRoot(name) {
  const directPackagePath = path.join(nodeModules, ...name.split("/"));
  if (fs.existsSync(path.join(directPackagePath, "package.json"))) {
    return directPackagePath;
  }

  const packageMapPath = path.join(nodeModules, ".package-map.json");
  if (!fs.existsSync(packageMapPath)) {
    console.error(`Cannot resolve ${name}: ${packageMapPath} does not exist`);
    process.exit(1);
  }

  const packageMap = readJson(packageMapPath);
  const packages = packageMap.packages;
  if (packages == null || typeof packages !== "object") {
    console.error(`Cannot resolve ${name}: ${packageMapPath} has no packages object`);
    process.exit(1);
  }

  const expectedVersion = exactManifestVersion(name);
  const candidates = Object.entries(packages)
    .filter(([, entry]) => {
      if (entry == null || typeof entry !== "object" || typeof entry.url !== "string") {
        return false;
      }
      if (!normalizeSeparators(entry.url).endsWith(`/node_modules/${name}`)) {
        return false;
      }
      return fs.existsSync(path.join(nodeModules, entry.url, "package.json"));
    })
    .sort(
      ([left], [right]) =>
        scorePackageId(right, name, expectedVersion) - scorePackageId(left, name, expectedVersion),
    );

  if (candidates.length === 0) {
    console.error(`Cannot resolve ${name}: no matching package entry in ${packageMapPath}`);
    process.exit(1);
  }

  const [, entry] = candidates[0];
  return path.resolve(nodeModules, entry.url);
}

function exactManifestVersion(name) {
  const manifestPath = path.join(cwd, "package.json");
  if (!fs.existsSync(manifestPath)) {
    return null;
  }

  const manifest = readJson(manifestPath);
  for (const key of ["dependencies", "devDependencies", "optionalDependencies"]) {
    const dependencies = manifest[key];
    const value = dependencies?.[name];
    if (typeof value === "string" && /^\d+\.\d+\.\d+(?:[-+].*)?$/.test(value)) {
      return value;
    }
  }

  return null;
}

function scorePackageId(id, name, expectedVersion) {
  let score = 0;
  if (expectedVersion != null && id.startsWith(`${name}@${expectedVersion}`)) {
    score += 2;
  }
  if (!id.includes("(")) {
    score += 1;
  }
  return score;
}

function resolveBinPath(packageJson, name) {
  const bin = packageJson.bin;
  if (typeof bin === "string") {
    return bin;
  }
  if (bin != null && typeof bin === "object" && typeof bin[name] === "string") {
    return bin[name];
  }

  console.error(`Package ${packageJson.name ?? "(unknown)"} does not expose bin "${name}"`);
  process.exit(1);
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function normalizeSeparators(value) {
  return value.replaceAll(path.sep, "/");
}
