/**
 * useIme - IME (Input Method Editor) composable
 */

import { ref, computed, type Ref } from "vue";

export interface UseImeOptions {
  /** Initial IME mode */
  mode?: ImeMode;
  /** Called when IME mode changes */
  onModeChange?: (mode: ImeMode) => void;
  /** Called when composition updates */
  onCompositionUpdate?: (text: string, cursor: number) => void;
  /** Called when text is committed */
  onCommit?: (text: string) => void;
}

export type ImeMode =
  | "direct"
  | "hiragana"
  | "katakana"
  | "half-katakana"
  | "full-alpha"
  | "pinyin"
  | "hangul";

export interface ImeManager {
  /** Whether IME is active */
  isActive: Ref<boolean>;
  /** Current input mode */
  mode: Ref<ImeMode>;
  /** Whether currently composing */
  isComposing: Ref<boolean>;
  /** Preedit text */
  preedit: Ref<string>;
  /** Cursor position in preedit */
  preeditCursor: Ref<number>;
  /** Candidate list */
  candidates: Ref<string[]>;
  /** Selected candidate index */
  selectedCandidate: Ref<number>;
  /** Mode display name */
  modeDisplay: Ref<string>;
  /** Enable IME */
  enable: () => void;
  /** Disable IME */
  disable: () => void;
  /** Set input mode */
  setMode: (mode: ImeMode) => void;
  /** Handle key event for IME */
  handleKey: (key: string, modifiers: { ctrl: boolean; alt: boolean }) => boolean;
  /** Commit current composition */
  commit: () => void;
  /** Cancel current composition */
  cancel: () => void;
  /** Select next candidate */
  nextCandidate: () => void;
  /** Select previous candidate */
  prevCandidate: () => void;
  /** Select candidate by number (1-9) */
  selectCandidate: (num: number) => void;
}

const MODE_DISPLAY: Record<ImeMode, string> = {
  direct: "A",
  hiragana: "あ",
  katakana: "ア",
  "half-katakana": "ｱ",
  "full-alpha": "Ａ",
  pinyin: "拼",
  hangul: "한",
};

export function useIme(options: UseImeOptions = {}): ImeManager {
  const { mode: initialMode = "direct", onModeChange, onCompositionUpdate, onCommit } = options;

  const isActive = ref(false);
  const mode = ref<ImeMode>(initialMode);
  const isComposing = ref(false);
  const preedit = ref("");
  const preeditCursor = ref(0);
  const candidates = ref<string[]>([]);
  const selectedCandidate = ref(0);

  const modeDisplay = computed(() => MODE_DISPLAY[mode.value] ?? "A");

  const enable = () => {
    isActive.value = true;
  };

  const disable = () => {
    isActive.value = false;
    cancel();
  };

  const setMode = (newMode: ImeMode) => {
    if (mode.value !== newMode) {
      cancel();
      mode.value = newMode;
      onModeChange?.(newMode);
    }
  };

  const handleKey = (key: string, modifiers: { ctrl: boolean; alt: boolean }): boolean => {
    if (!isActive.value || mode.value === "direct") {
      return false;
    }

    // IME mode toggle (usually Ctrl+Space or similar)
    // If we're here, mode is not 'direct' (filtered above), so toggle to 'direct'
    if (modifiers.ctrl && key === " ") {
      setMode("direct");
      return true;
    }

    // Handle composition
    if (isComposing.value) {
      switch (key) {
        case "enter":
          commit();
          return true;
        case "escape":
          cancel();
          return true;
        case "backspace":
          if (preedit.value.length > 0) {
            preedit.value = preedit.value.slice(0, -1);
            preeditCursor.value = Math.min(preeditCursor.value, preedit.value.length);
            onCompositionUpdate?.(preedit.value, preeditCursor.value);
            return true;
          }
          break;
        case "space":
          // Convert / select candidate
          if (candidates.value.length > 0) {
            nextCandidate();
            return true;
          }
          break;
      }

      // Number for candidate selection
      if (/^[1-9]$/.test(key)) {
        selectCandidate(parseInt(key, 10));
        return true;
      }
    }

    // Start/continue composition for printable characters
    if (key.length === 1 && !modifiers.ctrl && !modifiers.alt) {
      if (!isComposing.value) {
        isComposing.value = true;
      }
      preedit.value += key;
      preeditCursor.value = preedit.value.length;
      onCompositionUpdate?.(preedit.value, preeditCursor.value);
      return true;
    }

    return false;
  };

  const commit = () => {
    if (preedit.value) {
      const text = candidates.value[selectedCandidate.value] ?? preedit.value;
      onCommit?.(text);
    }
    preedit.value = "";
    preeditCursor.value = 0;
    candidates.value = [];
    selectedCandidate.value = 0;
    isComposing.value = false;
  };

  const cancel = () => {
    preedit.value = "";
    preeditCursor.value = 0;
    candidates.value = [];
    selectedCandidate.value = 0;
    isComposing.value = false;
  };

  const nextCandidate = () => {
    if (candidates.value.length > 0) {
      selectedCandidate.value = (selectedCandidate.value + 1) % candidates.value.length;
    }
  };

  const prevCandidate = () => {
    if (candidates.value.length > 0) {
      selectedCandidate.value =
        (selectedCandidate.value - 1 + candidates.value.length) % candidates.value.length;
    }
  };

  const selectCandidate = (num: number) => {
    if (num >= 1 && num <= candidates.value.length) {
      selectedCandidate.value = num - 1;
      commit();
    }
  };

  return {
    isActive,
    mode,
    isComposing,
    preedit,
    preeditCursor,
    candidates,
    selectedCandidate,
    modeDisplay,
    enable,
    disable,
    setMode,
    handleKey,
    commit,
    cancel,
    nextCandidate,
    prevCandidate,
    selectCandidate,
  };
}
