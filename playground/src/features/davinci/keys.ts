// Presenter keys for the Davinci tab: `1`-`4` jump to a stage, `←`/`→` walk
// the pass timeline. Keys typed into the editor or a form control are never
// taken.

import type { TimelineStep } from "./ladder";

export type KeyAction =
  | { kind: "stage"; stage: "s1" | "s2" | "s3" | "s4" }
  | { kind: "step"; step: TimelineStep };

const STAGE_KEYS: Record<string, "s1" | "s2" | "s3" | "s4"> = {
  "1": "s1",
  "2": "s2",
  "3": "s3",
  "4": "s4",
};

function isTyping(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false;
  if (target.closest(".monaco-editor")) return true;
  const tag = target.tagName;
  return (
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    tag === "SELECT" ||
    (target instanceof HTMLElement && target.isContentEditable)
  );
}

/** What a keydown means for the tab, or null when it is not ours. */
export function stepKeyAction(
  event: Pick<KeyboardEvent, "key" | "target" | "altKey" | "ctrlKey" | "metaKey" | "shiftKey">,
  timeline: readonly TimelineStep[],
  current: string | null,
): KeyAction | null {
  if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return null;
  if (isTyping(event.target)) return null;
  const stage = STAGE_KEYS[event.key];
  if (stage) return { kind: "stage", stage };
  if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") return null;
  if (timeline.length === 0) return null;
  const index = timeline.findIndex((step) => step.key === current);
  const next =
    event.key === "ArrowRight"
      ? Math.min(index + 1, timeline.length - 1)
      : index === -1
        ? timeline.length - 1
        : Math.max(index - 1, 0);
  return { kind: "step", step: timeline[next] };
}
