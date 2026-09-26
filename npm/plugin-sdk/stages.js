import { createHash } from "node:crypto";
import { SDK_FINGERPRINT } from "./identity.js";

/** Define a synchronous native pre-canonical static-attribute transform. */
export function defineTransformPlugin({ name, version, cacheInputs, transform }) {
  return stage({ name, version, cacheInputs }, transform, (batch, edits) => {
    if (batch.schema !== 1 || batch.stage !== "s2-precanonical-static-attributes") {
      throw new Error(`${name}: unsupported L2 transform batch`);
    }
    return { schema: 1, edits };
  });
}

/** Define a formatter or output hook over an actual compiled artifact. */
export function defineOutputPlugin({ name, version, family, cacheInputs, output }) {
  if (!["formatter", "output"].includes(family)) throw new TypeError("unknown output hook family");
  return stage({ name, version, family, cacheInputs }, output, (batch, operations) => {
    if (batch.schema !== 1 || batch.family !== family || batch.offsetEncoding !== "utf8") {
      throw new Error(`${name}: unsupported compiled output batch`);
    }
    return operations;
  });
}

/** Define a namespaced provider consumed by native plugin fact demands. */
export function defineFactProvider({
  name,
  version,
  visit,
  demands = [],
  provides,
  cacheInputs,
  provide,
}) {
  return stage(
    { name, version, visit, demands, provides, cacheInputs },
    provide,
    (batch, facts) => {
      if (batch.schema !== 1) throw new Error(`${name}: unsupported fact visit batch`);
      return facts;
    },
  );
}

function stage(manifest, callback, encode) {
  if (!manifest.name || !manifest.version || typeof callback !== "function") {
    throw new TypeError("a plugin hook needs name, version and a synchronous function");
  }
  manifest = structuredClone(manifest);
  const fingerprint = createHash("sha256")
    .update(JSON.stringify([SDK_FINGERPRINT, manifest, String(callback)]))
    .digest("hex");
  return immutable({
    ...manifest,
    fingerprint,
    run(batchJson) {
      const batch = immutable(JSON.parse(batchJson));
      const result = callback(batch);
      if (result?.then) throw new TypeError(`${manifest.name}: asynchronous hooks are unsupported`);
      return JSON.stringify(encode(batch, result));
    },
  });
}

function immutable(value) {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value)) immutable(child);
    Object.freeze(value);
  }
  return value;
}
