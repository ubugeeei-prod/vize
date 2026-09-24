import { readonly, shallowRef, toValue } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Lifecycle status of a {@link useWorkerFn} worker. */
export type WorkerFnStatus = "idle" | "running" | "success" | "error" | "timeout";

/** Minimal dedicated worker used by {@link useWorkerFn}. */
export interface WorkerLike {
  /** Send a structured-clonable message to the worker. */
  postMessage(message: unknown): void;
  /** Stop the worker immediately. */
  terminate(): void;
  /** Subscribe to `message`, `messageerror`, or `error`. */
  addEventListener(type: "message" | "messageerror" | "error", listener: EventListener): void;
  /** Remove a listener registered with `addEventListener`. */
  removeEventListener(type: "message" | "messageerror" | "error", listener: EventListener): void;
}

/** Constructor creating {@link WorkerLike} instances from a script URL. */
export type WorkerConstructorLike = new (url: string) => WorkerLike;

/** Capabilities needed to run a function in a Blob-URL worker. */
export interface WorkerFnHost {
  /** Worker constructor (e.g. `Worker`). */
  readonly Worker: WorkerConstructorLike;
  /** Create a URL for the generated worker script (e.g. `URL.createObjectURL`). */
  readonly createObjectURL: (script: Blob) => string;
  /** Release a URL returned by `createObjectURL`. */
  readonly revokeObjectURL: (url: string) => void;
}

/** Stable error codes used in {@link useWorkerFn} rejection messages. */
export type WorkerFnErrorCode =
  | "VIZE_COMPOSE_WORKER_FN_UNSUPPORTED"
  | "VIZE_COMPOSE_WORKER_FN_BUSY"
  | "VIZE_COMPOSE_WORKER_FN_TIMEOUT"
  | "VIZE_COMPOSE_WORKER_FN_TERMINATED"
  | "VIZE_COMPOSE_WORKER_FN_FAILED";

/** Options for {@link useWorkerFn}. */
export interface UseWorkerFnOptions {
  /**
   * Reject and terminate a run after this many milliseconds.
   *
   * @default undefined (no timeout)
   */
  readonly timeoutMs?: number;

  /**
   * Absolute script URLs loaded with `importScripts` before the function runs.
   *
   * @default []
   */
  readonly dependencies?: readonly string[];

  /**
   * Named standalone functions inlined into the worker so `fn` can call them.
   *
   * @default []
   */
  readonly localDependencies?: readonly ((...arguments_: never[]) => unknown)[];

  /**
   * Worker capabilities for alternate runtimes and tests.
   *
   * @default window.Worker and window.URL when a browser window exists
   */
  readonly host?: MaybeRefOrGetter<WorkerFnHost | null | undefined>;

  /**
   * Timer host for `timeoutMs`.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;
}

/** Reactive state and controls returned by {@link useWorkerFn}. */
export interface WorkerFnControls<Arguments extends readonly unknown[], Result> {
  /** Lifecycle status of the latest run. */
  readonly status: Readonly<Ref<WorkerFnStatus>>;
  /** Whether worker capabilities are available. */
  readonly supported: Readonly<Ref<boolean>>;
  /**
   * Run the function in a fresh worker. Arguments and result must be
   * structured-clonable. Rejects with a tagged `Error` whose message starts
   * with a {@link WorkerFnErrorCode}.
   */
  readonly run: (...arguments_: Arguments) => Promise<Awaited<Result>>;
  /** Terminate the running worker, rejecting its pending run. */
  readonly terminate: () => void;
}

function browserHost(): WorkerFnHost | undefined {
  if (typeof window === "undefined" || typeof window.Worker !== "function") return undefined;
  const { URL: url } = window;
  return {
    Worker: window.Worker,
    createObjectURL: (script) => url.createObjectURL(script),
    revokeObjectURL: (target) => url.revokeObjectURL(target),
  };
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

function workerError(code: WorkerFnErrorCode, message: string, cause?: unknown): Error {
  return cause === undefined
    ? new Error(`[${code}] ${message}`)
    : new Error(`[${code}] ${message}`, { cause });
}

/**
 * Build the worker script for `fn`. Exposed for inspection and testing.
 *
 * The function source is embedded with `Function.prototype.toString`, so it
 * must be self-contained: no closures over module variables, only its
 * arguments, `localDependencies`, `dependencies`, and worker globals.
 *
 * @param fn Function to embed.
 * @param options Imported scripts and inlined helper functions.
 * @default options {}
 * @returns JavaScript source for a dedicated worker.
 */
export function createWorkerFnScript(
  fn: (...arguments_: never[]) => unknown,
  options: Pick<UseWorkerFnOptions, "dependencies" | "localDependencies"> = {},
): string {
  const imports = (options.dependencies ?? []).map((url) => JSON.stringify(url)).join(", ");
  const helpers = (options.localDependencies ?? [])
    .map((helper) => {
      if (helper.name === "") {
        throw new TypeError(
          "[VIZE_COMPOSE_WORKER_FN_ANONYMOUS_DEPENDENCY] localDependencies must be named functions",
        );
      }
      return `const ${helper.name} = ${helper.toString()};`;
    })
    .join("\n");
  return [
    imports === "" ? "" : `importScripts(${imports});`,
    helpers,
    `const __vizeWorkerFn = (${fn.toString()});`,
    "self.onmessage = async (event) => {",
    "  try {",
    '    self.postMessage(["success", await __vizeWorkerFn(...event.data)]);',
    "  } catch (error) {",
    '    self.postMessage(["error", error instanceof Error ? error.message : String(error)]);',
    "  }",
    "};",
  ]
    .filter((line) => line !== "")
    .join("\n");
}

/**
 * Run a CPU-heavy function in a dedicated Blob-URL worker, off the main thread.
 *
 * Each `run` spawns a fresh worker, posts the arguments, and resolves with the
 * function's (awaited) result; the worker is terminated and its URL revoked
 * once the run settles, times out, or is terminated. Concurrent runs reject
 * with `VIZE_COMPOSE_WORKER_FN_BUSY`. The running worker is terminated when
 * the owning reactive scope stops. Server rendering never spawns workers:
 * `supported` is `false` and `run` rejects with
 * `VIZE_COMPOSE_WORKER_FN_UNSUPPORTED`.
 *
 * @example
 * ```ts
 * const { run, status } = useWorkerFn((values: number[]) =>
 *   [...values].sort((a, b) => a - b),
 * );
 * const sorted = await run(bigArray);
 * ```
 *
 * @param fn Self-contained function; see {@link createWorkerFnScript}.
 * @param options Timeout, dependencies, host, and scheduler.
 * @default options {}
 * @throws {RangeError} `[VIZE_COMPOSE_WORKER_FN_INVALID_TIMEOUT]` for an invalid timeout.
 * @returns Reactive status and run controls.
 */
export function useWorkerFn<Arguments extends readonly unknown[], Result>(
  fn: (...arguments_: Arguments) => Result,
  options: UseWorkerFnOptions = {},
): WorkerFnControls<Arguments, Result> {
  const { timeoutMs } = options;
  if (timeoutMs !== undefined && (!Number.isFinite(timeoutMs) || timeoutMs < 0)) {
    throw new RangeError(
      `[VIZE_COMPOSE_WORKER_FN_INVALID_TIMEOUT] timeoutMs must be a finite non-negative number; received ${String(timeoutMs)}`,
    );
  }
  const status = shallowRef<WorkerFnStatus>("idle");
  const resolveHost = (): WorkerFnHost | undefined =>
    options.host === undefined ? browserHost() : (toValue(options.host) ?? undefined);
  const supported = shallowRef(resolveHost() !== undefined);
  const scheduler = options.scheduler ?? defaultScheduler();
  let cancelActive: ((error: Error, next: WorkerFnStatus) => void) | undefined;

  const run = (...arguments_: Arguments): Promise<Awaited<Result>> => {
    const host = resolveHost();
    supported.value = host !== undefined;
    if (host === undefined) {
      return Promise.reject(
        workerError("VIZE_COMPOSE_WORKER_FN_UNSUPPORTED", "workers are not available"),
      );
    }
    if (cancelActive !== undefined) {
      return Promise.reject(
        workerError("VIZE_COMPOSE_WORKER_FN_BUSY", "a run is already in progress"),
      );
    }
    const script = createWorkerFnScript(fn, options);
    const url = host.createObjectURL(new Blob([script], { type: "text/javascript" }));
    let worker: WorkerLike;
    try {
      worker = new host.Worker(url);
    } catch (cause) {
      host.revokeObjectURL(url);
      status.value = "error";
      return Promise.reject(
        workerError("VIZE_COMPOSE_WORKER_FN_FAILED", "the worker could not start", cause),
      );
    }
    status.value = "running";

    return new Promise<Awaited<Result>>((resolve, reject) => {
      let timer: unknown;
      const cleanup = (): void => {
        cancelActive = undefined;
        if (timer !== undefined) scheduler.clearTimeout(timer);
        worker.removeEventListener("message", onMessage);
        worker.removeEventListener("messageerror", onFailure);
        worker.removeEventListener("error", onFailure);
        worker.terminate();
        host.revokeObjectURL(url);
      };
      const settleError = (error: Error, next: WorkerFnStatus): void => {
        cleanup();
        status.value = next;
        reject(error);
      };
      const onMessage: EventListener = (event) => {
        const payload: unknown = "data" in event ? event.data : undefined;
        if (!Array.isArray(payload) || payload.length !== 2) return;
        const [kind, value]: readonly unknown[] = payload;
        if (kind === "success") {
          cleanup();
          status.value = "success";
          resolve(workerResult<Result>(value));
        } else {
          settleError(workerError("VIZE_COMPOSE_WORKER_FN_FAILED", String(value)), "error");
        }
      };
      const onFailure: EventListener = (event) => {
        const message =
          "message" in event && typeof event.message === "string" ? event.message : event.type;
        settleError(workerError("VIZE_COMPOSE_WORKER_FN_FAILED", message, event), "error");
      };
      cancelActive = settleError;
      worker.addEventListener("message", onMessage);
      worker.addEventListener("messageerror", onFailure);
      worker.addEventListener("error", onFailure);
      if (timeoutMs !== undefined) {
        timer = scheduler.setTimeout(() => {
          timer = undefined;
          settleError(
            workerError(
              "VIZE_COMPOSE_WORKER_FN_TIMEOUT",
              `the run exceeded ${String(timeoutMs)} ms`,
            ),
            "timeout",
          );
        }, timeoutMs);
      }
      worker.postMessage(arguments_);
    });
  };

  const terminate = (): void => {
    cancelActive?.(
      workerError("VIZE_COMPOSE_WORKER_FN_TERMINATED", "the worker was terminated"),
      "idle",
    );
  };

  tryOnScopeDispose(terminate);

  return { status: readonly(status), supported: readonly(supported), run, terminate };
}

function workerResult<Result>(value: unknown): Awaited<Result> {
  // The worker executes `fn` itself and posts back its awaited return value
  // through structured cloning; the type cannot be re-checked at runtime.
  return value as Awaited<Result>;
}
