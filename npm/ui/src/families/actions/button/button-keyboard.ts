/** Keyboard event phase used by button activation semantics. */
export type ButtonKeyboardPhase = "keydown" | "keyup";

/** Action required to emulate a native button for one keyboard event. */
export type ButtonKeyboardAction = "activate" | "prevent" | "ignore";

/**
 * Resolve native-equivalent activation timing for a non-native button.
 *
 * Enter activates on keydown. Space prevents scrolling on keydown and
 * activates on keyup, matching the interaction users expect from a button.
 */
export function getButtonKeyboardAction(
  key: string,
  phase: ButtonKeyboardPhase,
): ButtonKeyboardAction {
  if (key === "Enter") return phase === "keydown" ? "activate" : "ignore";
  if (key === " ") return phase === "keydown" ? "prevent" : "activate";
  return "ignore";
}
