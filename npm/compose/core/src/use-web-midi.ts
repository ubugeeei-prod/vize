import { computed, readonly, ref, shallowReadonly, shallowRef, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Minimal `MIDIPort` read by {@link useWebMIDI}. */
export interface MIDIPortLike extends EventTarget {
  /** Stable port identifier. */
  readonly id: string;
  /** Port name. */
  readonly name?: string | null;
  /** Manufacturer name. */
  readonly manufacturer?: string | null;
  /** Device state. */
  readonly state: string;
  /** Connection state. */
  readonly connection: string;
}

/** Minimal `MIDIOutput` used by {@link useWebMIDI}. */
export interface MIDIOutputLike extends MIDIPortLike {
  /** Send one or more MIDI messages, optionally scheduled at `timestamp`. */
  send(data: number[], timestamp?: number): void;
}

/** Minimal `MIDIAccess` used by {@link useWebMIDI}. */
export interface MIDIAccessLike extends EventTarget {
  /** Available inputs. */
  readonly inputs: { values(): Iterable<MIDIPortLike> };
  /** Available outputs. */
  readonly outputs: { values(): Iterable<MIDIOutputLike> };
}

/** Access request flags of `navigator.requestMIDIAccess`. */
export interface MIDIRequestFlags {
  /** Request system-exclusive message access. */
  readonly sysex?: boolean;
  /** Include software synthesizers. */
  readonly software?: boolean;
}

/** Navigator-like capability used by {@link useWebMIDI}. */
export interface MIDIHost {
  /** Request MIDI access. */
  requestMIDIAccess(options?: MIDIRequestFlags): Promise<MIDIAccessLike>;
}

/** Plain description of one MIDI port. */
export interface MIDIPortInfo {
  /** Stable port identifier. */
  readonly id: string;
  /** Port name (empty when unknown). */
  readonly name: string;
  /** Manufacturer (empty when unknown). */
  readonly manufacturer: string;
  /** Device state, e.g. `"connected"` or `"disconnected"`. */
  readonly state: string;
  /** Connection state, e.g. `"open"`, `"closed"` or `"pending"`. */
  readonly connection: string;
}

/** A message received on an input port. */
export interface MIDIInputMessage {
  /** Raw message bytes. */
  readonly data: Uint8Array;
  /** Event timestamp in milliseconds. */
  readonly timeStamp: number;
  /** Identifier of the receiving input. */
  readonly inputId: string;
}

/** Lifecycle of the access request. */
export type MIDIAccessStatus = "idle" | "pending" | "granted" | "denied";

/** Options for {@link useWebMIDI}. */
export interface UseWebMIDIOptions {
  /**
   * Navigator-like MIDI capability.
   *
   * @default window.navigator when `requestMIDIAccess` exists
   */
  readonly host?: MaybeRefOrGetter<MIDIHost | null | undefined>;

  /**
   * Request system-exclusive access.
   *
   * @default false
   */
  readonly sysex?: boolean;

  /**
   * Include software synthesizers.
   *
   * @default false
   */
  readonly software?: boolean;

  /**
   * Receives every input message. When set, `midimessage` listeners are
   * attached to all inputs (which implicitly opens them).
   *
   * @default undefined
   */
  readonly onMessage?: (message: MIDIInputMessage) => void;
}

/** Reactive state and actions returned by {@link useWebMIDI}. */
export interface WebMIDIControls {
  /** Whether Web MIDI is available. */
  readonly supported: ComputedRef<boolean>;
  /** Access request status. */
  readonly status: Readonly<Ref<MIDIAccessStatus>>;
  /** Input ports, refreshed on `statechange`. */
  readonly inputs: Readonly<ShallowRef<readonly MIDIPortInfo[]>>;
  /** Output ports, refreshed on `statechange`. */
  readonly outputs: Readonly<ShallowRef<readonly MIDIPortInfo[]>>;
  /** Most recent request or send failure. */
  readonly error: Readonly<ShallowRef<unknown>>;
  /**
   * Request MIDI access (may prompt the user).
   *
   * @returns Whether access was granted.
   */
  readonly request: () => Promise<boolean>;
  /**
   * Send bytes to an output.
   *
   * @returns Whether the output exists and accepted the message.
   */
  readonly send: (
    outputId: string,
    data: Uint8Array | readonly number[],
    timestamp?: number,
  ) => boolean;
  /** Remove every listener and forget the access object. Idempotent. */
  readonly stop: () => void;
}

function browserMIDIHost(): MIDIHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator } = window;
  return typeof navigator.requestMIDIAccess === "function" ? navigator : undefined;
}

function describe(port: MIDIPortLike): MIDIPortInfo {
  return {
    id: port.id,
    name: port.name ?? "",
    manufacturer: port.manufacturer ?? "",
    state: port.state,
    connection: port.connection,
  };
}

function hasMessageData(event: Event): event is Event & { readonly data: Uint8Array | null } {
  return "data" in event;
}

/**
 * Access MIDI devices with the Web MIDI API.
 *
 * Nothing is requested until `request()` is called. Once granted, `inputs`
 * and `outputs` follow `statechange` events, and `onMessage` receives
 * messages from every input. Failures land in `error`. Listeners are removed
 * when the owning reactive scope stops; outside a scope call `stop()`.
 *
 * Server rendering: `supported` is false, `status` stays `"idle"` and no
 * access is requested.
 *
 * @example
 * ```ts
 * const midi = useWebMIDI({ onMessage: ({ data }) => console.log(data) });
 * await midi.request();
 * midi.send(midi.outputs.value[0]!.id, [0x90, 60, 127]);
 * ```
 *
 * @param options Host, access flags, and message callback.
 * @default options {}
 * @returns MIDI state and actions.
 */
export function useWebMIDI(options: UseWebMIDIOptions = {}): WebMIDIControls {
  const status = ref<MIDIAccessStatus>("idle");
  const inputs = shallowRef<readonly MIDIPortInfo[]>([]);
  const outputs = shallowRef<readonly MIDIPortInfo[]>([]);
  const error = shallowRef<unknown>(undefined);
  const listening = new Map<MIDIPortLike, (event: Event) => void>();
  let access: MIDIAccessLike | undefined;
  let generation = 0;

  const resolveHost = (): MIDIHost | undefined =>
    options.host === undefined ? browserMIDIHost() : (toValue(options.host) ?? undefined);

  const listen = (input: MIDIPortLike): void => {
    const onMessage = options.onMessage;
    if (!onMessage || listening.has(input)) return;
    const listener = (event: Event): void => {
      if (!hasMessageData(event) || !event.data) return;
      onMessage({ data: event.data, timeStamp: event.timeStamp, inputId: input.id });
    };
    listening.set(input, listener);
    input.addEventListener("midimessage", listener);
  };

  const unlisten = (): void => {
    for (const [input, listener] of listening) input.removeEventListener("midimessage", listener);
    listening.clear();
  };

  const refresh = (): void => {
    if (!access) return;
    const inputPorts = [...access.inputs.values()];
    for (const input of inputPorts) listen(input);
    inputs.value = inputPorts.map(describe);
    outputs.value = [...access.outputs.values()].map(describe);
  };

  const stop = (): void => {
    generation += 1;
    unlisten();
    access?.removeEventListener("statechange", refresh);
    access = undefined;
    inputs.value = [];
    outputs.value = [];
    if (status.value === "pending") status.value = "idle";
  };

  const request = async (): Promise<boolean> => {
    const host = resolveHost();
    if (!host) return false;
    stop();
    const current = generation;
    status.value = "pending";
    try {
      const next = await host.requestMIDIAccess({
        sysex: options.sysex ?? false,
        software: options.software ?? false,
      });
      if (current !== generation) return false;
      access = next;
      next.addEventListener("statechange", refresh);
      refresh();
      status.value = "granted";
      error.value = undefined;
      return true;
    } catch (cause) {
      if (current !== generation) return false;
      status.value = "denied";
      error.value = cause;
      return false;
    }
  };

  const send = (
    outputId: string,
    data: Uint8Array | readonly number[],
    timestamp?: number,
  ): boolean => {
    const output = access
      ? [...access.outputs.values()].find((port) => port.id === outputId)
      : undefined;
    if (!output) return false;
    try {
      if (timestamp === undefined) output.send(Array.from(data));
      else output.send(Array.from(data), timestamp);
      return true;
    } catch (cause) {
      error.value = cause;
      return false;
    }
  };

  tryOnScopeDispose(stop);

  return {
    supported: computed(() => resolveHost() !== undefined),
    status: readonly(status),
    inputs: shallowReadonly(inputs),
    outputs: shallowReadonly(outputs),
    error: readonly(error),
    request,
    send,
    stop,
  };
}
