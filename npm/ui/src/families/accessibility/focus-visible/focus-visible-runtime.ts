import { getCurrentScope } from "vue";

import { useInteractionModality } from "../interaction-modality/interaction-modality.ts";
import type { InteractionModality } from "../interaction-modality/interaction-modality.ts";
import { focusVisibleContext } from "./focus-visible-context.ts";
import type { FocusVisibleState } from "./focus-visible-types.ts";

const setupDiagnostic = "VIZE_UI_FOCUS_VISIBLE_SETUP";
const textEntryTypes = new Set([
  "",
  "date",
  "datetime-local",
  "email",
  "month",
  "number",
  "password",
  "search",
  "tel",
  "text",
  "time",
  "url",
  "week",
]);

/**
 * Whether an element accepts typed text, where browsers always indicate focus
 * regardless of the input modality that moved focus there.
 */
export function isTextEntryElement(element: Element): boolean {
  const tag = element.tagName.toLowerCase();
  if (tag === "textarea") return true;
  if (tag === "input")
    return textEntryTypes.has((element.getAttribute("type") ?? "").toLowerCase());
  return (
    element.getAttribute("contenteditable") !== null &&
    element.getAttribute("contenteditable") !== "false"
  );
}

/**
 * Decide whether focus on `element` should be indicated for `modality`.
 *
 * Keyboard and virtual (assistive technology) focus is always indicated, text
 * entry fields are indicated for every modality, and pointer or touch focus on
 * other elements is not — mirroring the `:focus-visible` heuristics browsers
 * document, but deterministically across engines.
 */
export function shouldShowFocusRing(
  element: Element,
  modality: InteractionModality | null,
): boolean {
  if (modality === "keyboard" || modality === "virtual") return true;
  if (isTextEntryElement(element)) return true;
  return modality === null;
}

/**
 * Read focus-visible state from the nearest FocusVisibleProvider, or track the
 * document interaction modality standalone.
 */
export function useFocusVisible(): FocusVisibleState {
  const provided = focusVisibleContext.useOptional();
  if (provided) return provided;
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const tracker = useInteractionModality();
  return Object.freeze({ isFocusVisible: tracker.isFocusVisible, modality: tracker.modality });
}
