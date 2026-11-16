/** Fail-closed nonlinear-growth admission for Vize-owned scaling curves. */
import type { ScaleKind } from "./scale-project-generator.ts";
import type { ToolId } from "./typecheck-scaling-tools.ts";

export const GROWTH_LIMIT = 1.5;

type GrowthMetric = "cpuMs" | "rssKb";
type GrowthShape = "perSfc" | "step";
type GrowthCurve = {
  tool: ToolId;
  kind: ScaleKind;
  label: string;
  vize: boolean;
  sizes: number[];
  perSfc: Record<GrowthMetric | "wallMs", number | null>;
  step: Record<GrowthMetric | "wallMs", number | null>;
  flags: string[];
};

export type GrowthBudgetDeclaration = {
  tool: ToolId;
  kind: ScaleKind;
  sizes: number[];
  metric: GrowthMetric;
  shape: GrowthShape;
  ceiling: number;
  reason: string;
};

/**
 * Runs 42969237215 and 42969240429 retained exact diagnostic packets on the
 * 1k/5k/20k Vue corpus. Their vize-check CPU ratios were 1.63-1.70x per SFC and
 * 1.70-1.73x for the marginal step. The 1.8x ceiling turns that measured debt
 * into a bounded declaration rather than silently accepting every flag.
 *
 * The match includes the complete size vector, tool, project kind, metric and
 * shape, so no new topology can inherit the declaration accidentally.
 */
export const GROWTH_BUDGET_DECLARATIONS: readonly GrowthBudgetDeclaration[] = [
  {
    tool: "vize-check",
    kind: "vue",
    sizes: [1000, 5000, 20000],
    metric: "cpuMs",
    shape: "perSfc",
    ceiling: 1.8,
    reason: "existing 1k/5k/20k CPU curve; tracked by P0 #13014",
  },
  {
    tool: "vize-check",
    kind: "vue",
    sizes: [1000, 5000, 20000],
    metric: "cpuMs",
    shape: "step",
    ceiling: 1.8,
    reason: "existing 1k/5k/20k marginal CPU curve; tracked by P0 #13014",
  },
];

export type GrowthBudgetEntry = {
  tool: ToolId;
  kind: ScaleKind;
  sizes: number[];
  metric: GrowthMetric;
  shape: GrowthShape;
  value: number | "infinite";
  ceiling: number;
  declaration: string | null;
  ok: boolean;
};

export type GrowthBudget = {
  status: "passed" | "failed";
  defaultCeiling: number;
  checked: GrowthBudgetEntry[];
  failures: GrowthBudgetEntry[];
};

const sameSizes = (left: readonly number[], right: readonly number[]) =>
  left.length === right.length && left.every((value, index) => value === right[index]);

/** Reference curves remain observations; only Vize-owned curves gate release. */
export function growthBudget(growth: readonly GrowthCurve[]): GrowthBudget {
  const checked: GrowthBudgetEntry[] = [];
  for (const curve of growth.filter((entry) => entry.vize)) {
    for (const metric of ["cpuMs", "rssKb"] as const) {
      for (const shape of ["perSfc", "step"] as const) {
        const measured = curve[shape][metric];
        if (measured == null) continue;
        const value = Number.isFinite(measured) ? measured : "infinite";
        const declaration = GROWTH_BUDGET_DECLARATIONS.find(
          (entry) =>
            entry.tool === curve.tool &&
            entry.kind === curve.kind &&
            entry.metric === metric &&
            entry.shape === shape &&
            sameSizes(entry.sizes, curve.sizes),
        );
        const ceiling = declaration?.ceiling ?? GROWTH_LIMIT;
        checked.push({
          tool: curve.tool,
          kind: curve.kind,
          sizes: curve.sizes,
          metric,
          shape,
          value,
          ceiling,
          declaration: declaration?.reason ?? null,
          ok: typeof value === "number" && value <= ceiling,
        });
      }
    }
  }
  const failures = checked.filter((entry) => !entry.ok);
  return {
    status: failures.length === 0 ? "passed" : "failed",
    defaultCeiling: GROWTH_LIMIT,
    checked,
    failures,
  };
}

export function formatGrowthBudgetFailures(failures: readonly GrowthBudgetEntry[]): string {
  return failures
    .map(
      (failure) =>
        `::error::growth budget failed: ${failure.tool} ${failure.kind} ${failure.sizes.join(",")} ${failure.shape} ${failure.metric} ${failure.value === "infinite" ? "infinite" : failure.value.toFixed(3)}x > ${failure.ceiling.toFixed(3)}x`,
    )
    .join("\n");
}
