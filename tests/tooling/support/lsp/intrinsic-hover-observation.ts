import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

/** Capture actual configuration bytes before the temporary project is removed. */
export function intrinsicWorkspaceControls(workspace: string) {
  return ["package.json", "tsconfig.json", "vite.config.mjs", "src/Counter.vue"].flatMap(
    (relative) => {
      const filename = path.join(workspace, relative);
      if (!fs.existsSync(filename)) return [];
      const bytes = fs.readFileSync(filename);
      return [
        {
          path: relative,
          bytesBase64: bytes.toString("base64"),
          sha256: createHash("sha256").update(bytes).digest("hex"),
        },
      ];
    },
  );
}

type Timing = { label: string; phase: string; elapsedMs: number };

/** Observed whole-request CI latency; no admission limit or universal speed claim. */
export class IntrinsicHoverTimings {
  private readonly observations: Timing[] = [];
  private readonly seen = new Set<string>();

  record(label: string, kind: string, elapsedMs: number): void {
    const phase = kind.startsWith("warm ")
      ? kind.endsWith("primitive")
        ? "warm-primitive"
        : "warm-DOM"
      : kind === "new revision physical range"
        ? "updated-DOM"
        : this.seen.has(label)
          ? "followup"
          : "first-DOM";
    this.seen.add(label);
    this.observations.push({ label, phase, elapsedMs });
  }

  summary() {
    const groups = new Map<string, number[]>();
    for (const { label, phase, elapsedMs } of this.observations) {
      const key = `${label} ${phase}`;
      const group = groups.get(key) ?? [];
      group.push(elapsedMs);
      groups.set(key, group);
    }
    return [...groups].map(([group, values]) => {
      values.sort((a, b) => a - b);
      return {
        group,
        count: values.length,
        p50Ms: values[Math.ceil(values.length * 0.5) - 1],
        p95Ms: values[Math.ceil(values.length * 0.95) - 1],
        maxMs: values.at(-1),
      };
    });
  }
}
