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
};

/** Errors per plant where a lane differs from one, keyed by lane id. */
export const PLANT_MULTIPLICITY: Record<string, Partial<Record<PlantKind, number>>> = {
  // #11643: the Vize mapper does not check the update:modelValue payload against the bound ref.
  "tsc-vize-mapper": { "v-model": 0 },
  "tsrs-fast-vize-mapper": { "v-model": 0 },
};
export const PLANT_MULTIPLICITY_NOTES: readonly string[] = [
  "The Vize mapper lanes expect no error on the `v-model` plant: the mapper does not check the `update:modelValue` payload against the bound ref (#11643).",
];

/** The errors `lane` must report: every plant with its count for that lane. */
export function expectedPlants(plants: readonly Plant[], lane: string): PlantExpectation[] {
  const counts = PLANT_MULTIPLICITY[lane] ?? {};
  return plants.map((plant) => ({ ...plant, count: counts[plant.kind] ?? 1 }));
}
