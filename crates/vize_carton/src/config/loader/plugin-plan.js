// Resolve `vue.plugins` once into the canonical immutable plugin plan.
// Every product reads this plan; generated receipts never replace it.
import {
  PLUGIN_REGISTRY,
  claimKey,
  normalizeCapability,
  ownershipConflict,
} from "./plugin-capabilities.js";
import {
  VizePluginPlanError,
  canonicalData,
  canonicalJson,
  compareCodePoints,
  deepFreeze,
  digestOf,
} from "./plugin-canonical.js";

export { PLUGIN_REGISTRY, VizePluginPlanError };
export const PLUGIN_PLAN_SCHEMA = PLUGIN_REGISTRY.planSchema;
export const PLUGIN_ENTRY = Symbol.for("vize.plugin.entry");

const NAME = /^(?:@[a-z0-9][a-z0-9._-]*\/)?[a-z0-9][a-z0-9._-]*$/;
const FINGERPRINT = /^[0-9a-f]{64}$/;
const MANIFEST_FIELDS = [
  "after",
  "apiVersion",
  "backend",
  "before",
  "cacheInputs",
  "capabilities",
  "fingerprint",
  "name",
  "options",
  "permissions",
  "stability",
  "supports",
  "version",
];
const SUPPORTS = { outputs: ["ssr", "vapor", "vdom"], runtimes: ["bun", "node"], vue: ["2", "3"] };
const PERMISSIONS = { env: "keys", "fs.read": "paths" };

/**
 * Build the plan for authored `vue.plugins`. `false`, `null` and `undefined`
 * entries are skipped like conditional Vite plugins; every other entry must be
 * a v2 entry or a v1 rule plugin hosted through the compatibility adapter.
 */
export function resolvePluginPlan(entries, host = {}) {
  if (entries === undefined) entries = [];
  if (!Array.isArray(entries))
    throw new VizePluginPlanError("entry-invalid", "vue.plugins", "must be an array");
  const runtime = host.runtime ?? (globalThis.Bun ? "bun" : "node");
  const lowered = [];
  const callbacks = new Map();
  const sources = new Map();
  entries.forEach((entry, index) => {
    if (entry === false || entry === null || entry === undefined) return;
    const { manifest, handlers } = lowerEntry(entry, `vue.plugins[${index}]`);
    const plugin = validateManifest(manifest, index, runtime);
    sources.set(plugin.name, entry);
    plugin.capabilities.forEach((capability, position) => {
      const handler = handlers[capability.source] ?? {};
      for (const name of PLUGIN_REGISTRY.capabilities[capability.id].callbacks) {
        if (typeof handler[name] !== "function")
          throw new VizePluginPlanError(
            "callback-missing",
            `${plugin.name} ${capability.id} ${capability.key}`,
            `the \`${name}\` callback is missing`,
          );
        callbacks.set(callbackKey(plugin.name, capability.id, capability.key, name), handler[name]);
      }
      delete plugin.capabilities[position].source;
    });
    lowered.push(plugin);
  });
  const plugins = orderPlugins(lowered);
  const claims = plugins
    .flatMap((plugin) =>
      plugin.capabilities.map((capability) => ({
        key: claimKey(capability.id, capability.key),
        owner: plugin.name,
      })),
    )
    .sort((left, right) => compareCodePoints(left.key, right.key));
  const conflict = ownershipConflict(claims);
  if (conflict) {
    const [first, second] = conflict;
    throw new VizePluginPlanError(
      "owner-conflict",
      second.key,
      first.key !== second.key
        ? `${second.owner} claims it, but ${first.owner} owns ${first.key} for every language`
        : first.owner === second.owner
          ? `${first.owner} claims it more than once`
          : `claimed by both ${first.owner} and ${second.owner}`,
    );
  }
  const plan = deepFreeze({ claims, planSchema: PLUGIN_PLAN_SCHEMA, plugins });
  const json = canonicalJson(plan);
  return Object.freeze({ plan, json, digest: digestOf(json), callbacks, entries: sources });
}

export function callbackKey(plugin, capability, key, name) {
  return `${plugin}\u0000${capability}\u0000${key}\u0000${name}`;
}

/**
 * Refuse a capability until every product that consumes it implements it,
 * whichever product loads it, so no product silently ignores it.
 */
export function refuseUnimplementedCapabilities(plan) {
  for (const plugin of plan.plugins) {
    for (const capability of plugin.capabilities) {
      const { consumers, admittedBy } = PLUGIN_REGISTRY.capabilities[capability.id];
      const missing = consumers.filter((consumer) => !admittedBy.includes(consumer));
      if (missing.length === 0) continue;
      const subject = `${capability.id} \`${capability.key}\``;
      const refusal =
        admittedBy.length === 0
          ? `no Vize product implements ${subject} yet`
          : `${subject} is not implemented by ${missing.join(", ")} yet`;
      throw new VizePluginPlanError(
        "capability-unadmitted",
        plugin.name,
        `${refusal}; run \`vize plugin inspect\` for the plan`,
      );
    }
  }
}

/** An empty plan has no observable product behavior. */
export function isEmptyPluginPlan(plan) {
  return plan.plugins.length === 0;
}

function lowerEntry(entry, where) {
  if (typeof entry !== "object" && typeof entry !== "function")
    throw new VizePluginPlanError("entry-invalid", where, "is not a Vize plugin entry");
  if (Array.isArray(entry) || typeof entry.then === "function")
    throw new VizePluginPlanError(
      "entry-invalid",
      where,
      "nested arrays and promises are not plugin entries; list resolved entries",
    );
  const descriptor = entry[PLUGIN_ENTRY];
  if (descriptor !== undefined) {
    if (descriptor?.apiVersion !== 2 || typeof descriptor.manifest !== "object")
      throw new VizePluginPlanError(
        "api-unsupported",
        where,
        `plugin API ${descriptor?.apiVersion} is not supported (supported: ${PLUGIN_REGISTRY.apiVersions.join(", ")})`,
      );
    return {
      manifest: descriptor.manifest,
      handlers: Array.isArray(descriptor.callbacks) ? descriptor.callbacks : [],
    };
  }
  return lowerV1Plugin(entry, where);
}

// The v1 adapter records v1 metadata only. Execution keeps the original
// plugin object, so v1 batches, reports and cache keys stay byte-identical.
function lowerV1Plugin(plugin, where) {
  if (
    typeof plugin.run !== "function" ||
    typeof plugin.fingerprint !== "string" ||
    plugin.rules === null ||
    typeof plugin.rules !== "object" ||
    "family" in plugin ||
    "provides" in plugin
  )
    throw new VizePluginPlanError(
      "entry-invalid",
      where,
      "expected a defineVizePlugin entry or a v1 definePlugin rule plugin; other v1 hooks keep their native APIs",
    );
  return {
    manifest: {
      apiVersion: 1,
      backend: "js",
      cacheInputs: plugin.cacheInputs === undefined ? null : plugin.cacheInputs,
      capabilities: [
        {
          data: {
            demands: [...(plugin.demands ?? [])],
            rules: Object.keys(plugin.rules),
            visit: plugin.visit === undefined ? null : [...plugin.visit],
          },
          id: "lint.plugin-v1",
          required: true,
          schemaVersion: 1,
        },
      ],
      fingerprint: plugin.fingerprint,
      name: plugin.name,
      stability: "v1",
      version: plugin.version,
    },
    handlers: [{ run: plugin.run }],
  };
}

function validateManifest(authored, index, runtime) {
  const where = `vue.plugins[${index}]`;
  if (authored === null || typeof authored !== "object" || Array.isArray(authored))
    throw new VizePluginPlanError("manifest-invalid", where, "the manifest must be an object");
  const { options = {}, ...rest } = authored;
  const manifest = {
    ...canonicalData(rest, where, "manifest", "manifest-invalid"),
    options: canonicalData(
      options,
      typeof authored.name === "string" ? authored.name : where,
      "options",
      "options-invalid",
    ),
  };
  for (const key of Object.keys(manifest)) {
    if (!MANIFEST_FIELDS.includes(key))
      throw new VizePluginPlanError("manifest-invalid", where, `unknown manifest field \`${key}\``);
  }
  const name = manifest.name;
  if (typeof name !== "string" || !NAME.test(name))
    throw new VizePluginPlanError("manifest-invalid", where, "name must be a package-style name");
  const fail = (code, message) => {
    throw new VizePluginPlanError(code, name, message);
  };
  const apiVersion = manifest.apiVersion;
  if (!PLUGIN_REGISTRY.apiVersions.includes(apiVersion))
    fail("api-unsupported", `plugin API ${apiVersion} is not supported`);
  if (apiVersion === 2 && manifest.stability !== "experimental")
    fail("manifest-invalid", 'API v2 plugins must declare stability "experimental"');
  if (typeof manifest.version !== "string" || manifest.version === "")
    fail("manifest-invalid", "version must be a non-empty string");
  if (typeof manifest.fingerprint !== "string" || manifest.fingerprint === "")
    fail("manifest-invalid", "fingerprint must be a non-empty string");
  if (apiVersion === 2 && !FINGERPRINT.test(manifest.fingerprint))
    fail("manifest-invalid", "fingerprint must be a sha256 hex digest");
  const backend = manifest.backend ?? "js";
  if (!PLUGIN_REGISTRY.backends.includes(backend))
    fail("manifest-invalid", `unknown backend \`${backend}\``);
  if (!PLUGIN_REGISTRY.admittedBackends.includes(backend))
    fail(
      "backend-unavailable",
      `the ${backend} backend is not available in plugin API v2 incubation; select an implemented backend explicitly (no fallback is attempted)`,
    );
  const supports = supportsOf(manifest.supports ?? {}, fail);
  if (!supports.runtimes.includes(runtime))
    fail("host-unsupported", `does not support the ${runtime} runtime`);
  const plugin = {
    after: namesOf(manifest.after ?? [], "after", fail),
    apiVersion,
    backend,
    before: namesOf(manifest.before ?? [], "before", fail),
    cacheInputs:
      apiVersion === 1 && manifest.cacheInputs === null
        ? null
        : cacheInputsOf(manifest.cacheInputs ?? [], fail),
    capabilities: [],
    fingerprint: manifest.fingerprint,
    index,
    name,
    options: manifest.options,
    permissions: permissionsOf(manifest.permissions ?? [], fail),
    skipped: [],
    stability: manifest.stability,
    supports,
    version: manifest.version,
  };
  if (apiVersion === 1) delete plugin.supports;
  if (!Array.isArray(manifest.capabilities) || manifest.capabilities.length === 0)
    fail("manifest-invalid", "capabilities must be a non-empty array");
  manifest.capabilities.forEach((capability, source) => {
    const at = `capabilities[${source}]`;
    if (capability === null || typeof capability !== "object" || Array.isArray(capability))
      fail("manifest-invalid", `${at} must be an object`);
    for (const key of Object.keys(capability)) {
      if (!["data", "id", "required", "schemaVersion"].includes(key))
        fail("manifest-invalid", `${at} has unknown field \`${key}\``);
    }
    const { id, required = true, schemaVersion } = capability;
    if (typeof id !== "string" || typeof required !== "boolean" || !Number.isInteger(schemaVersion))
      fail(
        "manifest-invalid",
        `${at} needs a string id, integer schemaVersion and boolean required`,
      );
    const known = PLUGIN_REGISTRY.capabilities[id];
    const unsupported =
      known === undefined
        ? `unknown capability \`${id}\``
        : !known.apiVersions.includes(apiVersion)
          ? `capability \`${id}\` is not available to plugin API ${apiVersion}`
          : !known.schemaVersions.includes(schemaVersion)
            ? `capability \`${id}\` schema ${schemaVersion} is not supported (supported: ${known.schemaVersions.join(", ")})`
            : undefined;
    if (unsupported !== undefined) {
      if (required)
        fail(known === undefined ? "capability-unknown" : "capability-unsupported", unsupported);
      plugin.skipped.push({ id, schemaVersion });
      return;
    }
    const { key, data } = normalizeCapability(plugin, { id, data: capability.data ?? {} });
    plugin.capabilities.push({ data, id, key, required, schemaVersion, source });
  });
  if (plugin.capabilities.length === 0)
    fail("manifest-invalid", "no supported capability remains after skipping optional ones");
  return plugin;
}

function supportsOf(value, fail) {
  if (value === null || typeof value !== "object" || Array.isArray(value))
    fail("manifest-invalid", "supports must be an object");
  const result = {};
  for (const [key, allowed] of Object.entries(SUPPORTS)) {
    const list = value[key] ?? allowed;
    if (!Array.isArray(list) || list.length === 0 || new Set(list).size !== list.length)
      fail("manifest-invalid", `supports.${key} must be a non-empty unique array`);
    for (const item of list) {
      if (!allowed.includes(item))
        fail("manifest-invalid", `supports.${key} has unknown \`${item}\``);
    }
    result[key] = [...list].sort(compareCodePoints);
  }
  for (const key of Object.keys(value)) {
    if (!Object.hasOwn(SUPPORTS, key))
      fail("manifest-invalid", `supports has unknown field \`${key}\``);
  }
  return result;
}

function namesOf(value, field, fail) {
  if (!Array.isArray(value) || value.some((name) => typeof name !== "string" || !NAME.test(name)))
    fail("manifest-invalid", `${field} must list plugin names`);
  if (new Set(value).size !== value.length) fail("manifest-invalid", `${field} has duplicates`);
  return [...value].sort(compareCodePoints);
}

function cacheInputsOf(value, fail) {
  if (!Array.isArray(value)) fail("manifest-invalid", "cacheInputs must be an array");
  const names = new Set();
  for (const input of value) {
    if (
      input === null ||
      typeof input !== "object" ||
      Object.keys(input).sort(compareCodePoints).join() !== "name,value" ||
      typeof input.name !== "string" ||
      typeof input.value !== "string" ||
      input.name === ""
    )
      fail("manifest-invalid", "cacheInputs entries are { name, value } strings");
    if (input.name.startsWith("@vize/"))
      fail(
        "manifest-invalid",
        `cache input \`${input.name}\` uses the host-reserved @vize/ prefix`,
      );
    if (names.has(input.name))
      fail("manifest-invalid", `cache input \`${input.name}\` is duplicated`);
    names.add(input.name);
  }
  return [...value].sort((left, right) => compareCodePoints(left.name, right.name));
}

function permissionsOf(value, fail) {
  if (!Array.isArray(value)) fail("manifest-invalid", "permissions must be an array");
  const kinds = new Set();
  const result = value.map((permission) => {
    const field = PERMISSIONS[permission?.kind];
    if (field === undefined) fail("manifest-invalid", `unknown permission \`${permission?.kind}\``);
    if (
      Object.keys(permission).sort(compareCodePoints).join() !==
        ["kind", field].sort(compareCodePoints).join() ||
      !Array.isArray(permission[field]) ||
      permission[field].length === 0 ||
      permission[field].some((item) => typeof item !== "string" || item === "")
    )
      fail("manifest-invalid", `the ${permission.kind} permission lists non-empty ${field}`);
    if (kinds.has(permission.kind))
      fail("manifest-invalid", `the ${permission.kind} permission is declared twice`);
    kinds.add(permission.kind);
    return {
      kind: permission.kind,
      [field]: [...new Set(permission[field])].sort(compareCodePoints),
    };
  });
  return result.sort((left, right) => compareCodePoints(left.kind, right.kind));
}

// Stable topological order: among ready plugins the authored config order wins.
function orderPlugins(plugins) {
  const byName = new Map();
  for (const plugin of plugins) {
    if (byName.has(plugin.name))
      throw new VizePluginPlanError(
        "duplicate-name",
        plugin.name,
        `listed at vue.plugins[${byName.get(plugin.name).index}] and vue.plugins[${plugin.index}]`,
      );
    byName.set(plugin.name, plugin);
  }
  const edges = new Map(plugins.map((plugin) => [plugin.name, new Set()]));
  const incoming = new Map(plugins.map((plugin) => [plugin.name, 0]));
  const edge = (from, to, plugin, field) => {
    if (!byName.has(from === plugin.name ? to : from))
      throw new VizePluginPlanError(
        "order-unknown",
        plugin.name,
        `${field} names \`${from === plugin.name ? to : from}\`, which is not in vue.plugins`,
      );
    if (from === to) throw new VizePluginPlanError("order-cycle", plugin.name, "orders itself");
    if (!edges.get(from).has(to)) {
      edges.get(from).add(to);
      incoming.set(to, incoming.get(to) + 1);
    }
  };
  for (const plugin of plugins) {
    for (const name of plugin.before) edge(plugin.name, name, plugin, "before");
    for (const name of plugin.after) edge(name, plugin.name, plugin, "after");
  }
  const ordered = [];
  const ready = plugins.filter((plugin) => incoming.get(plugin.name) === 0);
  while (ready.length > 0) {
    ready.sort((left, right) => left.index - right.index);
    const next = ready.shift();
    ordered.push(next);
    for (const name of edges.get(next.name)) {
      incoming.set(name, incoming.get(name) - 1);
      if (incoming.get(name) === 0) ready.push(byName.get(name));
    }
  }
  if (ordered.length !== plugins.length) {
    const cycle = plugins
      .filter((plugin) => incoming.get(plugin.name) > 0)
      .map((plugin) => plugin.name);
    throw new VizePluginPlanError(
      "order-cycle",
      cycle[0],
      `ordering cycle among ${cycle.join(", ")}`,
    );
  }
  return ordered;
}
