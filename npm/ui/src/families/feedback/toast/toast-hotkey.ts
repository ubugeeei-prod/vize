const modifierLabels = {
  altKey: "Alt",
  ctrlKey: "Ctrl",
  metaKey: "Meta",
  shiftKey: "Shift",
} as const;

type ToastHotkeyModifier = keyof typeof modifierLabels;

function isModifier(token: string): token is ToastHotkeyModifier {
  return token in modifierLabels;
}

/** Human-readable hotkey label such as `Alt+T` used in the region name. */
export function formatToastHotkey(hotkey: readonly string[]): string {
  return hotkey
    .map((token) => (isModifier(token) ? modifierLabels[token] : token.replace(/^Key/, "")))
    .join("+");
}

/**
 * Whether a keyboard event matches every hotkey token. Modifier tokens name
 * `KeyboardEvent` flags; other tokens match `event.code` or `event.key`.
 */
export function matchesToastHotkey(event: KeyboardEvent, hotkey: readonly string[]): boolean {
  if (hotkey.length === 0) return false;
  return hotkey.every((token) =>
    isModifier(token) ? event[token] : event.code === token || event.key === token,
  );
}
