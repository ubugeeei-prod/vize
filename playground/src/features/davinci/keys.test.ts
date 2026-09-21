import { describe, expect, it } from "vite-plus/test";
import { stepKeyAction } from "./keys";
import type { TimelineStep } from "./ladder";

const step = (key: string): TimelineStep => ({
  key,
  rung: "s2",
  pass: key,
  changed: false,
  producer: false,
  nanos: null,
  remarks: 0,
  walk: null,
});
const timeline = [step("s1/parse"), step("s2/lower"), step("s3/lower")];
const press = (key: string, target: EventTarget | null = document.body, modifiers = {}) => ({
  key,
  target,
  altKey: false,
  ctrlKey: false,
  metaKey: false,
  shiftKey: false,
  ...modifiers,
});

describe("stepKeyAction", () => {
  it("maps 1-4 to stages and arrows to neighbouring steps", () => {
    expect(stepKeyAction(press("3"), timeline, null)).toEqual({ kind: "stage", stage: "s3" });
    expect(stepKeyAction(press("4"), timeline, null)).toEqual({ kind: "stage", stage: "s4" });
    expect(stepKeyAction(press("ArrowRight"), timeline, "s1/parse")).toEqual({
      kind: "step",
      step: timeline[1],
    });
    expect(stepKeyAction(press("ArrowLeft"), timeline, "s2/lower")).toEqual({
      kind: "step",
      step: timeline[0],
    });
  });

  it("clamps at both ends and starts from the end when nothing is current", () => {
    expect(stepKeyAction(press("ArrowRight"), timeline, "s3/lower")).toEqual({
      kind: "step",
      step: timeline[2],
    });
    expect(stepKeyAction(press("ArrowLeft"), timeline, "s1/parse")).toEqual({
      kind: "step",
      step: timeline[0],
    });
    expect(stepKeyAction(press("ArrowLeft"), timeline, null)).toEqual({
      kind: "step",
      step: timeline[2],
    });
    expect(stepKeyAction(press("ArrowRight"), [], null)).toBeNull();
  });

  it("never takes keys typed into the editor, a field, or with modifiers", () => {
    const editor = document.createElement("div");
    editor.className = "monaco-editor";
    const inner = document.createElement("span");
    editor.append(inner);
    expect(stepKeyAction(press("2", inner), timeline, null)).toBeNull();
    expect(stepKeyAction(press("2", document.createElement("input")), timeline, null)).toBeNull();
    expect(stepKeyAction(press("2", document.body, { metaKey: true }), timeline, null)).toBeNull();
    expect(stepKeyAction(press("x"), timeline, null)).toBeNull();
  });
});
