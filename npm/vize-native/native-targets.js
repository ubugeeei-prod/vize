/* eslint-disable */
// @ts-nocheck

const { readFileSync } = require("node:fs");
const { execSync } = require("node:child_process");

const binaryName = "vize-vitrine";
const packageVersion = require("./package.json").version;

function isFileMusl(file) {
  return file.includes("libc.musl-") || file.includes("ld-musl-");
}

function isMuslFromFilesystem() {
  try {
    return readFileSync("/usr/bin/ldd", "utf-8").includes("musl");
  } catch {
    return null;
  }
}

function isMuslFromReport() {
  let report = null;

  if (typeof process.report?.getReport === "function") {
    process.report.excludeNetwork = true;
    report = process.report.getReport();
  }

  if (!report) {
    return null;
  }
  if (report.header && report.header.glibcVersionRuntime) {
    return false;
  }
  if (Array.isArray(report.sharedObjects)) {
    return report.sharedObjects.some(isFileMusl);
  }

  return false;
}

function isMuslFromChildProcess() {
  try {
    return execSync("ldd --version", { encoding: "utf8" }).includes("musl");
  } catch {
    return false;
  }
}

function isMusl() {
  if (process.platform !== "linux") {
    return false;
  }

  const fromFs = isMuslFromFilesystem();
  if (fromFs !== null) {
    return fromFs;
  }

  const fromReport = isMuslFromReport();
  if (fromReport !== null) {
    return fromReport;
  }

  return isMuslFromChildProcess();
}

function nativeTargets(loadErrors) {
  if (process.platform === "darwin") {
    if (process.arch === "arm64" || process.arch === "x64") {
      return ["darwin-universal", `darwin-${process.arch}`];
    }
    loadErrors.push(new Error(`Unsupported architecture on macOS: ${process.arch}`));
    return [];
  }

  if (process.platform === "linux") {
    if (process.arch === "arm64" || process.arch === "x64") {
      return [`linux-${process.arch}-${isMusl() ? "musl" : "gnu"}`];
    }
    loadErrors.push(new Error(`Unsupported architecture on Linux: ${process.arch}`));
    return [];
  }

  if (process.platform === "win32") {
    if (process.arch === "arm64" || process.arch === "x64") {
      return [`win32-${process.arch}-msvc`];
    }
    loadErrors.push(new Error(`Unsupported architecture on Windows: ${process.arch}`));
    return [];
  }

  loadErrors.push(new Error(`Unsupported OS: ${process.platform}, architecture: ${process.arch}`));
  return [];
}

function requireTargetPackage(target) {
  const packageName = `@vizejs/native-${target}`;
  const binding = require(packageName);
  const bindingPackageVersion = require(`${packageName}/package.json`).version;

  if (
    bindingPackageVersion !== packageVersion &&
    process.env.VIZE_ALLOW_NATIVE_VERSION_MISMATCH !== "1"
  ) {
    throw new Error(
      `Native binding package version mismatch, expected ${packageVersion} but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`,
    );
  }

  return binding;
}

function loadTarget(target, loadErrors) {
  try {
    return require(`./${binaryName}.${target}.node`);
  } catch (error) {
    loadErrors.push(error);
  }

  try {
    return requireTargetPackage(target);
  } catch (error) {
    loadErrors.push(error);
  }

  return null;
}

module.exports = { loadTarget, nativeTargets };
