// The compatibility adapter for the measured P4-16 proxy arm.
// Rules and serialized execution use the actual author package.
import { runRules } from "../../../../npm/plugin-sdk/index.js";
export { BATCH_SCHEMA, definePlugin, runBatch } from "../../../../npm/plugin-sdk/index.js";

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
