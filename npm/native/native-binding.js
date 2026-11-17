/* eslint-disable */
// @ts-nocheck

const { loadTarget, nativeTargets } = require("./native-targets");
const { requireWithBindingIdentity } = require("./native-binding-identity");
const loadErrors = [];
let attemptedTargets = [];

function isMissingModuleForSpecifier(error, specifier) {
  return (
    error?.code === "MODULE_NOT_FOUND" &&
    typeof error.message === "string" &&
    error.message.includes(`'${specifier}'`)
  );
}

function tryRequire(specifier) {
  try {
    return requireWithBindingIdentity(require, specifier);
  } catch (error) {
    if (isMissingModuleForSpecifier(error, specifier)) {
      error.vizeMissingNativeSpecifier = true;
    }
    loadErrors.push(error);
    return null;
  }
}

function loadConfiguredBinding() {
  if (!process.env.NAPI_RS_NATIVE_LIBRARY_PATH) {
    return null;
  }
  return tryRequire(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
}

function loadNativeBinding() {
  const configured = loadConfiguredBinding();
  if (configured) {
    return configured;
  }

  attemptedTargets = nativeTargets(loadErrors);
  for (const target of attemptedTargets) {
    const binding = loadTarget(target, loadErrors);
    if (binding) {
      return binding;
    }
  }

  return null;
}

function loadWasiBinding() {
  let wasiBinding = null;
  let wasiBindingError = null;

  wasiBinding = tryRequire("./vize-vitrine.wasi.cjs");
  if (!wasiBinding && process.env.NAPI_RS_FORCE_WASI) {
    wasiBindingError = loadErrors[loadErrors.length - 1];
  }

  if (!wasiBinding) {
    wasiBinding = tryRequire("@vizejs/native-wasm32-wasi");
    if (!wasiBinding && process.env.NAPI_RS_FORCE_WASI) {
      const error = loadErrors[loadErrors.length - 1];
      if (!wasiBindingError) {
        wasiBindingError = error;
      } else if (wasiBindingError !== error) {
        wasiBindingError.cause = error;
      }
    }
  }

  if (process.env.NAPI_RS_FORCE_WASI === "error" && !wasiBinding) {
    const error = new Error("WASI binding not found and NAPI_RS_FORCE_WASI is set to error");
    error.cause = wasiBindingError;
    throw error;
  }

  return wasiBinding;
}

// Every Linux GNU binding is linked against this glibc and gated on it at
// release (tools/support/github/glibc-floor.ts keeps the same value).
const linuxGnuGlibcFloor = "2.17";
const linuxFloorDocs =
  "https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#linux-glibc-floor";

function compareVersions(left, right) {
  const a = left.split(".").map(Number);
  const b = right.split(".").map(Number);
  for (let index = 0; index < Math.max(a.length, b.length); index += 1) {
    const difference = (a[index] ?? 0) - (b[index] ?? 0);
    if (difference !== 0) return difference;
  }
  return 0;
}

function runtimeGlibcVersion() {
  if (process.platform !== "linux") return null;
  try {
    const version = process.report?.getReport?.()?.header?.glibcVersionRuntime;
    return typeof version === "string" ? version : null;
  } catch {
    return null;
  }
}

// `/lib64/libm.so.6: version `GLIBC_2.35' not found (required by ...)`
function glibcHints(message) {
  const required = /version [`'"]?(GLIBC_[A-Z0-9_.]+)[`'"]? not found/.exec(message)?.[1];
  if (!required) return null;
  const system = runtimeGlibcVersion();
  const version = require("./package.json").version;
  const hints = [
    `This system has ${system ? `glibc ${system}` : "an unknown glibc"}; the binding asks for ${required}.`,
  ];
  if (system && compareVersions(system, linuxGnuGlibcFloor) < 0) {
    hints.push(
      `@vizejs/native ${version} supports glibc ${linuxGnuGlibcFloor} or newer: run on a newer distribution or base image.`,
    );
  } else {
    hints.push(
      `@vizejs/native ${version} is built for glibc ${linuxGnuGlibcFloor} or newer, so this is a packaging bug. ` +
        "Please report it at https://github.com/ubugeeei-prod/vize/issues with this message. " +
        "Until a fixed release, use a distribution with a newer glibc.",
    );
  }
  hints.push(
    `The musl build (@vizejs/native-linux-${process.arch}-musl) only loads on musl systems such as Alpine, not in a glibc process.`,
    `Supported Linux systems: ${linuxFloorDocs}`,
  );
  return hints;
}

function dlopenHints(error) {
  const message = String(error?.message ?? "");
  const glibc = glibcHints(message);
  if (glibc) return glibc;
  if (/cannot open shared object file/.test(message)) {
    return [
      "A system library the binding links is missing. Linux GNU bindings need only libc, libm, and libgcc_s (`libgcc` / `libgcc-s1`); install the missing one.",
    ];
  }
  if (
    /invalid ELF header|wrong ELF class|Exec format error|not a valid Win32 application/.test(
      message,
    )
  ) {
    return [
      "The installed binding was built for another platform. Install dependencies on this machine instead of copying node_modules from another OS or CPU.",
    ];
  }
  return [];
}

function nodeEngineFloor() {
  const range = require("./package.json").engines?.node;
  return /^>=\s*(\d+(?:\.\d+){0,2})$/.exec(range ?? "")?.[1] ?? null;
}

function missingHints() {
  const packages = attemptedTargets.map((target) => `@vizejs/native-${target}`);
  const glibc = runtimeGlibcVersion();
  const libc = process.platform === "linux" ? `, ${glibc ? `glibc ${glibc}` : "musl"}` : "";
  const hints = [
    `${packages.join(" or ") || "No platform package"} is not installed (${process.platform}-${process.arch}${libc}, Node.js ${process.versions.node}).`,
  ];
  const nodeFloor = nodeEngineFloor();
  if (nodeFloor && compareVersions(process.versions.node, nodeFloor) < 0) {
    hints.push(
      `@vizejs/native supports Node.js ${nodeFloor} or newer, and package managers skip its platform package on older Node.js versions. Use Node.js ${nodeFloor} or newer and install again.`,
    );
  } else {
    hints.push(
      "The platform package is an optional dependency: install without --omit=optional / --no-optional, and do not reuse node_modules or a lockfile made on another OS or CPU.",
      "npm has a bug related to optional dependencies (https://github.com/npm/cli/issues/4828): if it persists, remove package-lock.json and node_modules and install again.",
    );
  }
  return hints;
}

function buildLoadError() {
  if (loadErrors.length === 0) {
    return new Error("Failed to load native binding");
  }

  // A binding that exists but cannot be loaded outranks every candidate that
  // is merely absent (the local file, the WASI fallback).
  const loadFailure = loadErrors.find((error) => !error?.vizeMissingNativeSpecifier);
  if (loadFailure) {
    return new Error(
      [`Failed to load native binding: ${loadFailure.message}`, ...dlopenHints(loadFailure)].join(
        "\n",
      ),
      { cause: loadFailure },
    );
  }

  return new Error(["Cannot find native binding.", ...missingHints()].join("\n"), {
    cause: loadErrors.reduce((error, current) => {
      current.cause = error;
      return current;
    }),
  });
}

let nativeBinding = loadNativeBinding();

if (!nativeBinding || process.env.NAPI_RS_FORCE_WASI) {
  nativeBinding = loadWasiBinding() || nativeBinding;
}

if (!nativeBinding) {
  throw buildLoadError();
}

module.exports = nativeBinding;
