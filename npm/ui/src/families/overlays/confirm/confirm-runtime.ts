import { computed, shallowRef } from "vue";
import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  ConfirmAction,
  ConfirmApi,
  ConfirmChooseOptions,
  ConfirmOptions,
  ConfirmRequest,
} from "./confirm-types.ts";

const optionDiagnostic = "VIZE_UI_CONFIRM_OPTION";
const providerDiagnostic = "VIZE_UI_CONFIRM_PROVIDER_MISSING";

/** Value a `confirm` request's accepting action resolves with. */
export const confirmActionValue = "confirm";

interface ConfirmEntry {
  readonly request: ConfirmRequest;
  readonly settle: (value: string | null) => void;
}

/** Labels applied when a request does not name its own. */
export interface ConfirmQueueDefaults {
  readonly confirmLabel: () => string;
  readonly cancelLabel: () => string;
}

/** FIFO queue of confirmation requests owned by one ConfirmProvider. */
export interface ConfirmQueue {
  /** Request currently on screen, or `null`. */
  readonly active: ComputedRef<ConfirmRequest | null>;

  /** Number of queued requests including the active one. */
  readonly pending: ComputedRef<number>;

  /** Queue one normalized request and resolve with the chosen value or `null`. */
  readonly enqueue: (request: Omit<ConfirmRequest, "id">) => Promise<string | null>;

  /** Settle the active request. Values that are not one of its actions are ignored. */
  readonly settleActive: (value: string | null) => boolean;

  /** Settle every queued request as cancelled. */
  readonly cancelAll: () => void;

  /** Cancel everything and make later requests resolve `null` immediately. */
  readonly dispose: () => void;

  /** Public promise API bound to this queue. */
  readonly api: ConfirmApi;
}

function assertTitle(title: unknown): void {
  if (typeof title !== "string" || title.trim().length === 0) {
    throw new TypeError(`${optionDiagnostic}: title must be a non-empty string`);
  }
}

function assertActions(actions: readonly ConfirmAction[]): void {
  if (actions.length === 0) {
    throw new TypeError(`${optionDiagnostic}: choose requires at least one action`);
  }
  const values = new Set<string>();
  for (const action of actions) {
    if (typeof action.value !== "string" || action.value.length === 0 || values.has(action.value)) {
      throw new TypeError(`${optionDiagnostic}: action values must be unique non-empty strings`);
    }
    values.add(action.value);
  }
}

/** Create an SSR-safe confirmation queue. It touches no DOM and starts no timers. */
export function createConfirmQueue(defaults: ConfirmQueueDefaults): ConfirmQueue {
  const entries = shallowRef<readonly ConfirmEntry[]>([]);
  const active = computed(() => entries.value[0]?.request ?? null);
  const pending = computed(() => entries.value.length);
  let sequence = 0;
  let disposed = false;

  const remove = (entry: ConfirmEntry): void => {
    entries.value = entries.value.filter((candidate) => candidate !== entry);
  };

  const enqueue = (input: Omit<ConfirmRequest, "id">): Promise<string | null> => {
    if (disposed) return Promise.resolve(null);
    sequence += 1;
    const request: ConfirmRequest = Object.freeze({ ...input, id: `confirm-request-${sequence}` });
    return new Promise<string | null>((resolve) => {
      let settled = false;
      const entry: ConfirmEntry = {
        request,
        settle: (value) => {
          if (settled) return;
          settled = true;
          remove(entry);
          resolve(value);
        },
      };
      entries.value = [...entries.value, entry];
    });
  };

  const settleActive = (value: string | null): boolean => {
    const entry = entries.value[0];
    if (entry === undefined) return false;
    if (value !== null && !entry.request.actions.some((action) => action.value === value)) {
      return false;
    }
    entry.settle(value);
    return true;
  };

  const cancelAll = (): void => {
    for (const entry of entries.value) entry.settle(null);
  };

  const confirm = async (options: ConfirmOptions): Promise<boolean> => {
    assertTitle(options.title);
    const destructive = options.destructive === true;
    const result = await enqueue({
      actions: [
        {
          destructive,
          label: options.confirmLabel ?? defaults.confirmLabel(),
          value: confirmActionValue,
        },
      ],
      cancelLabel: options.cancelLabel ?? defaults.cancelLabel(),
      data: options.data,
      description: options.description ?? null,
      destructive,
      kind: "confirm",
      title: options.title,
    });
    return result === confirmActionValue;
  };

  const choose = async <const Value extends string>(
    options: ConfirmChooseOptions<Value>,
  ): Promise<Value | null> => {
    assertTitle(options.title);
    assertActions(options.actions);
    const result = await enqueue({
      actions: options.actions.map((action) => ({
        destructive: action.destructive === true,
        label: action.label,
        value: action.value,
      })),
      cancelLabel: options.cancelLabel ?? defaults.cancelLabel(),
      data: options.data,
      description: options.description ?? null,
      destructive: options.actions.some((action) => action.destructive === true),
      kind: "choose",
      title: options.title,
    });
    return options.actions.find((action) => action.value === result)?.value ?? null;
  };

  return Object.freeze({
    active,
    api: Object.freeze({ cancelAll, choose, confirm, pending }),
    cancelAll,
    dispose: () => {
      disposed = true;
      cancelAll();
    },
    enqueue,
    pending,
    settleActive,
  });
}

/** Context carrying the nearest ConfirmProvider queue API. */
export const confirmContext = createContext<ConfirmApi>("Confirm");

/**
 * Read the nearest ConfirmProvider's promise API.
 *
 * `Data` types the `data` payload callers attach to requests; custom renderers
 * receive it as `unknown` and narrow it themselves.
 *
 * @throws {Error} `VIZE_UI_CONFIRM_PROVIDER_MISSING` outside a ConfirmProvider.
 */
export function useConfirm<Data = unknown>(): ConfirmApi<Data> {
  const api = confirmContext.useOptional();
  if (api === undefined) {
    throw new Error(`${providerDiagnostic}: useConfirm requires a ConfirmProvider ancestor`);
  }
  return {
    cancelAll: api.cancelAll,
    choose: (options) => api.choose(options),
    confirm: (options) => api.confirm(options),
    pending: api.pending,
  };
}
