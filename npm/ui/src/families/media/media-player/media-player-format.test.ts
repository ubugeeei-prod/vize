import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  MEDIA_PLAYER_DEFAULT_RATES,
  bufferedEnd,
  clampMediaValue,
  formatMediaTime,
  isCaptionTrack,
  isEditableTarget,
  mediaPlayerDefaultMessages,
  normalizeMediaDuration,
  pointerRatio,
  readTimeRanges,
  resolveMediaPlayerMessages,
  resolveMediaShortcut,
  resolveSliderKey,
  toPercent,
} from "./media-player-format.ts";

test("formats media time with aligned hours and unknown values", () => {
  assert.equal(formatMediaTime(0), "0:00");
  assert.equal(formatMediaTime(83.9), "1:23");
  assert.equal(formatMediaTime(600), "10:00");
  assert.equal(formatMediaTime(3725), "1:02:05");
  assert.equal(formatMediaTime(65, 3600), "0:01:05", "hours align with long durations");
  assert.equal(formatMediaTime(-217), "-3:37");
  assert.equal(formatMediaTime(Number.NaN), "--:--");
  assert.equal(formatMediaTime(Number.POSITIVE_INFINITY), "--:--");
  assert.equal(formatMediaTime(5, Number.POSITIVE_INFINITY), "0:05");
});

test("normalizes durations, clamps values, and computes percentages", () => {
  assert.equal(normalizeMediaDuration(Number.NaN), 0);
  assert.equal(normalizeMediaDuration(-1), 0);
  assert.equal(normalizeMediaDuration(12.5), 12.5);
  assert.equal(normalizeMediaDuration(Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY);
  assert.equal(clampMediaValue(5, 0, 1), 1);
  assert.equal(clampMediaValue(-5, 0, 1), 0);
  assert.equal(clampMediaValue(Number.NaN, 0, 1), 0);
  assert.equal(toPercent(1, 3), 33.33);
  assert.equal(toPercent(5, 0), 0);
  assert.equal(toPercent(5, Number.POSITIVE_INFINITY), 0);
  assert.equal(pointerRatio({ left: 10, width: 100 }, 35, false), 0.25);
  assert.equal(pointerRatio({ left: 10, width: 100 }, 35, true), 0.75);
  assert.equal(pointerRatio({ left: 10, width: 0 }, 35, false), 0);
  assert.equal(pointerRatio({ left: 10, width: 100 }, 500, false), 1);
});

test("copies buffered ranges and finds the buffered end around a time", () => {
  const ranges = readTimeRanges({
    length: 2,
    start: (index: number) => [0, 40][index] ?? 0,
    end: (index: number) => [20, 60][index] ?? 0,
  });
  assert.deepEqual(ranges, [
    { start: 0, end: 20 },
    { start: 40, end: 60 },
  ]);
  assert.ok(Object.isFrozen(ranges[0]));
  assert.deepEqual(readTimeRanges(null), []);
  assert.equal(bufferedEnd(ranges, 45), 60);
  assert.equal(bufferedEnd(ranges, 30), 30);
});

test("resolves keyboard shortcuts and slider keys", () => {
  const press = (key: string, modifiers: Partial<KeyboardEvent> = {}) =>
    resolveMediaShortcut({ altKey: false, ctrlKey: false, metaKey: false, key, ...modifiers });
  assert.deepEqual(press(" "), { action: "toggle-play" });
  assert.deepEqual(press("K"), { action: "toggle-play" });
  assert.deepEqual(press("j"), { action: "skip-backward" });
  assert.deepEqual(press("l"), { action: "skip-forward" });
  assert.deepEqual(press("ArrowLeft"), { action: "seek-backward" });
  assert.deepEqual(press("ArrowRight"), { action: "seek-forward" });
  assert.deepEqual(press("ArrowUp"), { action: "volume-up" });
  assert.deepEqual(press("ArrowDown"), { action: "volume-down" });
  assert.deepEqual(press("m"), { action: "mute" });
  assert.deepEqual(press("f"), { action: "fullscreen" });
  assert.deepEqual(press("c"), { action: "captions" });
  assert.deepEqual(press("0"), { action: "seek-percent", percent: 0 });
  assert.deepEqual(press("9"), { action: "seek-percent", percent: 90 });
  assert.equal(press("k", { metaKey: true }), null);
  assert.equal(press("Enter"), null);

  assert.deepEqual(resolveSliderKey("ArrowRight", false), { kind: "step", sign: 1 });
  assert.deepEqual(resolveSliderKey("ArrowRight", true), { kind: "step", sign: -1 });
  assert.deepEqual(resolveSliderKey("ArrowLeft", true), { kind: "step", sign: 1 });
  assert.deepEqual(resolveSliderKey("ArrowDown", true), { kind: "step", sign: -1 });
  assert.deepEqual(resolveSliderKey("PageUp", false), { kind: "page", sign: 1 });
  assert.deepEqual(resolveSliderKey("End", false), { kind: "edge", edge: "max" });
  assert.equal(resolveSliderKey("a", false), null);
});

test("detects editable targets and caption tracks", () => {
  const text = document.createElement("input");
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  const editable = document.createElement("div");
  editable.contentEditable = "true";
  assert.equal(isEditableTarget(text), true);
  assert.equal(isEditableTarget(checkbox), false);
  assert.equal(isEditableTarget(document.createElement("textarea")), true);
  assert.equal(isEditableTarget(document.createElement("select")), true);
  assert.equal(isEditableTarget(document.createElement("button")), false);
  assert.equal(isEditableTarget(null), false);
  assert.equal(isCaptionTrack({ kind: "captions" }), true);
  assert.equal(isCaptionTrack({ kind: "subtitles" }), true);
  assert.equal(isCaptionTrack({ kind: "chapters" }), false);
});

test("merges messages over frozen English defaults", () => {
  assert.equal(resolveMediaPlayerMessages(undefined), mediaPlayerDefaultMessages);
  const merged = resolveMediaPlayerMessages({ play: "Lecture", volume: undefined });
  assert.equal(merged.play, "Lecture");
  assert.equal(merged.volume, "Volume");
  assert.equal(merged.seekValueText("1:00", "2:00"), "1:00 of 2:00");
  assert.equal(merged.volumeValueText(40, false), "40%");
  assert.equal(merged.volumeValueText(40, true), "Muted");
  assert.equal(merged.playbackRate(1.5), "Playback speed 1.5×");
  assert.ok(Object.isFrozen(merged));
  assert.deepEqual(MEDIA_PLAYER_DEFAULT_RATES, [0.5, 1, 1.25, 1.5, 2]);
});
