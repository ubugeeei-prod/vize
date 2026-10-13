/* eslint-disable */
// @ts-nocheck

const { randomUUID } = require("node:crypto");
const { extname } = require("node:path");
const registry = Symbol.for("@vizejs/native/binding-identities/v1");
const shared = globalThis[registry];
const identities = shared instanceof WeakMap ? shared : new WeakMap();
if (shared === undefined) {
  try {
    Object.defineProperty(globalThis, registry, { value: identities });
  } catch {
    // A frozen global still permits safe identities local to this helper copy.
  }
}

function remember(binding, isNative = false) {
  if ((typeof binding !== "object" || binding === null) && typeof binding !== "function") return;
  if (identities.has(binding)) return;
  let token;
  if (isNative) {
    try {
      token = binding.getNativeBuildToken?.();
    } catch {
      // An unavailable or unknown provider cannot authorize persistent reuse.
    }
  }
  identities.set(
    binding,
    Object.freeze(
      typeof token === "string" && /^[0-9a-f]{32}$/.test(token)
        ? { kind: "native", token }
        : { kind: "process", nonce: randomUUID() },
    ),
  );
}

// Read the actual image's immutable token, never a possibly replaced disk file.
function requireWithBindingIdentity(requireBinding, specifier) {
  const binding = requireBinding(specifier);
  let isNative = false;
  try {
    isNative = extname(requireBinding.resolve(specifier)) === ".node";
  } catch {
    // A cached binding can still load after its original path disappears.
  }
  remember(binding, isNative);
  return binding;
}

function getBindingIdentity(binding) {
  remember(binding);
  return identities.get(binding);
}

module.exports = { requireWithBindingIdentity, getBindingIdentity };
