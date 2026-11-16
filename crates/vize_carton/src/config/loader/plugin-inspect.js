// Read-only views over a resolved plugin plan: admission and inspection.
import { PLUGIN_REGISTRY } from "./plugin-capabilities.js";
import {
  VizePluginPlanError,
  canonicalData,
  canonicalJson,
  deepFreeze,
  digestOf,
} from "./plugin-canonical.js";

/**
 * Capabilities addressed to `product` that it does not implement. A product
 * that receives any of them must refuse the plan instead of ignoring them.
 */
export function pluginPlanRefusals(plan, product, { output } = {}) {
  if (!PLUGIN_REGISTRY.products.includes(product))
    throw new VizePluginPlanError("product-unknown", product, "is not a Vize product");
  const refused = [];
  for (const plugin of plan.plugins) {
    for (const capability of plugin.capabilities) {
      const known = PLUGIN_REGISTRY.capabilities[capability.id];
      if (!known.consumers.includes(product)) continue;
      if (!known.admittedBy.includes(product)) {
        refused.push({
          capability: capability.id,
          key: capability.key,
          plugin: plugin.name,
          reason: "unadmitted",
        });
      } else if (
        output !== undefined &&
        plugin.supports &&
        !plugin.supports.outputs.includes(output)
      ) {
        refused.push({
          capability: capability.id,
          key: capability.key,
          plugin: plugin.name,
          reason: `output:${output}`,
        });
      }
    }
  }
  return refused;
}

/** Fail closed when `product` cannot honor every capability addressed to it. */
export function admitPluginPlan(plan, product, options) {
  const [first, ...rest] = pluginPlanRefusals(plan, product, options);
  if (first === undefined) return;
  const more = rest.length === 0 ? "" : ` (and ${rest.length} more)`;
  throw new VizePluginPlanError(
    first.reason === "unadmitted" ? "capability-unadmitted" : "host-unsupported",
    first.plugin,
    first.reason === "unadmitted"
      ? `${product} does not implement ${first.capability} \`${first.key}\` yet${more}`
      : `${first.capability} \`${first.key}\` does not support the ${first.reason.slice(7)} output${more}`,
  );
}

/** Re-read canonical plan bytes; non-canonical or stale bytes fail closed. */
export function verifyPluginPlan(json, expectedDigest) {
  let parsed;
  try {
    parsed = JSON.parse(json);
  } catch {
    throw new VizePluginPlanError("plan-invalid", "plan", "is not JSON");
  }
  const canonical = canonicalJson(canonicalData(parsed, "plan", "plan", "plan-invalid"));
  if (canonical !== json)
    throw new VizePluginPlanError("plan-invalid", "plan", "is not in canonical form");
  if (parsed.planSchema !== PLUGIN_REGISTRY.planSchema)
    throw new VizePluginPlanError(
      "plan-invalid",
      "plan",
      `schema ${parsed.planSchema} is not supported (supported: ${PLUGIN_REGISTRY.planSchema})`,
    );
  const digest = digestOf(json);
  if (expectedDigest !== undefined && digest !== expectedDigest)
    throw new VizePluginPlanError(
      "plan-stale",
      "plan",
      `digest ${digest} does not match the resolved plan ${expectedDigest}; reload vite.config.*`,
    );
  return deepFreeze(parsed);
}

/** The `vize plugin inspect --json` view: the plan plus per-product admission. */
export function inspectPluginPlan({ plan, digest }) {
  return deepFreeze({
    admission: Object.fromEntries(
      PLUGIN_REGISTRY.products.map((product) => {
        const refused = pluginPlanRefusals(plan, product);
        const used = plan.plugins.some((plugin) =>
          plugin.capabilities.some((capability) =>
            PLUGIN_REGISTRY.capabilities[capability.id].consumers.includes(product),
          ),
        );
        return [
          product,
          { refused, status: refused.length ? "refused" : used ? "ready" : "unused" },
        ];
      }),
    ),
    claims: plan.claims,
    digest,
    order: plan.plugins.map((plugin) => plugin.name),
    planSchema: plan.planSchema,
    plugins: plan.plugins,
  });
}
