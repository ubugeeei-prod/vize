// Experimental plugin API v2 authoring. `setup` runs once per entry while the
// config is resolved and only records contributions; the host validates the
// lowered manifest and builds the canonical plan. See README.md.
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { SDK_FINGERPRINT } from "./identity.js";

export const PLUGIN_API_VERSION = 2;
/** Hosts discover entries through this registered symbol, never by module identity. */
export const PLUGIN_ENTRY = Symbol.for("vize.plugin.entry");

const V2_RUNTIME_FINGERPRINT = createHash("sha256")
  .update(readFileSync(new URL("v2.js", import.meta.url)))
  .digest("hex");
const DEFINITION_FIELDS = [
  "after",
  "apiVersion",
  "before",
  "cacheInputs",
  "name",
  "permissions",
  "setup",
  "stability",
  "supports",
  "version",
];

/**
 * Define an experimental v2 plugin. Returns a factory; each call with plugin
 * options returns one frozen entry for `vue.plugins`.
 */
export function defineVizePlugin(definition) {
  if (definition === null || typeof definition !== "object")
    throw new TypeError("defineVizePlugin needs a definition object");
  for (const key of Object.keys(definition)) {
    if (!DEFINITION_FIELDS.includes(key))
      throw new TypeError(`defineVizePlugin: unknown field \`${key}\``);
  }
  const { name, version, setup } = definition;
  if (definition.apiVersion !== PLUGIN_API_VERSION)
    throw new TypeError(`${name}: apiVersion must be ${PLUGIN_API_VERSION}`);
  if (definition.stability !== "experimental")
    throw new TypeError(`${name}: API v2 plugins must declare stability "experimental"`);
  if (typeof name !== "string" || typeof version !== "string" || typeof setup !== "function")
    throw new TypeError("defineVizePlugin needs a name, a version and a synchronous setup");
  return function createEntry(options = {}) {
    const capabilities = [];
    const callbacks = [];
    const sources = [];
    const returned = setup(createSetupApi(name, capabilities, callbacks, sources), options);
    if (returned !== undefined)
      throw new TypeError(`${name}: setup must return nothing (asynchronous setup is unsupported)`);
    const manifest = {
      after: clone(definition.after ?? []),
      apiVersion: PLUGIN_API_VERSION,
      backend: "js",
      before: clone(definition.before ?? []),
      cacheInputs: clone(definition.cacheInputs ?? []),
      capabilities,
      name,
      options: clone(options),
      permissions: clone(definition.permissions ?? []),
      stability: "experimental",
      supports: clone(definition.supports ?? {}),
      version,
    };
    const fingerprint = createHash("sha256")
      .update(
        JSON.stringify([SDK_FINGERPRINT, V2_RUNTIME_FINGERPRINT, manifest, String(setup), sources]),
      )
      .digest("hex");
    const descriptor = immutable({
      apiVersion: PLUGIN_API_VERSION,
      callbacks,
      manifest: { ...manifest, fingerprint },
    });
    return Object.freeze({ name, version, [PLUGIN_ENTRY]: descriptor });
  };
}

function createSetupApi(plugin, capabilities, callbacks, sources) {
  const add = (id, data, handlers = {}) => {
    capabilities.push({ data: clone(data), id, required: true, schemaVersion: 1 });
    sources.push(Object.entries(handlers).map(([key, callback]) => [key, String(callback)]));
    callbacks.push(
      Object.freeze(
        Object.fromEntries(
          Object.entries(handlers).map(([key, callback]) => [
            key,
            synchronous(plugin, id, callback),
          ]),
        ),
      ),
    );
  };
  return Object.freeze({
    lint: Object.freeze({
      /** Register one plugin-namespaced rule over public template records. */
      rule(rule) {
        if (rule === null || typeof rule !== "object" || typeof rule.run !== "function")
          throw new TypeError(`${plugin}: api.lint.rule needs an object with a run function`);
        const { run, meta = {}, ...selection } = rule;
        for (const key of Object.keys(meta)) {
          if (!["defaultLevel", "docs", "fixable", "messages"].includes(key))
            throw new TypeError(
              `${plugin}: lint rule ${rule.id} meta has unknown field \`${key}\``,
            );
        }
        add(
          "lint.rule",
          {
            ...selection,
            defaultLevel: meta.defaultLevel,
            docs: meta.docs,
            fixable: meta.fixable,
            messages: meta.messages,
          },
          { run },
        );
      },
    }),
    sfc: Object.freeze({
      /** Claim exclusive ownership of one custom block type and optional language. */
      block(type, contract = {}) {
        add("sfc.custom-block", { ...contract, type });
      },
    }),
    vue: Object.freeze({
      /** Declare one custom directive contract. */
      directive(name, contract = {}) {
        add("vue.directive", { ...contract, name });
      },
    }),
  });
}

function synchronous(plugin, where, callback) {
  return function guarded(...args) {
    const result = callback.apply(this, args);
    if (result && typeof result.then === "function")
      throw new TypeError(`${plugin} ${where}: asynchronous callbacks are unsupported`);
    return result;
  };
}

// Plain data is copied so freezing the manifest never freezes authored objects.
// Other values pass through for the host to refuse with a located error.
function clone(value) {
  if (Array.isArray(value)) return value.map(clone);
  if (value !== null && typeof value === "object") {
    const prototype = Object.getPrototypeOf(value);
    if (prototype !== Object.prototype && prototype !== null) return value;
    return Object.fromEntries(Object.entries(value).map(([key, child]) => [key, clone(child)]));
  }
  return value;
}

function immutable(value) {
  if (value !== null && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value)) immutable(child);
    Object.freeze(value);
  }
  return value;
}
