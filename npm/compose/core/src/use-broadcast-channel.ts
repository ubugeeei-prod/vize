import { readonly, ref, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Minimal structural view of a `BroadcastChannel` instance. */
export interface BroadcastChannelLike extends EventTarget {
  /** Post a structured-cloneable message to every other channel with the same name. */
  readonly postMessage: (message: unknown) => void;

  /** Close the channel; no further messages are delivered. */
  readonly close: () => void;
}

/** Constructor creating channels, compatible with `window.BroadcastChannel`. */
export type BroadcastChannelConstructor = new (name: string) => BroadcastChannelLike;

/**
 * Capability host exposing a channel constructor; `window` satisfies it.
 *
 * The constructor is wrapped in an object because a bare class would be
 * mistaken for a getter by `MaybeRefOrGetter` resolution.
 */
export interface BroadcastChannelHost {
  /** Channel constructor. */
  readonly BroadcastChannel: BroadcastChannelConstructor;
}

/** Stable failure codes reported by {@link useBroadcastChannel}. */
export type BroadcastChannelErrorCode =
  | "open-failed"
  | "post-failed"
  | "message-error"
  | "invalid-message";

/** Failure observed while using a broadcast channel. */
export interface BroadcastChannelFailure {
  /** Which step failed. */
  readonly code: BroadcastChannelErrorCode;

  /** Channel name involved in the failure. */
  readonly name: string;

  /** Exact thrown value, event, or rejected payload. */
  readonly cause: unknown;
}

/** Options for {@link useBroadcastChannel}. */
export interface UseBroadcastChannelOptions<Message> {
  /**
   * Host exposing the channel constructor. `null`/`undefined` keeps the
   * composable closed, which is also what happens during server rendering.
   *
   * @default window when it exposes `BroadcastChannel`
   */
  readonly host?: MaybeRefOrGetter<BroadcastChannelHost | null | undefined>;

  /**
   * Validate incoming payloads. Rejected payloads leave `data` untouched and
   * report an `"invalid-message"` failure.
   *
   * @default every payload is accepted
   */
  readonly validate?: (data: unknown) => data is Message;

  /**
   * Observe every accepted message.
   *
   * @default undefined
   */
  readonly onMessage?: (message: Message) => void;

  /**
   * Observe failures. Failures never throw out of the composable.
   *
   * @default undefined
   */
  readonly onError?: (failure: BroadcastChannelFailure) => void;
}

/** Reactive state and controls returned by {@link useBroadcastChannel}. */
export interface BroadcastChannelControls<Message> {
  /** Most recent accepted message received from another context. */
  readonly data: Readonly<ShallowRef<Message | undefined>>;

  /** Whether a channel constructor is available. */
  readonly supported: Readonly<Ref<boolean>>;

  /** Whether no channel is currently open. */
  readonly closed: Readonly<Ref<boolean>>;

  /** Most recent failure, cleared by the next successful post or message. */
  readonly error: Readonly<ShallowRef<BroadcastChannelFailure | undefined>>;

  /**
   * Post a typed message.
   *
   * @param message Structured-cloneable payload.
   * @returns Whether the message was handed to an open channel.
   */
  readonly post: (message: Message) => boolean;

  /** Close the channel and stop following the reactive name. Idempotent. */
  readonly close: () => void;
}

function browserBroadcastChannelHost(): BroadcastChannelHost | undefined {
  return typeof window !== "undefined" && typeof window.BroadcastChannel === "function"
    ? window
    : undefined;
}

/**
 * Exchange typed messages with other tabs, windows, and workers of the same
 * origin through the Broadcast Channel API.
 *
 * The channel follows the reactive `name`: changing it closes the old channel
 * and opens a new one. The channel closes when the owning reactive scope
 * stops or when `close()` is called; outside a scope the caller owns
 * `close()`. Incoming payloads pass the `validate` type guard before they
 * reach `data`, so foreign messages cannot widen the declared type.
 *
 * Server rendering: without a browser `window` no channel is opened (Node's
 * global `BroadcastChannel` is ignored), `supported` is `false`, and `post`
 * returns `false`.
 *
 * @example
 * ```ts
 * type Sync = { readonly type: "logout" };
 * const { data, post } = useBroadcastChannel<Sync>("auth");
 * post({ type: "logout" });
 * ```
 *
 * @typeParam Message Payload type exchanged over the channel.
 * @param name Reactive channel name.
 * @param options Host, validation, and observation hooks.
 * @default options {}
 * @returns Reactive channel state and controls.
 */
export function useBroadcastChannel<Message>(
  name: MaybeRefOrGetter<string>,
  options: UseBroadcastChannelOptions<Message> = {},
): BroadcastChannelControls<Message> {
  const data = shallowRef<Message | undefined>(undefined);
  const error = shallowRef<BroadcastChannelFailure | undefined>(undefined);
  const supported = ref(false);
  const closed = ref(true);
  let channel: BroadcastChannelLike | undefined;
  let stopped = false;

  const fail = (code: BroadcastChannelErrorCode, cause: unknown): void => {
    const failure: BroadcastChannelFailure = { code, name: toValue(name), cause };
    error.value = failure;
    options.onError?.(failure);
  };

  const resolveHost = (): BroadcastChannelConstructor | undefined =>
    (options.host === undefined ? browserBroadcastChannelHost() : toValue(options.host))
      ?.BroadcastChannel;

  const onMessage = (event: Event): void => {
    const payload: unknown = event instanceof MessageEvent ? event.data : undefined;
    if (options.validate !== undefined && !options.validate(payload)) {
      fail("invalid-message", payload);
      return;
    }
    // Without a validator the declared message type is trusted, as for
    // `postMessage` peers of the same application.
    const message = payload as Message;
    error.value = undefined;
    data.value = message;
    options.onMessage?.(message);
  };
  const onMessageError = (event: Event): void => fail("message-error", event);

  const stopWatch = watch(
    [() => toValue(name), resolveHost],
    ([channelName, Host], _previous, onCleanup) => {
      supported.value = Host !== undefined;
      if (Host === undefined || stopped) return;
      let opened: BroadcastChannelLike;
      try {
        opened = new Host(channelName);
      } catch (cause) {
        fail("open-failed", cause);
        return;
      }
      channel = opened;
      closed.value = false;
      opened.addEventListener("message", onMessage);
      opened.addEventListener("messageerror", onMessageError);
      onCleanup(() => {
        opened.removeEventListener("message", onMessage);
        opened.removeEventListener("messageerror", onMessageError);
        opened.close();
        if (channel === opened) channel = undefined;
        closed.value = true;
      });
    },
    { immediate: true, flush: "sync" },
  );

  const post = (message: Message): boolean => {
    if (channel === undefined) return false;
    try {
      channel.postMessage(message);
    } catch (cause) {
      fail("post-failed", cause);
      return false;
    }
    error.value = undefined;
    return true;
  };

  const close = (): void => {
    stopped = true;
    stopWatch();
  };

  tryOnScopeDispose(close);

  return {
    data,
    supported: readonly(supported),
    closed: readonly(closed),
    error,
    post,
    close,
  };
}
