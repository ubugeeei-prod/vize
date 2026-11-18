/**
 * One registry project can hold apps that `vue-tsc` checks with different
 * tsconfigs (vuetify's library and docs map `@/` differently). Each
 * `typecheckPerformance.targets` entry is a typecheck unit with its own
 * tsconfig, corpus, Vize run, baseline and qualification. The first unit keeps
 * the project id, so its artifacts keep their names; later units are
 * `<project>@<target>`. Compiler, lint, glyph and LSP coverage stay per project.
 */

export interface TypecheckTarget {
  readonly id: string;
  readonly tsconfig: string;
  readonly corpusGlobs: readonly string[];
  /** The baseline's own config, when it is generated (a Nuxt `.nuxt` config). */
  readonly baselineTsconfig?: string;
  /** A target whose inputs cannot be prepared offline yet; tracked by `issue`. */
  readonly configurationOnly?: { readonly reason: string; readonly issue: number };
}

interface TargetedProject {
  readonly id: string;
  readonly tsconfig?: unknown;
  readonly typecheckPerformance?: {
    readonly targets?: readonly TypecheckTarget[];
    readonly baseline?: { readonly tsconfig?: unknown; readonly prepare?: unknown };
  } & Record<string, unknown>;
}

export const TARGET_SEPARATOR = "@";

export function typecheckUnits<Project extends TargetedProject>(project: Project): Project[] {
  const performance = project.typecheckPerformance;
  const targets = performance?.targets;
  if (performance == null || targets == null || targets.length === 0) return [project];
  const { targets: _targets, ...shared } = performance;
  return targets.map((target, index) => {
    const unit: Record<string, unknown> = {
      ...shared,
      corpusGlobs: target.corpusGlobs,
    };
    if (performance.baseline != null) {
      unit.baseline = {
        ...performance.baseline,
        tsconfig: target.baselineTsconfig ?? target.tsconfig,
      };
    }
    if (target.configurationOnly != null) unit.configurationOnly = target.configurationOnly;
    return {
      ...project,
      id: index === 0 ? project.id : `${project.id}${TARGET_SEPARATOR}${target.id}`,
      tsconfig: target.tsconfig,
      typecheckPerformance: unit,
      typecheckTarget: { project: project.id, target: target.id },
    };
  });
}

/** Every tsconfig any of the project's typecheck units reads. */
export function typecheckUnitTsconfigs(project: TargetedProject): string[] {
  const paths = new Set<string>();
  for (const unit of typecheckUnits(project)) {
    for (const candidate of [unit.typecheckPerformance?.baseline?.tsconfig, unit.tsconfig]) {
      if (typeof candidate === "string") paths.add(candidate);
    }
  }
  return [...paths];
}
