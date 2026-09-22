// The P4-16 JS plugin SDK spike: `definePlugin` and the rule context over
// one serialized S2 visit batch (the decided shape), plus the proxy-arm
// context the spike measured against. GA is P6-7 (`@vizejs/plugin-sdk`).
import { createHash } from "node:crypto";

/** Every batch this SDK reads is at this wire version. */
export const BATCH_SCHEMA = 1;

/**
 * A plugin: static manifest (`visit` kinds, `demands` fact groups) plus
 * rules `(ctx) => void`. `run` is what the native host calls, once per
 * document: batch JSON in, report JSON out.
 */
export function definePlugin({ name, version, visit, demands = [], rules }) {
  const code = Object.entries(rules).map(([id, rule]) => [id, String(rule)]);
  const fingerprint = createHash("sha256")
    .update(JSON.stringify([name, version, visit ?? null, demands, code]))
    .digest("hex");
  const plugin = { name, version, fingerprint, visit, demands, rules };
  plugin.run = (batchJson) => JSON.stringify(runBatch(plugin, JSON.parse(batchJson)));
  return plugin;
}

/** Run every rule over one parsed batch; returns the report array. */
export function runBatch(plugin, batch) {
  if (batch.schema !== BATCH_SCHEMA) {
    throw new Error(`${plugin.name}: batch schema ${batch.schema}, SDK reads ${BATCH_SCHEMA}`);
  }
  const byId = new Map(batch.nodes.map((node) => [node.id, node]));
  const facts = (name) => new Map(batch.facts[name]);
  const parent = (id) => batch.parents[id];
  return runRules(plugin, { nodes: batch.nodes, byId, facts, parent });
}

/** The proxy arm: the same rules over a native handle, one napi call per read. */
export function runProxy(plugin, handle) {
  const node = (id) =>
    new Proxy(
      { id },
      {
        get(target, key) {
          if (key === "id") return target.id;
          if (key === "kind") return handle.kind(target.id);
          if (key === "alias") return aliasProxy(handle, target.id);
          return handle.field(target.id, String(key)) ?? undefined;
        },
      },
    );
  const visit = plugin.visit ? new Set(plugin.visit) : null;
  const nodes = [];
  for (let id = 0; id < handle.count(); id += 1) {
    if (!visit || visit.has(handle.kind(id))) nodes.push(node(id));
  }
  const byId = new Map(nodes.map((each) => [each.id, each]));
  const facts = () => ({ get: (scope) => handle.scope(scope) ?? undefined });
  const parent = (id) => handle.parent(id);
  return runRules(plugin, { nodes, byId, facts, parent });
}

function aliasProxy(handle, id) {
  const value = handle.field(id, "alias.value");
  if (value === null) return undefined;
  return { value, key: handle.field(id, "alias.key"), index: handle.field(id, "alias.index") };
}

function runRules(plugin, source) {
  const reports = [];
  for (const [rule, check] of Object.entries(plugin.rules)) {
    check(context(plugin, rule, source, reports));
  }
  return reports;
}

function context(plugin, rule, { nodes, byId, facts, parent }, reports) {
  const where = `${plugin.name}/${rule}`;
  const visits = (kind) => !plugin.visit || plugin.visit.includes(kind);
  return {
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
    report(node, message) {
      reports.push({ rule, node: node.id, message });
    },
  };
}
