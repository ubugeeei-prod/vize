/**
 * Floors of the corpus pair, measured on the head build beside its lanes:
 *
 * - floor-absolute: every SFC projects to `export default {} as any` (host,
 *   lib, .ts and mapper-process cost with no projection to check);
 * - floor-expressions: Vize's head projection with the Vue contract, slot,
 *   event and model typing rewritten to `any` (typecheck-floors/ablate.ts),
 *   served from a recording with zero mapper wait;
 * - floor-replay: Vize's head projection itself, served the same way, so
 *   wall(vize-head) − wall(floor-replay) is what mapper wait costs.
 *
 * Recordings come from one untimed tsc run through record-proxy.ts.
 */

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { ablateResult } from "./typecheck-floors/ablate.ts";
import { transformRecording } from "./typecheck-floors/recording.ts";

const FLOORS_DIR = join(dirname(fileURLToPath(import.meta.url)), "typecheck-floors");

export const FLOOR_VARIANTS = [
  { id: "floor-absolute", label: "absolute floor (export default {} as any)" },
  { id: "floor-expressions", label: "expressions-only, zero mapper wait" },
  { id: "floor-replay", label: "Vize head, zero mapper wait" },
] as const;
export type FloorVariant = (typeof FLOOR_VARIANTS)[number];

const MAPPERS: Record<"stub" | "replay" | "record", (vize: string[]) => string[]> = {
  stub: () => ["node", join(FLOORS_DIR, "stub-mapper.ts")],
  replay: () => ["node", join(FLOORS_DIR, "replay-mapper.ts")],
  record: (vize) => ["node", join(FLOORS_DIR, "record-proxy.ts"), ...vize],
};
export const floorTsconfig = (mapper: keyof typeof MAPPERS) => `tsconfig.floor-${mapper}.json`;

/** One mapper package and tsconfig per floor mapper, beside the project's installed Vize. */
export function prepareFloorPackages(project: string): void {
  const vizeDir = join(project, "node_modules", "vize");
  const manifest = JSON.parse(readFileSync(join(vizeDir, "package.json"), "utf8")) as {
    typescript: { contentMapper: Record<string, unknown> };
  };
  const vize = ["node", join(vizeDir, "bin", "vize"), "content-mapper"];
  for (const [id, exec] of Object.entries(MAPPERS)) {
    const name = `vize-floor-${id}`;
    const dir = join(project, "node_modules", name);
    mkdirSync(dir, { recursive: true });
    const contentMapper = { ...manifest.typescript.contentMapper, exec: exec(vize) };
    writeFileSync(
      join(dir, "package.json"),
      `${JSON.stringify({ name, version: "0.0.0", private: true, typescript: { contentMapper } })}\n`,
    );
    writeFileSync(
      join(project, floorTsconfig(id as keyof typeof MAPPERS)),
      `${JSON.stringify({ extends: "./tsconfig.json", contentMappers: [{ package: name, extensions: [".vue"] }] })}\n`,
    );
  }
}

export type FloorRecordings = {
  replay: string;
  expressions: string;
  ablation: ReturnType<typeof transformRecording>;
};

/** Record the head projection once (`record` runs one untimed host check) and derive the ablated copy. */
export function recordFloors(
  dir: string,
  record: (tsconfig: string, env: Record<string, string>) => void,
): FloorRecordings {
  const replay = join(dir, "replay");
  record(floorTsconfig("record"), { FLOOR_RECORD_DIR: replay });
  const expressions = join(dir, "expressions");
  return { replay, expressions, ablation: transformRecording(replay, expressions, ablateResult) };
}

/** The tsconfig and environment a timed floor run checks with. */
export function floorRun(
  variant: FloorVariant,
  recordings: FloorRecordings,
): { tsconfig: string; env: Record<string, string> } {
  if (variant.id === "floor-absolute") return { tsconfig: floorTsconfig("stub"), env: {} };
  const dir = variant.id === "floor-expressions" ? recordings.expressions : recordings.replay;
  return { tsconfig: floorTsconfig("replay"), env: { FLOOR_REPLAY_DIR: dir } };
}
