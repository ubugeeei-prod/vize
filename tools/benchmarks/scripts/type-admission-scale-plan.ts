import { scaleDefaults } from "./type-admission-scale.ts";
import type { ScaleOptions } from "./type-admission-scale-types.ts";

export type ScaleAxis = "smoke" | "files" | "graph" | "file-size" | "types" | "mixed";
export type ScaleCase = { name: string; options: ScaleOptions };

/** Separate axes avoid attributing every growth problem to file count. */
export function admissionScalePlan(axis: ScaleAxis, sfcs: number): ScaleCase[] {
  const make = (name: string, options: Partial<ScaleOptions>): ScaleCase => ({
    name,
    options: { ...scaleDefaults, ...options },
  });
  switch (axis) {
    case "smoke":
      return [make("smoke", {})];
    case "files":
      return [make(`files-${sfcs}`, { sfcs, allowLarge: true })];
    case "graph":
      return (["chain", "diamond", "scc", "crosspkg"] as const).map((shape) =>
        make(`graph-${shape}`, { graphNodes: 1_000, shape }),
      );
    case "file-size":
      return [128 * 1024, 1024 * 1024, 10 * 1024 * 1024].map((sfcBytes) =>
        make(`file-size-${sfcBytes}`, { sfcs: 1, sfcBytes }),
      );
    case "types":
      return [
        ...[4, 16, 64].map((typeWidth) => make(`type-width-${typeWidth}`, { typeWidth })),
        ...[16, 32].map((typeDepth) => make(`type-depth-${typeDepth}`, { typeDepth })),
      ];
    case "mixed":
      return [
        make("mixed", {
          sfcs: 32,
          sfcBytes: 128 * 1024,
          graphNodes: 256,
          shape: "crosspkg",
          typeWidth: 16,
          typeDepth: 16,
        }),
      ];
    default:
      throw new Error("unknown scale axis: " + axis);
  }
}
