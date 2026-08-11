import { tryOnScopeDispose } from "./scope.ts";

/** Stable error code reported when one or more owned cleanups fail. */
export const DISPOSAL_ERROR_CODE = "VIZE_COMPOSE_DISPOSAL_FAILED" as const;

/** Aggregate failure produced after every owned cleanup has been attempted. */
export class DisposalError extends AggregateError {
  /** Stable machine-readable error code. */
  readonly code = DISPOSAL_ERROR_CODE;

  /**
   * Create an aggregate disposal failure.
   *
   * Nested {@link DisposalError} instances are flattened by the scope before
   * construction, so `errors` contains the original cleanup failures in
   * execution order.
   *
   * @param errors Cleanup failures in deterministic execution order.
   */
  constructor(errors: readonly unknown[]) {
    super(errors, `[${DISPOSAL_ERROR_CODE}] ${String(errors.length)} cleanup operation(s) failed.`);
    this.name = "DisposalError";
  }
}

/** Handle for removing one cleanup from its owner before disposal. */
export interface CleanupRegistration {
  /** Whether this registration is still retained by its owner. */
  readonly active: boolean;

  /**
   * Release the cleanup without running it.
   *
   * @returns Whether an active registration was removed.
   */
  readonly unregister: () => boolean;
}

/** Options for {@link createDisposalScope}. */
export interface CreateDisposalScopeOptions {
  /**
   * Dispose the owner with the active Vue reactive scope when one exists.
   * Outside a reactive scope, ownership remains entirely with the caller.
   *
   * @default true
   */
  readonly scope?: boolean;
}

/** Explicit owner for a deterministic group of cleanup operations. */
export interface DisposalScope {
  /** Whether disposal has started or completed. */
  readonly disposed: boolean;

  /** Number of active cleanup registrations currently retained. */
  readonly size: number;

  /**
   * Register one cleanup.
   *
   * Active registrations run in last-in, first-out order. After disposal,
   * registration runs the cleanup immediately and returns an inactive handle;
   * a late failure is wrapped in {@link DisposalError}.
   *
   * @param cleanup Synchronous, idempotent resource cleanup.
   * @returns A handle that can release the cleanup without running it.
   * @throws {DisposalError} A late cleanup failed after the owner was disposed.
   */
  readonly add: <Cleanup extends () => unknown>(
    cleanup: [ReturnType<Cleanup>] extends [never]
      ? Cleanup
      : ReturnType<Cleanup> extends PromiseLike<unknown>
        ? never
        : Cleanup,
  ) => CleanupRegistration;

  /**
   * Create a child whose lifetime cannot outlive this owner.
   *
   * Disposing the child early unregisters it from the parent. Disposing the
   * parent disposes the child in normal LIFO order.
   *
   * @returns A new child disposal scope.
   */
  readonly child: () => DisposalScope;

  /**
   * Run every active cleanup in last-in, first-out order exactly once.
   *
   * Reentrant and late registrations run immediately. All cleanups are
   * attempted even when some fail. Repeated calls after successful or failed
   * disposal do nothing.
   *
   * @throws {DisposalError} One or more cleanup operations failed.
   */
  readonly dispose: () => void;
}

interface CleanupRecord {
  active: boolean;
  readonly cleanup: () => unknown;
}

/**
 * Create an explicit, deterministic cleanup owner.
 *
 * The owner is runtime-neutral and performs no browser-global access. By
 * default it joins an active Vue reactive scope; when no scope exists, or
 * `scope` is disabled, the caller must invoke `dispose()`. Cleanups are
 * synchronous by contract so disposal completes before the owner releases
 * its records.
 *
 * @param options Reactive-scope ownership behavior.
 * @default options {}
 * @returns A disposal owner with child-lifetime support.
 */
export function createDisposalScope(options: CreateDisposalScopeOptions = {}): DisposalScope {
  const owner = createOwner();
  if (options.scope ?? true) tryOnScopeDispose(owner.dispose);
  return owner;
}

function createOwner(onDispose?: () => void): DisposalScope {
  const records: CleanupRecord[] = [];
  let disposed = false;
  let size = 0;

  const add = (cleanup: () => unknown): CleanupRegistration => {
    if (disposed) {
      try {
        cleanup();
      } catch (error) {
        throw new DisposalError(flattenDisposalError(error));
      }
      return inactiveRegistration();
    }

    const record: CleanupRecord = { active: true, cleanup };
    records.push(record);
    size += 1;
    return {
      get active() {
        return record.active;
      },
      unregister() {
        if (!record.active) return false;
        record.active = false;
        size -= 1;
        return true;
      },
    };
  };

  const dispose = () => {
    if (disposed) return;
    disposed = true;
    onDispose?.();

    const errors: unknown[] = [];
    for (let index = records.length - 1; index >= 0; index -= 1) {
      const record = records[index];
      if (!record?.active) continue;
      record.active = false;
      size -= 1;
      try {
        record.cleanup();
      } catch (error) {
        errors.push(...flattenDisposalError(error));
      }
    }
    records.length = 0;
    if (errors.length > 0) throw new DisposalError(errors);
  };

  const child = () => {
    let parentRegistration: CleanupRegistration | undefined;
    const childOwner = createOwner(() => parentRegistration?.unregister());
    parentRegistration = add(childOwner.dispose);
    return childOwner;
  };

  return {
    get disposed() {
      return disposed;
    },
    get size() {
      return size;
    },
    add,
    child,
    dispose,
  };
}

function inactiveRegistration(): CleanupRegistration {
  return { active: false, unregister: () => false };
}

function flattenDisposalError(error: unknown): readonly unknown[] {
  return error instanceof DisposalError ? error.errors : [error];
}
