// What every TS-45 driver receives, and the scenario loader. Drivers read the
// scenario for *inputs* (positions, edits, how many diagnostics to wait for);
// only `conformance.ts` compares outputs.
import fs from "node:fs";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { fileURLToPath } from "node:url";

import type { Scenario, Step } from "../conformance.ts";
import { latestPublish, readTranscript } from "./transcript.ts";

export const suiteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
export const repoRoot = path.resolve(suiteRoot, "..", "..");

export type DriverContext = {
  scenario: Scenario;
  /** The workspace directory the editor opens, and every spelling of it a client may use. */
  workspace: string;
  roots: string[];
  documentPath: string;
  /** Normalized scenario URI (`${workspace}/…`), as the judge sees it. */
  documentKey: string;
  /** Environment with the tap shim first on PATH and the transcript path set. */
  env: NodeJS.ProcessEnv;
  transcript: string;
  /** Scratch directory for the editor's own config/state. */
  scratch: string;
};

export type Driver = {
  client: string;
  /** The client version actually under test (read from the binary for real editors). */
  version: () => string;
  mode: "editor" | "replay";
  run(context: DriverContext): Promise<void>;
};

/** Loads `scenario.json`, inlining every `{ "$file": … }` expectation. */
export function loadScenario(file = path.join(suiteRoot, "scenario.json")): Scenario {
  const scenario = JSON.parse(fs.readFileSync(file, "utf8")) as Scenario;
  for (const step of scenario.steps as Array<Step & { expect: any }>) {
    if (step.expect != null && typeof step.expect === "object" && "$file" in step.expect) {
      step.expect = JSON.parse(
        fs.readFileSync(path.join(path.dirname(file), step.expect.$file), "utf8"),
      );
    }
  }
  return scenario;
}

export function step<K extends Step["kind"]>(
  scenario: Scenario,
  id: string,
  kind: K,
): Extract<Step, { kind: K }> {
  const found = scenario.steps.find((candidate) => candidate.id === id);
  if (found?.kind !== kind) throw new Error(`scenario step ${id} is not a ${kind} step`);
  return found as Extract<Step, { kind: K }>;
}

/**
 * Waits until the newest publish for the scenario document after transcript
 * entry `after` carries `count` diagnostics and nothing newer arrives for
 * `quietMs`. Only pacing: the judge re-reads the transcript for the verdict.
 */
export async function settleDiagnostics(
  context: DriverContext,
  after: number,
  count: number,
  quietMs = 1_000,
) {
  const deadline = Date.now() + 180_000;
  let seen = -1;
  let since = Date.now();
  for (;;) {
    const publish = latestPublish(
      readTranscript(context.transcript),
      context.documentKey,
      context.roots,
      after,
    );
    if (publish != null && publish.index !== seen) {
      seen = publish.index;
      since = Date.now();
    }
    if (
      publish != null &&
      publish.params.diagnostics.length === count &&
      Date.now() - since >= quietMs
    )
      return;
    if (Date.now() > deadline) {
      throw new Error(
        `diagnostics did not settle at ${count} (last: ${JSON.stringify(publish?.params ?? null)})`,
      );
    }
    await sleep(100);
  }
}

export function transcriptLength(context: DriverContext): number {
  return readTranscript(context.transcript).length;
}
