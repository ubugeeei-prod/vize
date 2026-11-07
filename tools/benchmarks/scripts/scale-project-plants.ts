/**
 * What each checker lane must report on a generated scale project's planted
 * errors: the codes a plant may raise, and per-lane counts where a lane
 * differs from one. A zero pins a known miss to its issue: the lane's other
 * plants still prove it checked the project, and the fix fails the gate until
 * the entry is dropped.
 */

import type { PlantKind } from "./scale-project-sfc.ts";

export type Plant = { kind: PlantKind; file: string; line: number; codes: readonly string[] };
export type PlantExpectation = Plant & { count: number };

/** Codes each plant may raise; TS2769 is the overload form of an emit call mismatch. */
export const PLANT_CODES: Record<PlantKind, readonly string[]> = {
  script: ["TS2322"],
  "emit-script": ["TS2345", "TS2769"],
  generic: ["TS2322"],
  slot: ["TS2339"],
  "emit-template": ["TS2322", "TS2345"],
  "v-model": ["TS2322"],
  global: ["TS2322"],
  "listener-arrow": ["TS2322"],
  "listener-prop": ["TS2322"],
  "listener-statement": ["TS2345"],
};

/** Errors per plant where every lane reports more than one: the prop and the handler. */
const DEFAULT_COUNTS: Partial<Record<PlantKind, number>> = { "listener-prop": 2 };

// vue-tsc and Vue's mapper do not check an inline handler's annotated parameters.
const VUE_LANGUAGE_TOOLS: Partial<Record<PlantKind, number>> = {
  "listener-arrow": 0,
  "listener-prop": 1,
};

/** Errors per plant where a lane differs from the default, keyed by lane id. */
export const PLANT_MULTIPLICITY: Record<string, Partial<Record<PlantKind, number>>> = {
  "vue-tsc": VUE_LANGUAGE_TOOLS,
  "tsc-vue-mapper": VUE_LANGUAGE_TOOLS,
};
export const PLANT_MULTIPLICITY_NOTES: readonly string[] = [
  "vue-tsc and Vue's mapper do not check an inline handler's annotated parameters, so they expect no handler error on the listener plants.",
];

/** The errors `lane` must report: every plant with its count for that lane. */
export function expectedPlants(plants: readonly Plant[], lane: string): PlantExpectation[] {
  const counts = PLANT_MULTIPLICITY[lane] ?? {};
  return plants.map((plant) => ({
    ...plant,
    count: counts[plant.kind] ?? DEFAULT_COUNTS[plant.kind] ?? 1,
  }));
}
