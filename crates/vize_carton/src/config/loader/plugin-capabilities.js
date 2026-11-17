// Capability data schemas for plugin plan schema 1. Each normalizer returns the
// canonical data and its ownership key, or fails closed.
import { VizePluginPlanError, compareCodePoints } from "./plugin-canonical.js";
import registry from "./plugin-registry.json" with { type: "json" };

export const PLUGIN_REGISTRY = registry;

const KEBAB = /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/;
const MESSAGE_ID = /^[A-Za-z][A-Za-z0-9_-]*$/;
const LEVELS = ["off", "warn", "error"];
const FIXABLE = ["none", "safe", "unsafe"];

/** Normalize one capability's data; `data` is already canonical JSON. */
export function normalizeCapability(plugin, capability) {
  const where = `${plugin.name} ${capability.id}`;
  const normalize = NORMALIZERS[capability.id];
  return normalize(plugin, capability.data, where);
}

const NORMALIZERS = {
  "lint.plugin-v1"(plugin, data, where) {
    fields(data, where, ["demands", "rules", "visit"], ["demands", "rules", "visit"]);
    const visit =
      data.visit === null ? null : kinds(data.visit, `${where} visit`, { allowEmpty: true });
    const rules = strings(data.rules, `${where} rules`, { unique: true });
    if (rules.length === 0) invalid(where, "a v1 plugin needs at least one rule");
    return {
      key: plugin.name,
      data: { demands: facts(data.demands, `${where} demands`), rules, visit },
    };
  },
  "lint.rule"(plugin, data, where) {
    fields(
      data,
      where,
      ["defaultLevel", "demands", "docs", "documents", "fixable", "id", "messages", "visit"],
      ["id", "messages", "visit"],
    );
    const id = kebab(data.id, `${where} id`);
    const messages = record(data.messages, `${where} messages`);
    const ids = Object.keys(messages);
    if (ids.length === 0) invalid(where, "messages must declare at least one message id");
    for (const messageId of ids) {
      if (!MESSAGE_ID.test(messageId)) invalid(where, `message id \`${messageId}\` is invalid`);
      if (typeof messages[messageId] !== "string" || messages[messageId] === "")
        invalid(where, `message \`${messageId}\` must be a non-empty string`);
    }
    const documents = strings(data.documents ?? ["template"], `${where} documents`, {
      unique: true,
      sorted: true,
    });
    for (const document of documents) {
      if (!registry.lintDocuments.includes(document))
        invalid(where, `document domain \`${document}\` is not available in schema 1`);
    }
    if (documents.length === 0) invalid(where, "documents must not be empty");
    return {
      key: `${plugin.name}/${id}`,
      data: {
        defaultLevel: oneOf(data.defaultLevel ?? "off", LEVELS, `${where} defaultLevel`),
        demands: facts(data.demands ?? [], `${where} demands`),
        docs: text(data.docs ?? "", `${where} docs`),
        documents,
        fixable: oneOf(data.fixable ?? "none", FIXABLE, `${where} fixable`),
        id,
        messages,
        visit: kinds(data.visit, `${where} visit`, { allowEmpty: false }),
      },
    };
  },
  "sfc.custom-block"(_plugin, data, where) {
    fields(data, where, ["lang", "type"], ["type"]);
    const type = kebab(data.type, `${where} type`);
    if (["template", "script", "style"].includes(type))
      invalid(where, `\`${type}\` is a core SFC block`);
    const lang = data.lang === undefined ? undefined : kebab(data.lang, `${where} lang`);
    return {
      key: lang === undefined ? type : `${type}:${lang}`,
      data: lang === undefined ? { type } : { lang, type },
    };
  },
  "vue.directive"(_plugin, data, where) {
    fields(data, where, ["modifiers", "name"], ["name"]);
    const name = kebab(data.name, `${where} name`);
    if (registry.reservedDirectives.includes(name))
      invalid(where, `v-${name} is a built-in Vue directive`);
    const modifiers = {};
    for (const [modifier, contract] of Object.entries(
      record(data.modifiers ?? {}, `${where} modifiers`),
    )) {
      const at = `${where} modifier \`${modifier}\``;
      if (!/^[A-Za-z][A-Za-z0-9_-]*$/.test(modifier)) invalid(at, "has an invalid name");
      fields(record(contract, at), at, ["default", "optional"], []);
      const optional = boolean(contract.optional ?? true, `${at} optional`);
      if (!optional && contract.default !== undefined)
        invalid(at, "is required and cannot declare a default");
      modifiers[modifier] = {
        default: boolean(contract.default ?? false, `${at} default`),
        optional,
      };
    }
    return { key: name, data: { modifiers, name } };
  },
};

export function claimKey(capabilityId, key) {
  return `${capabilityId}:${key}`;
}

/**
 * Exclusive owners. Exact keys conflict; a custom block claimed for every
 * language also conflicts with the same block claimed for one language.
 */
export function ownershipConflict(claims) {
  const owners = new Map();
  for (const claim of claims) {
    const previous = owners.get(claim.key);
    if (previous) return [previous, claim];
    owners.set(claim.key, claim);
  }
  const blocks = new Map();
  for (const claim of claims) {
    const match = /^sfc\.custom-block:([^:]+)(?::(.+))?$/.exec(claim.key);
    if (!match) continue;
    const entry = blocks.get(match[1]) ?? { any: undefined, langs: [] };
    if (match[2] === undefined) entry.any = claim;
    else entry.langs.push(claim);
    blocks.set(match[1], entry);
    if (entry.any && entry.langs.length > 0) return [entry.any, entry.langs[0]];
  }
  return undefined;
}

function fields(data, where, known, required) {
  record(data, where);
  for (const key of Object.keys(data)) {
    if (!known.includes(key)) invalid(where, `unknown field \`${key}\``);
  }
  for (const key of required) {
    if (data[key] === undefined) invalid(where, `missing field \`${key}\``);
  }
}

function record(value, where) {
  if (value === null || typeof value !== "object" || Array.isArray(value))
    invalid(where, "must be an object");
  return value;
}

function strings(value, where, { unique = false, sorted = false } = {}) {
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string" || entry === ""))
    invalid(where, "must be an array of non-empty strings");
  if (unique && new Set(value).size !== value.length) invalid(where, "contains duplicates");
  return sorted ? [...value].sort(compareCodePoints) : value;
}

function kinds(value, where, { allowEmpty }) {
  const list = strings(value, where, { unique: true, sorted: true });
  if (!allowEmpty && list.length === 0) invalid(where, "must visit at least one record kind");
  for (const kind of list) {
    if (!registry.recordKinds.includes(kind)) invalid(where, `unknown record kind \`${kind}\``);
  }
  return list;
}

function facts(value, where) {
  const list = strings(value, where, { unique: true, sorted: true });
  for (const fact of list) {
    if (!registry.factGroups.includes(fact)) invalid(where, `unknown fact group \`${fact}\``);
  }
  return list;
}

function kebab(value, where) {
  if (typeof value !== "string" || !KEBAB.test(value)) invalid(where, "must be kebab-case");
  return value;
}

function text(value, where) {
  if (typeof value !== "string") invalid(where, "must be a string");
  return value;
}

function boolean(value, where) {
  if (typeof value !== "boolean") invalid(where, "must be a boolean");
  return value;
}

function oneOf(value, allowed, where) {
  if (!allowed.includes(value)) invalid(where, `must be one of ${allowed.join(", ")}`);
  return value;
}

function invalid(where, message) {
  throw new VizePluginPlanError("capability-invalid", where, message);
}
