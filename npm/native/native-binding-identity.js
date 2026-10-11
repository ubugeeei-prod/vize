/* eslint-disable */
// @ts-nocheck

const { createHash, randomUUID } = require("node:crypto");
const { closeSync, fstatSync, openSync, readSync, statSync } = require("node:fs");
const { extname } = require("node:path");

const identities = new WeakMap();

function sameFile(before, after) {
  return ["dev", "ino", "size", "mtimeNs", "ctimeNs"].every((key) => before[key] === after[key]);
}

function fingerprint(filename) {
  const hash = createHash("sha256");
  const buffer = Buffer.allocUnsafe(1024 * 1024);
  const descriptor = openSync(filename, "r");
  try {
    const before = fstatSync(descriptor, { bigint: true });
    let bytes;
    while ((bytes = readSync(descriptor, buffer, 0, buffer.length, null)) > 0) {
      hash.update(buffer.subarray(0, bytes));
    }
    if (!sameFile(before, fstatSync(descriptor, { bigint: true }))) return null;
    if (!sameFile(before, statSync(filename, { bigint: true }))) return null;
    return { before, sha256: hash.digest("hex"), bytes: Number(before.size), filename };
  } finally {
    closeSync(descriptor);
  }
}

function remember(binding, captured) {
  if ((typeof binding !== "object" || binding === null) && typeof binding !== "function") {
    return;
  }
  if (!identities.has(binding)) {
    identities.set(
      binding,
      Object.freeze(
        captured
          ? {
              kind: "native",
              sha256: captured.sha256,
              bytes: captured.bytes,
              filename: captured.filename,
            }
          : { kind: "process", nonce: randomUUID() },
      ),
    );
  }
}

/** Capture the body at its first load; later disk replacements cannot relabel loaded memory. */
function requireWithBindingIdentity(requireBinding, specifier) {
  const filename = requireBinding.resolve(specifier);
  let captured = null;
  if (!requireBinding.cache[filename] && extname(filename) === ".node") {
    try {
      captured = fingerprint(filename);
    } catch {
      // Loading may succeed even when a trustworthy body identity is unavailable.
    }
  }
  const binding = requireBinding(specifier);
  if (captured) {
    try {
      if (!sameFile(captured.before, statSync(filename, { bigint: true }))) captured = null;
    } catch {
      captured = null;
    }
  }
  remember(binding, captured);
  return binding;
}

function getBindingIdentity(binding) {
  remember(binding, null);
  return identities.get(binding);
}

module.exports = { requireWithBindingIdentity, getBindingIdentity };
