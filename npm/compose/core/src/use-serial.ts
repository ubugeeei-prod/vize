import { computed, readonly, ref, shallowReadonly, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** USB identifiers used to filter `requestPort`. */
export interface SerialPortFilter {
  /** USB vendor id. */
  readonly usbVendorId?: number;
  /** USB product id. */
  readonly usbProductId?: number;
  /** Bluetooth RFCOMM service class id. */
  readonly bluetoothServiceClassId?: number | string;
}

/** Line settings passed to `SerialPort.open`. */
export interface SerialOpenOptions {
  /** Baud rate, e.g. `9600` or `115200`. */
  readonly baudRate: number;
  /**
   * Data bits per frame (7 or 8).
   *
   * @default 8
   */
  readonly dataBits?: number;
  /**
   * Stop bits (1 or 2).
   *
   * @default 1
   */
  readonly stopBits?: number;
  /**
   * Parity mode.
   *
   * @default "none"
   */
  readonly parity?: "none" | "even" | "odd";
  /**
   * Read/write buffer size in bytes.
   *
   * @default 255
   */
  readonly bufferSize?: number;
  /**
   * Flow control mode.
   *
   * @default "none"
   */
  readonly flowControl?: "none" | "hardware";
}

/** Minimal `SerialPort` used by {@link useSerial}. */
export interface SerialPortLike extends EventTarget {
  /** Byte stream from the device while open; `null` otherwise or after a fatal error. */
  readonly readable: ReadableStream<Uint8Array> | null;
  /** Byte sink to the device while open. */
  readonly writable: WritableStream<Uint8Array> | null;
  /** Open the port. */
  open(options: SerialOpenOptions): Promise<void>;
  /** Close the port. */
  close(): Promise<void>;
}

/** `navigator.serial`-like capability used by {@link useSerial}. */
export interface SerialHost extends EventTarget {
  /** Ports the page already has access to. */
  getPorts(): Promise<SerialPortLike[]>;
  /** Prompt the user for a port. */
  requestPort(options?: {
    readonly filters?: readonly SerialPortFilter[];
  }): Promise<SerialPortLike>;
}

/** How received bytes are delivered. */
export type SerialDecoding = "bytes" | "text";

/** Chunk type delivered for a {@link SerialDecoding}. */
export type SerialChunk<Decoding extends SerialDecoding> = Decoding extends "text"
  ? string
  : Uint8Array;

/** Options for {@link useSerial}. */
export interface UseSerialOptions<Decoding extends SerialDecoding = "bytes"> {
  /**
   * `navigator.serial`-like capability.
   *
   * @default window.navigator.serial when it exists
   */
  readonly host?: MaybeRefOrGetter<SerialHost | null | undefined>;

  /**
   * Deliver raw bytes or text decoded with `encoding` (streaming, so split
   * multi-byte characters are joined).
   *
   * @default "bytes"
   */
  readonly decode?: Decoding;

  /**
   * Text encoding used when `decode` is `"text"`.
   *
   * @default "utf-8"
   */
  readonly encoding?: string;

  /**
   * Receives every chunk read from the open port.
   *
   * @default undefined
   */
  readonly onData?: (chunk: SerialChunk<Decoding>) => void;
}

/** Reactive state and actions returned by {@link useSerial}. */
export interface SerialControls {
  /** Whether Web Serial is available. */
  readonly supported: ComputedRef<boolean>;
  /** Ports the page has access to, refreshed on `connect` / `disconnect`. */
  readonly ports: Readonly<ShallowRef<readonly SerialPortLike[]>>;
  /** Currently open port. */
  readonly port: Readonly<ShallowRef<SerialPortLike | undefined>>;
  /** Whether a port is open. */
  readonly connected: Readonly<Ref<boolean>>;
  /** Most recent failure (request, open, read, write). */
  readonly error: Readonly<ShallowRef<unknown>>;
  /**
   * Prompt the user for a port (requires user activation).
   *
   * @returns The chosen port, or `undefined` when cancelled or failed.
   */
  readonly requestPort: (
    filters?: readonly SerialPortFilter[],
  ) => Promise<SerialPortLike | undefined>;
  /**
   * Refresh `ports`.
   *
   * @returns The granted ports.
   */
  readonly getPorts: () => Promise<readonly SerialPortLike[]>;
  /**
   * Open `port` (closing any other open port) and start the read loop.
   *
   * @returns Whether the port opened.
   */
  readonly open: (port: SerialPortLike, options: SerialOpenOptions) => Promise<boolean>;
  /**
   * Write text (UTF-8 encoded) or bytes.
   *
   * @returns Whether the data was written.
   */
  readonly write: (data: string | Uint8Array) => Promise<boolean>;
  /** Stop reading and close the open port. Idempotent. */
  readonly close: () => Promise<void>;
}

function browserSerialHost(): SerialHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator } = window;
  // Web Serial is not part of the DOM lib; read it through a structural guard.
  const serial: unknown = "serial" in navigator ? navigator.serial : undefined;
  return isSerialHost(serial) ? serial : undefined;
}

function isSerialHost(value: unknown): value is SerialHost {
  return typeof value === "object" && value !== null && "requestPort" in value;
}

interface ReadSession {
  active: boolean;
  reader: ReadableStreamDefaultReader<Uint8Array> | undefined;
  done: Promise<void>;
}

/**
 * Talk to serial devices with the Web Serial API.
 *
 * `requestPort()` prompts for a port, `open()` opens it and starts a read
 * loop that feeds `onData` with bytes or decoded text, `write()` sends text
 * or bytes. `ports` follows the host's `connect` / `disconnect` events and a
 * port's own `disconnect` closes the session. Failures land in `error`. The
 * read loop is cancelled and the port closed when the owning reactive scope
 * stops; outside a scope call `close()`.
 *
 * Server rendering: `supported` and `connected` are false, nothing is read.
 *
 * @example
 * ```ts
 * const serial = useSerial({ decode: "text", onData: (text) => log(text) });
 * const port = await serial.requestPort([{ usbVendorId: 0x2341 }]);
 * if (port) await serial.open(port, { baudRate: 115200 });
 * ```
 *
 * @param options Host, decoding, and data callback.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_SERIAL_INVALID_ENCODING` for an unknown `encoding`.
 * @returns Serial state and actions.
 */
export function useSerial<const Decoding extends SerialDecoding = "bytes">(
  options: UseSerialOptions<Decoding> = {},
): SerialControls {
  const ports = shallowRef<readonly SerialPortLike[]>([]);
  const port = shallowRef<SerialPortLike | undefined>(undefined);
  const connected = ref(false);
  const error = shallowRef<unknown>(undefined);
  const encoding = options.encoding ?? "utf-8";
  const text = options.decode === "text";
  if (text) {
    try {
      new TextDecoder(encoding);
    } catch {
      throw new RangeError(
        `[VIZE_COMPOSE_SERIAL_INVALID_ENCODING] unsupported text encoding "${encoding}"`,
      );
    }
  }
  let session: ReadSession | undefined;

  const resolveHost = (): SerialHost | undefined =>
    options.host === undefined ? browserSerialHost() : (toValue(options.host) ?? undefined);

  const deliver = (chunk: Uint8Array | string): void => {
    // `chunk` is a string exactly when `Decoding` is "text", matching SerialChunk.
    if (chunk.length > 0) options.onData?.(chunk as SerialChunk<Decoding>);
  };

  const readLoop = async (target: SerialPortLike, current: ReadSession): Promise<void> => {
    const decoder = text ? new TextDecoder(encoding) : undefined;
    let stream = target.readable;
    while (stream && current.active) {
      const reader = stream.getReader();
      current.reader = reader;
      try {
        for (;;) {
          const { value, done } = await reader.read();
          if (done) break;
          deliver(decoder ? decoder.decode(value, { stream: true }) : value);
        }
      } catch (cause) {
        if (current.active) error.value = cause;
      } finally {
        reader.releaseLock();
        current.reader = undefined;
      }
      // Non-fatal errors replace `readable`; the same stream again means it is finished.
      if (target.readable === stream) break;
      stream = target.readable;
    }
    if (decoder) deliver(decoder.decode());
  };

  const onPortDisconnect = (): void => {
    void close();
  };

  async function close(): Promise<void> {
    const current = session;
    const target = port.value;
    session = undefined;
    port.value = undefined;
    connected.value = false;
    if (!current || !target) return;
    target.removeEventListener("disconnect", onPortDisconnect);
    current.active = false;
    let failure: unknown;
    try {
      await current.reader?.cancel();
    } catch (cause) {
      failure = cause;
    }
    try {
      await current.done;
    } catch (cause) {
      failure ??= cause;
    }
    try {
      await target.close();
    } catch (cause) {
      failure ??= cause;
    }
    if (failure !== undefined) error.value = failure;
  }

  const open = async (target: SerialPortLike, openOptions: SerialOpenOptions): Promise<boolean> => {
    await close();
    try {
      await target.open(openOptions);
    } catch (cause) {
      error.value = cause;
      return false;
    }
    const current: ReadSession = { active: true, reader: undefined, done: Promise.resolve() };
    session = current;
    port.value = target;
    connected.value = true;
    error.value = undefined;
    target.addEventListener("disconnect", onPortDisconnect);
    current.done = readLoop(target, current);
    return true;
  };

  const write = async (data: string | Uint8Array): Promise<boolean> => {
    const writable = port.value?.writable;
    if (!writable) return false;
    const writer = writable.getWriter();
    try {
      await writer.write(typeof data === "string" ? new TextEncoder().encode(data) : data);
      return true;
    } catch (cause) {
      error.value = cause;
      return false;
    } finally {
      writer.releaseLock();
    }
  };

  const getPorts = async (): Promise<readonly SerialPortLike[]> => {
    const host = resolveHost();
    if (!host) return [];
    try {
      ports.value = await host.getPorts();
    } catch (cause) {
      error.value = cause;
    }
    return ports.value;
  };

  const requestPort = async (
    filters?: readonly SerialPortFilter[],
  ): Promise<SerialPortLike | undefined> => {
    const host = resolveHost();
    if (!host) return undefined;
    try {
      const chosen = await host.requestPort(filters === undefined ? {} : { filters });
      error.value = undefined;
      await getPorts();
      return chosen;
    } catch (cause) {
      error.value = cause;
      return undefined;
    }
  };

  const onHostChange = (): void => {
    void getPorts();
  };

  watch(
    resolveHost,
    (host, _previous, onCleanup) => {
      if (!host) {
        ports.value = [];
        return;
      }
      host.addEventListener("connect", onHostChange);
      host.addEventListener("disconnect", onHostChange);
      onCleanup(() => {
        host.removeEventListener("connect", onHostChange);
        host.removeEventListener("disconnect", onHostChange);
      });
      void getPorts();
    },
    { immediate: true, flush: "sync" },
  );

  tryOnScopeDispose(() => {
    void close();
  });

  return {
    supported: computed(() => resolveHost() !== undefined),
    ports: shallowReadonly(ports),
    port: shallowReadonly(port),
    connected: readonly(connected),
    error: readonly(error),
    requestPort,
    getPorts,
    open,
    write,
    close,
  };
}
