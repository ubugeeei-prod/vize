/* eslint-disable */
// @ts-nocheck

const { loadTarget, nativeTargets } = require("./native-targets");

const loadErrors = [];

function tryRequire(specifier) {
  try {
    return require(specifier);
  } catch (error) {
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

  for (const target of nativeTargets(loadErrors)) {
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

function buildLoadError() {
  if (loadErrors.length === 0) {
    return new Error("Failed to load native binding");
  }

  return new Error(
    `Cannot find native binding. ` +
      `npm has a bug related to optional dependencies (https://github.com/npm/cli/issues/4828). ` +
      "Please try `npm i` again after removing both package-lock.json and node_modules directory.",
    {
      cause: loadErrors.reduce((error, current) => {
        current.cause = error;
        return current;
      }),
    },
  );
}

let nativeBinding = loadNativeBinding();

if (!nativeBinding || process.env.NAPI_RS_FORCE_WASI) {
  nativeBinding = loadWasiBinding() || nativeBinding;
}

if (!nativeBinding) {
  throw buildLoadError();
}

module.exports = nativeBinding;
