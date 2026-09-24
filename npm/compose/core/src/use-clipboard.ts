import { computed, readonly, ref, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import { usePermission } from "./use-permission.ts";
import type { PermissionQueryState, PermissionsHost } from "./use-permission.ts";

/** One clipboard entry, compatible with the DOM `ClipboardItem`. */
export interface ClipboardItemLike {
  /** MIME types available in this item. */
  readonly types: readonly string[];

  /** Read the item's data for one of its `types`. */
  getType(type: string): Promise<Blob>;
}

/** Minimal async Clipboard API consumed by {@link useClipboard}. */
export interface ClipboardLike {
  /** Read plain text. */
  readText(): Promise<string>;

  /** Write plain text. */
  writeText(text: string): Promise<void>;

  /** Read rich items. Absent in older browsers. */
  read?(): Promise<readonly ClipboardItemLike[]>;

  /** Write rich items. Absent in older browsers. */
  write?(items: readonly ClipboardItemLike[]): Promise<void>;
}

/** Capabilities used by {@link useClipboard}; every member is optional. */
export interface ClipboardHost {
  /** Async Clipboard API (`navigator.clipboard`). */
  readonly clipboard?: ClipboardLike | null;

  /** Permissions API used to report read/write permission state. */
  readonly permissions?: PermissionsHost | null;

  /**
   * Synchronous legacy copy used when the async API is missing or refuses.
   * Returns whether the copy succeeded.
   */
  readonly legacyCopy?: ((text: string) => boolean) | null;

  /** Target of `copy`/`cut` events observed when `listen` is enabled. */
  readonly events?: EventTarget | null;
}

/** Why a clipboard operation did not complete. */
export type ClipboardFailureReason = "permission-denied" | "unsupported" | "failed";

/** Successful clipboard operation. */
export interface ClipboardSuccess<Value> {
  /** The operation succeeded. */
  readonly status: "success";

  /** Value that was read or written. */
  readonly value: Value;

  /** Mechanism that performed the operation. */
  readonly method: "clipboard" | "legacy";
}

/** Failed clipboard operation. */
export interface ClipboardFailure {
  /** Why the operation failed. */
  readonly status: ClipboardFailureReason;

  /** Exact error thrown by the host, when one was thrown. */
  readonly error: unknown;
}

/** Discriminated result of a clipboard operation. */
export type ClipboardResult<Value> = ClipboardSuccess<Value> | ClipboardFailure;

/** Options for {@link useClipboard}. */
export interface UseClipboardOptions {
  /**
   * Clipboard capabilities for alternate runtimes and tests.
   *
   * @default navigator.clipboard, navigator.permissions, and an
   * `execCommand("copy")` fallback when a browser window exists
   */
  readonly host?: MaybeRefOrGetter<ClipboardHost | null | undefined>;

  /**
   * Fall back to `document.execCommand("copy")` when the async API is
   * unavailable or rejects a write.
   *
   * @default true
   */
  readonly legacy?: boolean;

  /**
   * Milliseconds for which `copied` stays `true` after a successful write.
   *
   * @default 1500
   */
  readonly copiedDuringMs?: number;

  /**
   * Timer host used to reset `copied`.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;

  /**
   * Update `text` by reading the clipboard after every `copy`/`cut` event.
   *
   * @default false
   */
  readonly listen?: boolean;

  /**
   * Query the `clipboard-read` / `clipboard-write` permissions.
   *
   * @default true
   */
  readonly queryPermissions?: boolean;
}

/** Reactive state and actions returned by {@link useClipboard}. */
export interface ClipboardControls {
  /** Whether the async Clipboard API or the legacy fallback is available. */
  readonly supported: ComputedRef<boolean>;

  /** Text most recently copied or read. */
  readonly text: Readonly<Ref<string>>;

  /** `true` for `copiedDuringMs` after a successful copy. */
  readonly copied: Readonly<Ref<boolean>>;

  /** Most recent failure, cleared by the next success. */
  readonly error: Readonly<ShallowRef<ClipboardFailure | undefined>>;

  /** State of the `clipboard-read` permission. */
  readonly readPermission: Readonly<Ref<PermissionQueryState>>;

  /** State of the `clipboard-write` permission. */
  readonly writePermission: Readonly<Ref<PermissionQueryState>>;

  /**
   * Copy text, falling back to the legacy mechanism when allowed.
   *
   * @param text Text to copy.
   * @returns The discriminated outcome; never rejects.
   */
  readonly copy: (text: string) => Promise<ClipboardResult<string>>;

  /**
   * Read plain text.
   *
   * @returns The discriminated outcome; never rejects.
   */
  readonly readText: () => Promise<ClipboardResult<string>>;

  /**
   * Write rich clipboard items.
   *
   * @param items Items to write.
   * @returns The discriminated outcome; never rejects.
   */
  readonly writeItems: (
    items: readonly ClipboardItemLike[],
  ) => Promise<ClipboardResult<readonly ClipboardItemLike[]>>;

  /**
   * Read rich clipboard items.
   *
   * @returns The discriminated outcome; never rejects.
   */
  readonly readItems: () => Promise<ClipboardResult<readonly ClipboardItemLike[]>>;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

function legacyDocumentCopy(document: Document): (text: string) => boolean {
  return (text) => {
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.opacity = "0";
    area.style.pointerEvents = "none";
    document.body.append(area);
    area.select();
    try {
      // `execCommand` is deprecated but remains the only synchronous copy
      // mechanism in insecure contexts and older engines.
      return document.execCommand("copy");
    } catch {
      return false;
    } finally {
      area.remove();
    }
  };
}

function browserClipboardHost(): ClipboardHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator, document } = window;
  return {
    clipboard: navigator.clipboard ?? null,
    permissions: navigator.permissions ?? null,
    legacyCopy: typeof document.execCommand === "function" ? legacyDocumentCopy(document) : null,
    events: document,
  };
}

function isPermissionError(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "name" in error &&
    (error.name === "NotAllowedError" || error.name === "SecurityError")
  );
}

function failure(error: unknown): ClipboardFailure {
  return { status: isPermissionError(error) ? "permission-denied" : "failed", error };
}

const unsupported: ClipboardFailure = { status: "unsupported", error: undefined };

/**
 * Permission-aware clipboard access with a legacy copy fallback.
 *
 * Text and rich items go through the async Clipboard API. When it is
 * missing (insecure contexts, older engines) or refuses a write, `copy`
 * falls back to `document.execCommand("copy")` unless `legacy` is false.
 * Every action resolves to a discriminated {@link ClipboardResult}
 * (`"success"`, `"permission-denied"`, `"unsupported"`, `"failed"`) instead
 * of rejecting, and the read/write permission states are exposed reactively.
 *
 * Server rendering: nothing is read and no permission is queried; `text` is
 * empty and `supported` is false. The `copied` reset timer and the optional
 * `copy`/`cut` listeners are released when the owning scope stops.
 *
 * @example
 * ```ts
 * const { copy, copied } = useClipboard();
 * await copy("npm i @vizejs/composable");
 * ```
 *
 * @param options Capabilities, fallback policy, and feedback timing.
 * @default options {}
 * @returns Reactive clipboard state and actions.
 */
export function useClipboard(options: UseClipboardOptions = {}): ClipboardControls {
  const copiedDuringMs = options.copiedDuringMs ?? 1500;
  if (!Number.isFinite(copiedDuringMs) || copiedDuringMs < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_CLIPBOARD_INVALID_DURATION] copiedDuringMs must be a non-negative finite number; received ${String(copiedDuringMs)}`,
    );
  }
  const scheduler = options.scheduler ?? defaultScheduler;
  const allowLegacy = options.legacy ?? true;
  const text = ref("");
  const copied = ref(false);
  const error = shallowRef<ClipboardFailure | undefined>(undefined);
  let resetHandle: unknown;
  let resetPending = false;

  const resolveHost = (): ClipboardHost | undefined =>
    options.host === undefined ? browserClipboardHost() : (toValue(options.host) ?? undefined);

  const queryPermissions = options.queryPermissions ?? true;
  const permissionHost = (): PermissionsHost | undefined =>
    queryPermissions ? (resolveHost()?.permissions ?? undefined) : undefined;
  const readPermission = usePermission("clipboard-read", { host: permissionHost });
  const writePermission = usePermission("clipboard-write", { host: permissionHost });

  const supported = computed(() => {
    const host = resolveHost();
    return Boolean(host?.clipboard) || Boolean(allowLegacy && host?.legacyCopy);
  });

  const settle = <Value>(result: ClipboardResult<Value>): ClipboardResult<Value> => {
    error.value = result.status === "success" ? undefined : result;
    return result;
  };

  const markCopied = (value: string): void => {
    text.value = value;
    copied.value = true;
    if (resetPending) scheduler.clearTimeout(resetHandle);
    resetPending = true;
    resetHandle = scheduler.setTimeout(() => {
      resetPending = false;
      copied.value = false;
    }, copiedDuringMs);
  };

  const tryLegacy = (host: ClipboardHost, value: string): ClipboardResult<string> | undefined => {
    if (!allowLegacy || !host.legacyCopy) return undefined;
    try {
      return host.legacyCopy(value)
        ? { status: "success", value, method: "legacy" }
        : { status: "failed", error: undefined };
    } catch (cause) {
      return failure(cause);
    }
  };

  const copy = async (value: string): Promise<ClipboardResult<string>> => {
    const host = resolveHost();
    if (!host) return settle(unsupported);
    let result: ClipboardResult<string> | undefined;
    if (host.clipboard) {
      try {
        await host.clipboard.writeText(value);
        result = { status: "success", value, method: "clipboard" };
      } catch (cause) {
        result = tryLegacy(host, value) ?? failure(cause);
      }
    } else {
      result = tryLegacy(host, value) ?? unsupported;
    }
    if (result.status === "success") markCopied(value);
    return settle(result);
  };

  const readText = async (): Promise<ClipboardResult<string>> => {
    const clipboard = resolveHost()?.clipboard;
    if (!clipboard) return settle(unsupported);
    try {
      const value = await clipboard.readText();
      text.value = value;
      return settle({ status: "success", value, method: "clipboard" });
    } catch (cause) {
      return settle(failure(cause));
    }
  };

  const writeItems = async (
    items: readonly ClipboardItemLike[],
  ): Promise<ClipboardResult<readonly ClipboardItemLike[]>> => {
    const clipboard = resolveHost()?.clipboard;
    if (!clipboard?.write) return settle(unsupported);
    try {
      await clipboard.write(items);
      return settle({ status: "success", value: items, method: "clipboard" });
    } catch (cause) {
      return settle(failure(cause));
    }
  };

  const readItems = async (): Promise<ClipboardResult<readonly ClipboardItemLike[]>> => {
    const clipboard = resolveHost()?.clipboard;
    if (!clipboard?.read) return settle(unsupported);
    try {
      const value = await clipboard.read();
      return settle({ status: "success", value, method: "clipboard" });
    } catch (cause) {
      return settle(failure(cause));
    }
  };

  if (options.listen ?? false) {
    watch(
      () => resolveHost()?.events ?? undefined,
      (events, _previous, onCleanup) => {
        if (!events) return;
        const onCopy = (): void => {
          void readText();
        };
        events.addEventListener("copy", onCopy);
        events.addEventListener("cut", onCopy);
        onCleanup(() => {
          events.removeEventListener("copy", onCopy);
          events.removeEventListener("cut", onCopy);
        });
      },
      { immediate: true, flush: "sync" },
    );
  }

  tryOnScopeDispose(() => {
    if (resetPending) scheduler.clearTimeout(resetHandle);
    resetPending = false;
  });

  return {
    supported,
    text: readonly(text),
    copied: readonly(copied),
    error: readonly(error),
    readPermission: readPermission.state,
    writePermission: writePermission.state,
    copy,
    readText,
    writeItems,
    readItems,
  };
}
