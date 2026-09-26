// Authoring API for serialized L2 visits and native compiler hooks.
// See README.md for the runtime and trust boundaries.
import { createHash } from "node:crypto";
import { SDK_FINGERPRINT } from "./identity.js";

/** Every batch this SDK reads is at this wire version. */
export const BATCH_SCHEMA = 1;

/**
 * A plugin: static manifest (`visit` kinds, `demands` fact groups) plus
 * rules `(ctx) => void`. `run` is what the native host calls, once per
 * document: batch JSON in, report JSON out.
 */
export function definePlugin({ name, version, visit, demands = [], cacheInputs, rules }) {
  if (
    !name ||
    !version ||
    !rules ||
    Object.values(rules).some((rule) => typeof rule !== "function")
  ) {
    throw new TypeError("a plugin needs name, version and synchronous rule functions");
  }
  const code = Object.entries(rules).map(([id, rule]) => [id, String(rule)]);
  const fingerprint = createHash("sha256")
    .update(JSON.stringify([SDK_FINGERPRINT, name, version, visit ?? null, demands, code]))
    .digest("hex");
  const plugin = {
    name,
    version,
    fingerprint,
    visit: visit && Object.freeze([...visit]),
    demands: Object.freeze([...demands]),
    cacheInputs:
      cacheInputs && Object.freeze(cacheInputs.map((input) => Object.freeze({ ...input }))),
    rules: Object.freeze({ ...rules }),
  };
  plugin.run = (batchJson) => JSON.stringify(runBatch(plugin, JSON.parse(batchJson)));
  return Object.freeze(plugin);
}

/** Run every rule over one parsed batch; returns the report array. */
export function runBatch(plugin, batch) {
  if (batch.schema !== BATCH_SCHEMA) {
    throw new Error(`${plugin.name}: batch schema ${batch.schema}, SDK reads ${BATCH_SCHEMA}`);
  }
  freeze(batch);
  const byId = new Map(batch.nodes.map((node) => [node.id, node]));
  const facts = (name) => new Map(batch.facts[name]);
  const parent = (id) => batch.parents[id];
  return runRules(plugin, { nodes: batch.nodes, byId, facts, parent });
}

export function runRules(plugin, source) {
  const reports = [];
  for (const [rule, check] of Object.entries(plugin.rules)) {
    const returned = check(context(plugin, rule, source, reports));
    if (returned?.then)
      throw new TypeError(`${plugin.name}/${rule}: asynchronous rules are unsupported`);
  }
  return reports;
}

function context(plugin, rule, { nodes, byId, facts, parent }, reports) {
  const where = `${plugin.name}/${rule}`;
  const visits = (kind) => !plugin.visit || plugin.visit.includes(kind);
  return Object.freeze({
    nodes,
    facts(name) {
      if (!plugin.demands.includes(name)) {
        throw new Error(`${where}: fact group \`${name}\` was not declared in demands`);
      }
      return facts(name);
    },
    *ancestors(node, kind) {
      if (!visits(kind)) {
        throw new Error(`${where}: ancestors of kind ${kind} need ${kind} in visit`);
      }
      for (let id = parent(node.id); id !== -1; id = parent(id)) {
        const found = byId.get(id);
        if (found && found.kind === kind) yield found;
      }
    },
    report(node, message, fix) {
      if (!byId.has(node.id)) throw new Error(`${where}: report needs a visited node`);
      if (typeof message !== "string" || (fix !== undefined && typeof fix !== "string")) {
        throw new TypeError(`${where}: message and optional fix must be strings`);
      }
      reports.push({ rule, node: node.id, message, ...(fix === undefined ? {} : { fix }) });
    },
  });
}

function freeze(value) {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value)) freeze(child);
    Object.freeze(value);
  }
}

export { defineFactProvider, defineOutputPlugin, defineTransformPlugin } from "./stages.js";
export { applyFixes } from "./fixes.js";
