import type { PluginPlan, ResolvedPluginPlan } from "./plugin-plan.js";

export interface PluginRefusal {
  readonly capability: string;
  readonly key: string;
  readonly plugin: string;
  /** `unadmitted`, or `output:<name>`. */
  readonly reason: string;
}

export interface PluginPlanInspection {
  readonly admission: Readonly<
    Record<
      string,
      {
        readonly refused: readonly PluginRefusal[];
        readonly status: "ready" | "refused" | "unused";
      }
    >
  >;
  readonly claims: PluginPlan["claims"];
  readonly digest: string;
  readonly order: readonly string[];
  readonly planSchema: number;
  readonly plugins: PluginPlan["plugins"];
}

export declare function pluginPlanRefusals(
  plan: PluginPlan,
  product: string,
  options?: { output?: string },
): PluginRefusal[];

export declare function admitPluginPlan(
  plan: PluginPlan,
  product: string,
  options?: { output?: string },
): void;

export declare function verifyPluginPlan(json: string, expectedDigest?: string): PluginPlan;

export declare function inspectPluginPlan(
  resolved: Pick<ResolvedPluginPlan, "plan" | "digest">,
): PluginPlanInspection;
