import { readonly, shallowRef, toValue, unref, watch } from "vue";
import type { MaybeRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { calculateRetryDelay } from "./retry-delay.ts";
import type { RetryDelayOptions } from "./retry-delay.ts";
import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Connection status exposed by {@link useEventSource}. */
export type EventSourceStatus = "connecting" | "open" | "closed";

/** Minimal `EventSource` instance used by {@link useEventSource}. */
export interface EventSourceLike {
  /** `0` connecting, `1` open, `2` closed (native values). */
  readonly readyState: number;
  /** Close the stream; the source never reconnects afterwards. */
  close(): void;
  /** Subscribe to `open`, `error`, or a named server event. */
  addEventListener(type: string, listener: EventListener): void;
  /** Remove a listener registered with `addEventListener`. */
  removeEventListener(type: string, listener: EventListener): void;
}

/** Constructor creating {@link EventSourceLike} streams (e.g. `EventSource`). */
export type EventSourceConstructorLike = new (
  url: string,
  init?: { readonly withCredentials?: boolean },
) => EventSourceLike;

/** Server event map: event name to decoded data type. */
export type EventSourceEventMap = Record<string, unknown>;

/** Event names of a map whose data is not plain text and therefore needs a parser. */
type ParsedEventNames<Events extends EventSourceEventMap> = {
  [Name in keyof Events & string]: [string] extends [Events[Name]] ? never : Name;
}[keyof Events & string];

/**
 * Per-event decoders. A decoder is required for every event whose data type
 * cannot hold the raw text, and optional otherwise.
 */
export type EventSourceParsers<Events extends EventSourceEventMap> = {
  readonly [Name in ParsedEventNames<Events>]: (raw: string) => Events[Name];
} & {
  readonly [Name in Exclude<keyof Events & string, ParsedEventNames<Events>>]?: (
    raw: string,
  ) => Events[Name];
};

/** One decoded server event, discriminated by `event`. */
export type EventSourceMessage<Events extends EventSourceEventMap> = {
  readonly [Name in keyof Events & string]: {
    /** Server event name. */
    readonly event: Name;
    /** Decoded data. */
    readonly data: Events[Name];
    /** `id:` field of the event, or `""`. */
    readonly lastEventId: string;
  };
}[keyof Events & string];

/** Discriminated failure reported by {@link useEventSource}. */
export type EventSourceFailure =
  | {
      /** The stream dispatched an `error` event. */
      readonly kind: "error";
      /** The dispatched event. */
      readonly event: Event;
    }
  | {
      /** The constructor threw. */
      readonly kind: "connect";
      /** Exact thrown value. */
      readonly cause: unknown;
    }
  | {
      /** A parser threw. */
      readonly kind: "parse";
      /** Event whose data could not be parsed. */
      readonly event: string;
      /** Exact thrown value. */
      readonly cause: unknown;
    }
  | {
      /** Automatic reconnection gave up. */
      readonly kind: "reconnect-exhausted";
      /** Reconnection attempts performed. */
      readonly attempts: number;
    };

/** Options for reconnecting streams the browser gave up on. */
export interface EventSourceReconnectOptions extends RetryDelayOptions {
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

/** Options for {@link useEventSource}. */
export interface UseEventSourceOptions<Events extends EventSourceEventMap> {
  /**
   * Named server events to subscribe to.
   *
   * @default ["message"]
   */
  readonly events?: readonly (keyof Events & string)[];

  /**
   * Per-event decoders (required for events whose data is not text).
   *
   * @default raw text for every event
   */
  readonly parse?: EventSourceParsers<Events>;

  /**
   * Send cookies with cross-origin requests.
   *
   * @default false
   */
  readonly withCredentials?: boolean;

  /**
   * `EventSource` constructor (a plain value or ref, never a getter).
   * Supplying one also enables automatic connection outside a browser.
   *
   * @default window.EventSource when a browser window exists
   */
  readonly host?: MaybeRef<EventSourceConstructorLike | null | undefined>;

  /**
   * Connect as soon as a URL is available (browser or explicit host only).
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Reopen the stream with backoff after the browser closes it for good
   * (native reconnection covers transient drops).
   *
   * @default false
   */
  readonly autoReconnect?: boolean | EventSourceReconnectOptions;

  /**
   * Timer host for reconnection.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;

  /**
   * Observe failures.
   *
   * @default undefined
   */
  readonly onError?: (failure: EventSourceFailure) => void;
}

/** Reactive state and controls returned by {@link useEventSource}. */
export interface EventSourceControls<Events extends EventSourceEventMap> {
  /** Connection status. */
  readonly status: Readonly<Ref<EventSourceStatus>>;
  /** Latest decoded event, discriminated by name. */
  readonly message: Readonly<ShallowRef<EventSourceMessage<Events> | undefined>>;
  /** Data of the latest event. */
  readonly data: Readonly<ShallowRef<Events[keyof Events & string] | undefined>>;
  /** Name of the latest event. */
  readonly event: Readonly<Ref<(keyof Events & string) | undefined>>;
  /** `id:` field of the latest event. */
  readonly lastEventId: Readonly<Ref<string | undefined>>;
  /** Latest failure, cleared when the stream opens. */
  readonly error: Readonly<ShallowRef<EventSourceFailure | undefined>>;
  /** Current underlying source, if any. */
  readonly source: Readonly<ShallowRef<EventSourceLike | undefined>>;
  /**
   * Observe one subscribed event with typed data.
   *
   * @returns A function removing the handler.
   */
  readonly on: <Name extends keyof Events & string>(
    event: Name,
    handler: (data: NoInfer<Events[Name]>, lastEventId: string) => void,
  ) => () => void;
  /** Open (or reopen) the stream. */
  readonly open: () => void;
  /** Close the stream and disable reconnection. */
  readonly close: () => void;
}

/**
 * Required options when the event map contains non-text data: `parse` must
 * then provide those decoders.
 */
export type EventSourceParseRequirement<Events extends EventSourceEventMap> = [
  ParsedEventNames<Events>,
] extends [never]
  ? unknown
  : { readonly parse: EventSourceParsers<Events> };

const CLOSED = 2;

function browserEventSource(): EventSourceConstructorLike | undefined {
  return typeof window !== "undefined" && typeof window.EventSource === "function"
    ? window.EventSource
    : undefined;
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

function eventData<Events extends EventSourceEventMap, Name extends keyof Events & string>(
  value: unknown,
): Events[Name] {
  // The only unchecked step: values reach this point either from the decoder
  // `EventSourceParsers<Events>[Name]` (which returns `Events[Name]`) or as
  // raw text for events whose type accepts `string` (enforced by
  // `EventSourceParseRequirement`). Indexing the mapped parser type by a
  // generic key loses that link, so it is restored here.
  return value as Events[Name];
}

/**
 * Reactive Server-Sent Events stream with typed, per-event decoding.
 *
 * Every subscribed event is decoded by its parser (raw text by default) and
 * exposed as a discriminated `message`, plus `data`/`event`/`lastEventId`
 * shortcuts and typed `on()` handlers. Transient drops use the browser's
 * native reconnection; `autoReconnect` additionally reopens streams the
 * browser closed for good. The stream and timers are released when the
 * owning reactive scope stops (the caller owns `close()` outside a scope).
 * Server rendering never connects and reports `"closed"`.
 *
 * @example
 * ```ts
 * const feed = useEventSource<{ price: number; notice: string }>("/stream", {
 *   events: ["price", "notice"],
 *   parse: { price: (raw) => Number(raw) },
 * });
 * feed.on("price", (price) => chart.push(price));
 * ```
 *
 * @param url Reactive URL; changing it reconnects.
 * @param options Events, decoders, host, and reconnection options.
 * @default options {}
 * @returns Reactive state and controls.
 */
export function useEventSource<Events extends EventSourceEventMap = { message: string }>(
  url: MaybeRefOrGetter<string | URL | null | undefined>,
  ...[options]: unknown extends EventSourceParseRequirement<Events>
    ? [options?: UseEventSourceOptions<Events>]
    : [options: UseEventSourceOptions<Events> & EventSourceParseRequirement<Events>]
): EventSourceControls<Events> {
  type Name = keyof Events & string;
  const settings: UseEventSourceOptions<Events> = options ?? {};
  const status = shallowRef<EventSourceStatus>("closed");
  const message = shallowRef<EventSourceMessage<Events>>();
  const data = shallowRef<Events[Name]>();
  const event = shallowRef<Name>();
  const lastEventId = shallowRef<string | undefined>(undefined);
  const error = shallowRef<EventSourceFailure | undefined>(undefined);
  const source = shallowRef<EventSourceLike | undefined>(undefined);
  const scheduler = settings.scheduler ?? defaultScheduler();
  const reconnect = settings.autoReconnect === true ? {} : settings.autoReconnect || undefined;
  const handlers = new Map<string, Set<(value: unknown, id: string) => void>>();
  let attempts = 0;
  let manuallyClosed = false;
  let reconnectTimer: unknown;
  let detach: (() => void) | undefined;

  const resolveHost = (): EventSourceConstructorLike | undefined =>
    settings.host === undefined ? browserEventSource() : (unref(settings.host) ?? undefined);

  const fail = (failure: EventSourceFailure): void => {
    error.value = failure;
    settings.onError?.(failure);
  };

  const dispose = (): void => {
    detach?.();
    detach = undefined;
    source.value?.close();
    source.value = undefined;
  };

  const scheduleReconnect = (): void => {
    if (!reconnect || manuallyClosed) return;
    if (attempts >= (reconnect.retries ?? Number.POSITIVE_INFINITY)) {
      fail({ kind: "reconnect-exhausted", attempts });
      reconnect.onFailed?.();
      return;
    }
    attempts += 1;
    reconnectTimer = scheduler.setTimeout(
      () => {
        reconnectTimer = undefined;
        connect();
      },
      calculateRetryDelay(attempts, reconnect),
    );
  };

  const deliver = <Current extends Name>(name: Current, raw: string, id: string): void => {
    const parsers: Partial<Record<string, (text: string) => unknown>> | undefined = settings.parse;
    const parser = parsers?.[name];
    let decoded: Events[Current];
    try {
      decoded = eventData<Events, Current>(parser ? parser(raw) : raw);
    } catch (cause) {
      fail({ kind: "parse", event: name, cause });
      return;
    }
    const next: {
      readonly event: Current;
      readonly data: Events[Current];
      readonly lastEventId: string;
    } = { event: name, data: decoded, lastEventId: id };
    message.value = next;
    data.value = decoded;
    event.value = name;
    lastEventId.value = id;
    for (const handler of handlers.get(name) ?? []) handler(decoded, id);
  };

  const connect = (): void => {
    if (reconnectTimer !== undefined) {
      scheduler.clearTimeout(reconnectTimer);
      reconnectTimer = undefined;
    }
    dispose();
    const target = toValue(url);
    const Host = resolveHost();
    if (target === null || target === undefined || Host === undefined) {
      status.value = "closed";
      return;
    }
    let instance: EventSourceLike;
    try {
      instance = new Host(String(target), { withCredentials: settings.withCredentials ?? false });
    } catch (cause) {
      status.value = "closed";
      fail({ kind: "connect", cause });
      scheduleReconnect();
      return;
    }
    source.value = instance;
    status.value = "connecting";

    const onOpen = (): void => {
      status.value = "open";
      error.value = undefined;
      attempts = 0;
    };
    const onError = (nativeEvent: Event): void => {
      fail({ kind: "error", event: nativeEvent });
      if (instance.readyState === CLOSED) {
        dispose();
        status.value = "closed";
        scheduleReconnect();
      } else {
        status.value = "connecting";
      }
    };
    const listeners = new Map<string, EventListener>();
    const names: readonly Name[] = settings.events ?? ["message"];
    for (const name of names) {
      const listener: EventListener = (nativeEvent) => {
        const raw: unknown = "data" in nativeEvent ? nativeEvent.data : "";
        const id: unknown = "lastEventId" in nativeEvent ? nativeEvent.lastEventId : "";
        deliver(
          name,
          typeof raw === "string" ? raw : String(raw),
          typeof id === "string" ? id : "",
        );
      };
      listeners.set(name, listener);
      instance.addEventListener(name, listener);
    }
    instance.addEventListener("open", onOpen);
    instance.addEventListener("error", onError);
    detach = () => {
      instance.removeEventListener("open", onOpen);
      instance.removeEventListener("error", onError);
      for (const [name, listener] of listeners) instance.removeEventListener(name, listener);
    };
  };

  const open = (): void => {
    manuallyClosed = false;
    attempts = 0;
    connect();
  };

  const close = (): void => {
    manuallyClosed = true;
    if (reconnectTimer !== undefined) {
      scheduler.clearTimeout(reconnectTimer);
      reconnectTimer = undefined;
    }
    dispose();
    status.value = "closed";
  };

  const on = <Current extends Name>(
    name: Current,
    handler: (value: Events[Current], id: string) => void,
  ): (() => void) => {
    const wrapped = (value: unknown, id: string): void => {
      // Handlers are keyed by event name and only receive that event's data.
      handler(eventData<Events, Current>(value), id);
    };
    const set = handlers.get(name) ?? new Set();
    set.add(wrapped);
    handlers.set(name, set);
    return () => {
      set.delete(wrapped);
    };
  };

  const automatic = settings.host !== undefined || typeof window !== "undefined";
  if (automatic) {
    let initial = true;
    const stop = watch(
      [() => toValue(url), resolveHost],
      ([target]) => {
        const first = initial;
        initial = false;
        if (first && !(settings.immediate ?? true)) return;
        if (!first && source.value === undefined && reconnectTimer === undefined) return;
        if (target === null || target === undefined) close();
        else open();
      },
      { immediate: true },
    );
    tryOnScopeDispose(stop);
  }

  tryOnScopeDispose(close);

  return {
    status: readonly(status),
    message,
    data,
    event,
    lastEventId,
    error,
    source,
    on,
    open,
    close,
  };
}
