import {
  computed,
  onUnmounted,
  readonly,
  ref,
  toValue,
  watch,
  type MaybeRefOrGetter,
  type Ref,
} from "vue";
import { useInput } from "./useInput.js";

export interface KeymapStroke {
  readonly key: string;
  readonly ctrl?: boolean;
  readonly alt?: boolean;
  readonly shift?: boolean;
  readonly meta?: boolean;
}

export interface KeymapBinding<Command extends string = string> {
  readonly command: Command;
  readonly sequence: readonly [KeymapStroke, ...KeymapStroke[]];
}

export type Keymap<Command extends string = string> = readonly KeymapBinding<Command>[];

export interface KeymapMatch<Command extends string> {
  readonly command?: Command;
  readonly binding?: KeymapBinding<Command>;
  readonly pending: boolean;
}

export interface KeymapResolver<Command extends string> {
  readonly pendingSequence: readonly KeymapStroke[];
  feed(stroke: KeymapStroke): KeymapMatch<Command>;
  reset(): void;
}

export interface UseKeymapOptions<Command extends string> {
  readonly keymap: MaybeRefOrGetter<Keymap<Command>>;
  readonly active?: MaybeRefOrGetter<boolean>;
  /** Idle time before an incomplete multi-stroke sequence is cleared. */
  readonly sequenceTimeout?: number;
  readonly onCommand: (command: Command, binding: KeymapBinding<Command>) => void;
}

export interface UseKeymapReturn {
  readonly isActive: Readonly<Ref<boolean>>;
  readonly pendingSequence: Readonly<Ref<readonly KeymapStroke[]>>;
  reset(): void;
}

function normalizedKey(key: string): string {
  const lower = key.toLowerCase();
  if (lower === "return") return "enter";
  if (lower === "esc") return "escape";
  if (lower === "pagedown") return "pagedown";
  if (lower === "pageup") return "pageup";
  return lower;
}

function normalizedStroke(stroke: KeymapStroke): Required<KeymapStroke> {
  return {
    key: normalizedKey(stroke.key),
    ctrl: stroke.ctrl ?? false,
    alt: stroke.alt ?? false,
    shift: stroke.shift ?? false,
    meta: stroke.meta ?? false,
  };
}

function strokesEqual(left: KeymapStroke, right: KeymapStroke): boolean {
  const normalizedLeft = normalizedStroke(left);
  const normalizedRight = normalizedStroke(right);
  return (
    normalizedLeft.key === normalizedRight.key &&
    normalizedLeft.ctrl === normalizedRight.ctrl &&
    normalizedLeft.alt === normalizedRight.alt &&
    normalizedLeft.shift === normalizedRight.shift &&
    normalizedLeft.meta === normalizedRight.meta
  );
}

function sequenceKey(sequence: readonly KeymapStroke[]): string {
  return sequence
    .map((stroke) => {
      const normalized = normalizedStroke(stroke);
      return [
        normalized.ctrl ? "ctrl" : "",
        normalized.alt ? "alt" : "",
        normalized.shift ? "shift" : "",
        normalized.meta ? "meta" : "",
        normalized.key,
      ]
        .filter(Boolean)
        .join("+");
    })
    .join(" ");
}

/** Validate and freeze a typed command map. */
export function defineKeymap<const Command extends string>(
  bindings: readonly KeymapBinding<Command>[],
): Keymap<Command> {
  const seen = new Set<string>();
  const normalized = bindings.map((binding) => {
    if (binding.sequence.length === 0) throw new TypeError("keymap sequences must not be empty");
    const sequence = binding.sequence.map(normalizedStroke) as [
      Required<KeymapStroke>,
      ...Required<KeymapStroke>[],
    ];
    const key = sequenceKey(sequence);
    if (seen.has(key)) throw new TypeError(`duplicate keymap sequence: ${key}`);
    seen.add(key);
    return Object.freeze({ command: binding.command, sequence: Object.freeze(sequence) });
  });
  return Object.freeze(normalized);
}

/** Stateful matcher for single chords, Emacs multi-chords, and Vim-style sequences. */
export function createKeymapResolver<Command extends string>(
  keymap: () => Keymap<Command>,
): KeymapResolver<Command> {
  let pending: KeymapStroke[] = [];

  const matchingBindings = (sequence: readonly KeymapStroke[]) =>
    keymap().filter(
      (binding) =>
        binding.sequence.length >= sequence.length &&
        sequence.every((stroke, index) => strokesEqual(stroke, binding.sequence[index]!)),
    );

  const feed = (stroke: KeymapStroke): KeymapMatch<Command> => {
    const next = [...pending, normalizedStroke(stroke)];
    let matches = matchingBindings(next);
    let candidate = next;
    if (matches.length === 0 && pending.length > 0) {
      candidate = [normalizedStroke(stroke)];
      matches = matchingBindings(candidate);
    }
    const exact = matches.find((binding) => binding.sequence.length === candidate.length);
    if (exact) {
      pending = [];
      return { command: exact.command, binding: exact, pending: false };
    }
    pending = matches.length > 0 ? candidate : [];
    return { pending: pending.length > 0 };
  };

  return {
    get pendingSequence() {
      return pending;
    },
    feed,
    reset: () => {
      pending = [];
    },
  };
}

/** Subscribe a typed command keymap to Fresco's normalized input stream. */
export function useKeymap<Command extends string>(
  options: UseKeymapOptions<Command>,
): UseKeymapReturn {
  const active = computed(() => toValue(options.active) ?? true);
  const resolver = createKeymapResolver(() => toValue(options.keymap));
  const pendingSequence = ref<readonly KeymapStroke[]>([]);
  const timeout = Math.max(0, options.sequenceTimeout ?? 1_000);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const reset = () => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
    resolver.reset();
    pendingSequence.value = [];
  };

  watch(
    () => toValue(options.keymap),
    () => reset(),
  );
  onUnmounted(reset);

  useInput({
    isActive: active,
    onKey: (key, modifiers) => {
      if (timer !== undefined) clearTimeout(timer);
      const match = resolver.feed({ key, ...modifiers });
      pendingSequence.value = [...resolver.pendingSequence];
      if (match.command !== undefined && match.binding !== undefined) {
        options.onCommand(match.command, match.binding);
      } else if (match.pending && timeout > 0) {
        timer = setTimeout(reset, timeout);
      }
    },
  });

  return { isActive: active, pendingSequence: readonly(pendingSequence), reset };
}
