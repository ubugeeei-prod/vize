import { computed, shallowRef } from "vue";
import type { ComputedRef, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Lifecycle state of an asynchronous resource. */
export type AsyncResourceStatus = "idle" | "pending" | "success" | "error" | "cancelled";

/** Context supplied to an asynchronous resource loader. */
export interface AsyncResourceContext {
  /** Signal aborted by cancellation, reset, scope disposal, or a newer execution. */
  readonly signal: AbortSignal;
}

/** Explicit result of one asynchronous resource execution. */
export type AsyncResourceExecution<Data, Failure> =
  | { readonly status: "success"; readonly data: Data }
  | { readonly status: "error"; readonly error: Failure }
  | { readonly status: "cancelled"; readonly reason: unknown }
  | { readonly status: "superseded" };

/** Options for {@link useAsyncResource}. */
export interface UseAsyncResourceOptions<Data> {
  /**
   * Initial data restored by {@link AsyncResource.reset}.
   *
   * @default undefined
   */
  readonly initialData?: Data;

  /**
   * Abort the active execution when a newer execution starts.
   *
   * @default true
   */
  readonly cancelPrevious?: boolean;

  /**
   * Retain the current data while a new execution is pending.
   *
   * @default true
   */
  readonly keepData?: boolean;

  /**
   * Cancel an active execution when the current reactive scope is disposed.
   *
   * @default true
   */
  readonly scope?: boolean;
}

/** Reactive state and controls for an asynchronous loader. */
export interface AsyncResource<Data, Arguments extends readonly unknown[], Failure> {
  readonly data: Readonly<ShallowRef<Data | undefined>>;
  readonly error: Readonly<ShallowRef<Failure | undefined>>;
  readonly status: Readonly<Ref<AsyncResourceStatus>>;
  readonly pending: ComputedRef<boolean>;
  readonly execute: (...arguments_: Arguments) => Promise<AsyncResourceExecution<Data, Failure>>;
  readonly cancel: (reason?: unknown) => boolean;
  readonly reset: () => void;
}

interface ActiveExecution {
  readonly generation: number;
  readonly controller: AbortController;
  superseded: boolean;
}

/**
 * Create a scoped, abortable asynchronous resource with latest-result-wins
 * state. Every execution returns a discriminated result, so cancellation,
 * supersession, loader failure, and successful `undefined` data stay distinct.
 */
export function useAsyncResource<Data, Arguments extends readonly unknown[], Failure = unknown>(
  loader: (context: AsyncResourceContext, ...arguments_: Arguments) => Promise<Data>,
  options: UseAsyncResourceOptions<Data> = {},
): AsyncResource<Data, Arguments, Failure> {
  const data = shallowRef<Data | undefined>(options.initialData);
  const error = shallowRef<Failure | undefined>(undefined);
  const status = shallowRef<AsyncResourceStatus>("idle");
  const pending = computed(() => status.value === "pending");
  let generation = 0;
  let active: ActiveExecution | undefined;

  const cancel = (reason: unknown = createAbortReason("The execution was cancelled.")) => {
    if (active === undefined) return false;
    generation += 1;
    active.controller.abort(reason);
    active = undefined;
    status.value = "cancelled";
    return true;
  };

  const execute = async (...arguments_: Arguments) => {
    if ((options.cancelPrevious ?? true) && active !== undefined) {
      active.superseded = true;
      active.controller.abort(createAbortReason("A newer execution started."));
    }
    const record: ActiveExecution = {
      generation: ++generation,
      controller: new AbortController(),
      superseded: false,
    };
    active = record;
    error.value = undefined;
    status.value = "pending";
    if (!(options.keepData ?? true)) data.value = undefined;

    try {
      const result = await loader({ signal: record.controller.signal }, ...arguments_);
      if (record.generation !== generation) return executionAfterInvalidation(record);
      data.value = result;
      status.value = "success";
      return { status: "success", data: result } as const;
    } catch (cause) {
      if (record.generation !== generation || record.controller.signal.aborted) {
        return executionAfterInvalidation(record);
      }
      error.value = cause as Failure;
      status.value = "error";
      return { status: "error", error: cause as Failure } as const;
    } finally {
      if (active === record) active = undefined;
    }
  };

  const reset = () => {
    cancel(createAbortReason("The resource was reset."));
    data.value = options.initialData;
    error.value = undefined;
    status.value = "idle";
  };

  if (options.scope ?? true) {
    tryOnScopeDispose(() => cancel(createAbortReason("The reactive scope was disposed.")));
  }

  return { data, error, status, pending, execute, cancel, reset };
}

function executionAfterInvalidation<Data, Failure>(
  execution: ActiveExecution,
): AsyncResourceExecution<Data, Failure> {
  if (execution.superseded || !execution.controller.signal.aborted) {
    return { status: "superseded" };
  }
  return { status: "cancelled", reason: execution.controller.signal.reason };
}

function createAbortReason(message: string): DOMException {
  return new DOMException(message, "AbortError");
}
