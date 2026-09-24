import { readonly, shallowRef, toValue, unref, watch } from "vue";
import type { MaybeRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { calculateRetryDelay } from "./retry-delay.ts";
import type { RetryDelayOptions } from "./retry-delay.ts";
import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Raw payload delivered by a WebSocket `message` event. */
export type WebSocketRawData = string | ArrayBuffer | Blob;

/** Payload accepted by `WebSocket.send`. */
export type WebSocketSendData = string | Blob | BufferSource;

/** Connection status exposed by {@link useWebSocket}. */
export type WebSocketStatus = "connecting" | "open" | "closing" | "closed";

/** Minimal WebSocket instance used by {@link useWebSocket}. */
export interface WebSocketLike {
  /** Transmit data over the open connection. */
  readonly send: (data: WebSocketSendData) => void;
  /** Start the closing handshake. */
  readonly close: (code?: number, reason?: string) => void;
  /** Subscribe to `open`, `message`, `error`, or `close`. */
  addEventListener(type: "open" | "message" | "error" | "close", listener: EventListener): void;
  /** Remove a listener registered with `addEventListener`. */
  removeEventListener(type: "open" | "message" | "error" | "close", listener: EventListener): void;
}

/** Constructor creating {@link WebSocketLike} connections (e.g. `WebSocket`). */
export type WebSocketConstructorLike = new (
  url: string,
  protocols?: string | string[],
) => WebSocketLike;

/** Discriminated failure reported by {@link useWebSocket}. */
export type WebSocketFailure =
  | {
      /** The socket dispatched an `error` event. */
      readonly kind: "error";
      /** The dispatched event. */
      readonly event: Event;
    }
  | {
      /** The constructor threw (invalid URL, blocked port, ...). */
      readonly kind: "connect";
      /** Exact thrown value. */
      readonly cause: unknown;
    }
  | {
      /** `parse` threw for an incoming message. */
      readonly kind: "parse";
      /** Exact thrown value. */
      readonly cause: unknown;
    }
  | {
      /** `validate` rejected a parsed message. */
      readonly kind: "invalid";
      /** Rejected message. */
      readonly data: unknown;
    }
  | {
      /** Automatic reconnection gave up. */
      readonly kind: "reconnect-exhausted";
      /** Reconnection attempts performed. */
      readonly attempts: number;
    };

/** Options for automatic reconnection. */
export interface WebSocketReconnectOptions extends RetryDelayOptions {
  /**
   * Maximum consecutive reconnection attempts.
   *
   * @default Infinity
   */
  readonly retries?: number;

  /**
   * Called once reconnection gives up.
   *
   * @default undefined
   */
  readonly onFailed?: () => void;
}

/** Options for the keep-alive heartbeat. */
export interface WebSocketHeartbeatOptions {
  /**
   * Raw ping payload.
   *
   * @default "ping"
   */
  readonly message?: WebSocketSendData;

  /**
   * Incoming payload treated as a pong and not exposed as data.
   *
   * @default the ping message when it is a string
   */
  readonly responseMessage?: string;

  /**
   * Delay between pings in milliseconds.
   *
   * @default 30000
   */
  readonly intervalMs?: number;

  /**
   * Close (and possibly reconnect) when no message arrives this long after a ping.
   *
   * @default 10000
   */
  readonly pongTimeoutMs?: number;
}

/**
 * Options for {@link useWebSocket}.
 *
 * Callbacks that consume message types use method syntax so options typed
 * for concrete messages remain assignable to the implementation signature.
 */
export interface UseWebSocketOptions<Incoming, Outgoing> {
  /**
   * Sub-protocols requested from the server.
   *
   * @default undefined
   */
  readonly protocols?: string | string[];

  /**
   * WebSocket constructor (a plain value or ref; never a getter, because a
   * constructor is itself a function). Supplying one also enables automatic
   * connection outside a browser.
   *
   * @default window.WebSocket when a browser window exists
   */
  readonly host?: MaybeRef<WebSocketConstructorLike | null | undefined>;

  /**
   * Connect as soon as a URL is available (browser or explicit host only).
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Reconnect after unexpected closes with exponential backoff.
   *
   * @default false
   */
  readonly autoReconnect?: boolean | WebSocketReconnectOptions;

  /**
   * Send periodic pings and drop connections whose peer stops answering.
   *
   * @default false
   */
  readonly heartbeat?: boolean | WebSocketHeartbeatOptions;

  /**
   * Decode incoming payloads.
   *
   * @default the raw payload
   */
  readonly parse?: (raw: WebSocketRawData) => Incoming;

  /**
   * Encode outgoing messages.
   *
   * @default sends payloads as-is and JSON-encodes anything else
   */
  serialize?(message: Outgoing): WebSocketSendData;

  /**
   * Reject parsed messages that fail this check (`"invalid"` failure).
   *
   * @default accepts every message
   */
  validate?(message: Incoming): boolean;

  /**
   * Queue messages sent while connecting and flush them on open.
   *
   * @default true
   */
  readonly bufferWhileConnecting?: boolean;

  /**
   * Timer host for reconnection and heartbeat.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;

  /**
   * Observe every accepted incoming message.
   *
   * @default undefined
   */
  onMessage?(message: Incoming, event: Event): void;

  /**
   * Observe failures.
   *
   * @default undefined
   */
  readonly onError?: (failure: WebSocketFailure) => void;
}

/** Reactive state and controls returned by {@link useWebSocket}. */
export interface WebSocketControls<Incoming, Outgoing> {
  /** Connection status. */
  readonly status: Readonly<Ref<WebSocketStatus>>;
  /** Latest accepted incoming message. */
  readonly data: Readonly<ShallowRef<Incoming | undefined>>;
  /** Latest failure, cleared when a connection opens. */
  readonly error: Readonly<ShallowRef<WebSocketFailure | undefined>>;
  /** Consecutive reconnection attempts since the last successful open. */
  readonly reconnectAttempts: Readonly<Ref<number>>;
  /** Current underlying socket, if any. */
  readonly socket: Readonly<ShallowRef<WebSocketLike | undefined>>;
  /**
   * Send a message, or queue it while connecting.
   *
   * @returns Whether the message was sent or queued.
   */
  send(message: Outgoing): boolean;
  /** Open (or reopen) the connection and re-enable reconnection. */
  readonly open: () => void;
  /** Close the connection and disable automatic reconnection. */
  readonly close: (code?: number, reason?: string) => void;
}

function browserWebSocket(): WebSocketConstructorLike | undefined {
  return typeof window !== "undefined" && typeof window.WebSocket === "function"
    ? window.WebSocket
    : undefined;
}

function toSendData(value: unknown): WebSocketSendData | undefined {
  if (typeof value === "string" || value instanceof ArrayBuffer) return value;
  if (typeof Blob !== "undefined" && value instanceof Blob) return value;
  if (ArrayBuffer.isView(value) && value.buffer instanceof ArrayBuffer) {
    return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
  }
  return undefined;
}

function isRawData(value: unknown): value is WebSocketRawData {
  return (
    typeof value === "string" ||
    value instanceof ArrayBuffer ||
    (typeof Blob !== "undefined" && value instanceof Blob)
  );
}

function defaultScheduler(): TimeoutScheduler {
  return {
    setTimeout: (callback, delayMs) => {
      const handle = globalThis.setTimeout(callback, delayMs);
      return () => globalThis.clearTimeout(handle);
    },
    clearTimeout: (cancel) => {
      if (typeof cancel === "function") cancel();
    },
  };
}

/**
 * Open a WebSocket with raw payloads.
 *
 * @param url Reactive URL; `null`/`undefined` keeps the socket closed.
 * @param options Connection options.
 * @returns Reactive state and controls.
 */
export function useWebSocket(
  url: MaybeRefOrGetter<string | URL | null | undefined>,
  options?: UseWebSocketOptions<WebSocketRawData, WebSocketSendData>,
): WebSocketControls<WebSocketRawData, WebSocketSendData>;
/**
 * Open a WebSocket with typed messages. `parse` is required unless raw
 * payloads fit `Incoming`; `serialize` is required unless `Outgoing` is
 * already sendable.
 *
 * @param url Reactive URL; `null`/`undefined` keeps the socket closed.
 * @param options Connection options with codecs.
 * @returns Reactive state and controls.
 */
export function useWebSocket<Incoming = WebSocketRawData, Outgoing = WebSocketSendData>(
  url: MaybeRefOrGetter<string | URL | null | undefined>,
  options: UseWebSocketOptions<Incoming, Outgoing> & WebSocketCodecRequirements<Incoming, Outgoing>,
): WebSocketControls<Incoming, Outgoing>;
/**
 * Reactive WebSocket with typed messages, auto-reconnect, and heartbeat.
 *
 * Unexpected closes reconnect with exponential backoff when `autoReconnect`
 * is enabled; `close()` is always final until `open()` is called again.
 * Messages sent while connecting are queued. The connection and all timers
 * are released when the owning reactive scope stops; outside a scope the
 * caller owns `close()`. Server rendering never connects: automatic
 * connection requires a browser or an explicit `host`, and status stays
 * `"closed"`.
 *
 * @example
 * ```ts
 * const chat = useWebSocket<ChatEvent, ChatCommand>("wss://example.test/chat", {
 *   parse: (raw) => parseChatEvent(String(raw)),
 *   serialize: (command) => JSON.stringify(command),
 *   autoReconnect: { retries: 5 },
 *   heartbeat: true,
 * });
 * chat.send({ type: "join", room: "general" });
 * ```
 *
 * @param url Reactive URL; changing it reconnects.
 * @param options Codecs, reconnection, heartbeat, and host options.
 * @default options {}
 * @returns Reactive state and controls.
 */
export function useWebSocket(
  url: MaybeRefOrGetter<string | URL | null | undefined>,
  options: UseWebSocketOptions<unknown, unknown> = {},
): WebSocketControls<unknown, unknown> {
  const status = shallowRef<WebSocketStatus>("closed");
  const data = shallowRef<unknown>(undefined);
  const error = shallowRef<WebSocketFailure | undefined>(undefined);
  const reconnectAttempts = shallowRef(0);
  const socket = shallowRef<WebSocketLike | undefined>(undefined);
  const scheduler = options.scheduler ?? defaultScheduler();
  const reconnect = options.autoReconnect === true ? {} : options.autoReconnect || undefined;
  const heartbeat = options.heartbeat === true ? {} : options.heartbeat || undefined;
  const pingMessage = heartbeat?.message ?? "ping";
  const pongMessage =
    heartbeat?.responseMessage ?? (typeof pingMessage === "string" ? pingMessage : undefined);
  const buffer: WebSocketSendData[] = [];
  let manuallyClosed = false;
  let reconnectTimer: unknown;
  let pingTimer: unknown;
  let pongTimer: unknown;
  let detach: (() => void) | undefined;

  const resolveHost = (): WebSocketConstructorLike | undefined =>
    options.host === undefined ? browserWebSocket() : (unref(options.host) ?? undefined);

  const fail = (failure: WebSocketFailure): void => {
    error.value = failure;
    options.onError?.(failure);
  };

  const clearTimer = (handle: unknown): undefined => {
    if (handle !== undefined) scheduler.clearTimeout(handle);
    return undefined;
  };

  const stopHeartbeat = (): void => {
    pingTimer = clearTimer(pingTimer);
    pongTimer = clearTimer(pongTimer);
  };

  const serialize = (message: unknown): WebSocketSendData =>
    options.serialize
      ? options.serialize(message)
      : (toSendData(message) ?? JSON.stringify(message));

  const schedulePing = (target: WebSocketLike): void => {
    if (!heartbeat) return;
    pingTimer = scheduler.setTimeout(() => {
      pingTimer = undefined;
      target.send(pingMessage);
      pongTimer = clearTimer(pongTimer);
      pongTimer = scheduler.setTimeout(() => {
        pongTimer = undefined;
        target.close();
      }, heartbeat.pongTimeoutMs ?? 10_000);
      schedulePing(target);
    }, heartbeat.intervalMs ?? 30_000);
  };

  const teardown = (): void => {
    stopHeartbeat();
    detach?.();
    detach = undefined;
  };

  const scheduleReconnect = (): void => {
    if (!reconnect || manuallyClosed) return;
    const retries = reconnect.retries ?? Number.POSITIVE_INFINITY;
    if (reconnectAttempts.value >= retries) {
      fail({ kind: "reconnect-exhausted", attempts: reconnectAttempts.value });
      reconnect.onFailed?.();
      return;
    }
    reconnectAttempts.value += 1;
    const delay = calculateRetryDelay(reconnectAttempts.value, reconnect);
    reconnectTimer = scheduler.setTimeout(() => {
      reconnectTimer = undefined;
      connect();
    }, delay);
  };

  const connect = (): void => {
    reconnectTimer = clearTimer(reconnectTimer);
    const current = socket.value;
    if (current) {
      teardown();
      current.close();
      socket.value = undefined;
    }
    const target = toValue(url);
    const Host = resolveHost();
    if (target === null || target === undefined || Host === undefined) {
      status.value = "closed";
      return;
    }
    let instance: WebSocketLike;
    try {
      instance =
        options.protocols === undefined
          ? new Host(String(target))
          : new Host(String(target), options.protocols);
    } catch (cause) {
      status.value = "closed";
      fail({ kind: "connect", cause });
      scheduleReconnect();
      return;
    }
    socket.value = instance;
    status.value = "connecting";

    const onOpen = (): void => {
      status.value = "open";
      error.value = undefined;
      reconnectAttempts.value = 0;
      for (const queued of buffer.splice(0)) instance.send(queued);
      schedulePing(instance);
    };
    const onMessage = (event: Event): void => {
      pongTimer = clearTimer(pongTimer);
      const raw: unknown = "data" in event ? event.data : undefined;
      if (!isRawData(raw)) return;
      if (heartbeat && pongMessage !== undefined && raw === pongMessage) return;
      let message: unknown;
      try {
        message = options.parse ? options.parse(raw) : raw;
      } catch (cause) {
        fail({ kind: "parse", cause });
        return;
      }
      if (options.validate && !options.validate(message)) {
        fail({ kind: "invalid", data: message });
        return;
      }
      data.value = message;
      options.onMessage?.(message, event);
    };
    const onError = (event: Event): void => fail({ kind: "error", event });
    const onClose = (): void => {
      teardown();
      if (socket.value === instance) socket.value = undefined;
      status.value = "closed";
      scheduleReconnect();
    };
    instance.addEventListener("open", onOpen);
    instance.addEventListener("message", onMessage);
    instance.addEventListener("error", onError);
    instance.addEventListener("close", onClose);
    detach = () => {
      instance.removeEventListener("open", onOpen);
      instance.removeEventListener("message", onMessage);
      instance.removeEventListener("error", onError);
      instance.removeEventListener("close", onClose);
    };
  };

  const open = (): void => {
    manuallyClosed = false;
    reconnectAttempts.value = 0;
    connect();
  };

  const close = (code?: number, reason?: string): void => {
    manuallyClosed = true;
    reconnectTimer = clearTimer(reconnectTimer);
    buffer.length = 0;
    const current = socket.value;
    teardown();
    socket.value = undefined;
    if (!current) {
      status.value = "closed";
      return;
    }
    status.value = "closing";
    current.close(code, reason);
    status.value = "closed";
  };

  const send = (message: unknown): boolean => {
    const payload = serialize(message);
    if (status.value === "open" && socket.value) {
      socket.value.send(payload);
      return true;
    }
    if (status.value === "connecting" && (options.bufferWhileConnecting ?? true)) {
      buffer.push(payload);
      return true;
    }
    return false;
  };

  const automatic = options.host !== undefined || typeof window !== "undefined";
  if (automatic) {
    let initial = true;
    const stop = watch(
      [() => toValue(url), resolveHost],
      ([target]) => {
        const first = initial;
        initial = false;
        if (first && !(options.immediate ?? true)) return;
        if (!first && socket.value === undefined && reconnectTimer === undefined) return;
        if (target === null || target === undefined) close();
        else open();
      },
      { immediate: true },
    );
    tryOnScopeDispose(stop);
  }

  tryOnScopeDispose(() => close());

  return {
    status: readonly(status),
    data,
    error,
    reconnectAttempts: readonly(reconnectAttempts),
    socket,
    send,
    open,
    close,
  };
}

/**
 * Codec options that become mandatory when the defaults cannot produce the
 * message types: `parse` unless raw payloads fit `Incoming`, `serialize`
 * unless `Outgoing` is already sendable.
 */
export type WebSocketCodecRequirements<Incoming, Outgoing> = ([WebSocketRawData] extends [Incoming]
  ? unknown
  : { readonly parse: (raw: WebSocketRawData) => Incoming }) &
  ([Outgoing] extends [WebSocketSendData]
    ? unknown
    : { readonly serialize: (message: Outgoing) => WebSocketSendData });
