import type { NativeHook, JsonValue, HookIdentity } from "./stages.js";

/** A standalone function expression. It receives (contextOrBatch, configuration).
 * Closures and host imports are unavailable; process access stays in the container.
 */
export type CallbackSource = string;
export interface SandboxIdentity extends HookIdentity {
  visit?: readonly string[];
  demands?: readonly string[];
  configuration?: JsonValue;
}
export type SandboxDefinition = SandboxIdentity &
  (
    | { family?: "rule"; rules: Readonly<Record<string, CallbackSource>> }
    | { family: "provider"; provides: readonly string[]; callback: CallbackSource }
    | { family: "transform" | "formatter" | "output"; callback: CallbackSource }
  );
export interface SandboxLimits {
  /** 100..60000 ms; default 5000. Runtime check and cleanup each add at most 5000 ms. */
  timeoutMs?: number;
  /** 1024..4194304 bytes for serialized request and each output stream; default 1 MiB. */
  maxBytes?: number;
}
/** Requires Docker and a pre-pulled SANDBOX_IMAGE. There are no host mounts or
 * host environment variables, network is disabled, root is read-only, and each
 * invocation has fixed memory/CPU/PID limits and confirmed container cleanup.
 * Runtime filesystem contents remain readable. Native callbacks are trusted.
 */
export type SandboxHook = NativeHook & { readonly visit?: string[]; readonly demands?: string[] };
export declare function createSandboxRunner(
  definition: SandboxIdentity & {
    family: "provider";
    provides: readonly string[];
    callback: CallbackSource;
  },
  limits?: SandboxLimits,
): SandboxHook & { readonly provides: string[] };
export declare function createSandboxRunner(
  definition: SandboxIdentity & { family: "formatter" | "output"; callback: CallbackSource },
  limits?: SandboxLimits,
): SandboxHook & { readonly family: "formatter" | "output" };
export declare function createSandboxRunner(
  definition: SandboxDefinition,
  limits?: SandboxLimits,
): SandboxHook;
export type SandboxErrorCode =
  | "runtime_unavailable"
  | "execution_stopped"
  | "execution_failed"
  | "cleanup_failed";
/** Available decoded output can be partial after a stopped execution. */
export interface SandboxExecutionObservation {
  readonly phase: "run";
  readonly elapsedMs: number;
  readonly status: number | null;
  readonly signal: string | null;
  readonly stdout: string | null;
  readonly stderr: string | null;
}
export declare class SandboxRuntimeError extends Error {
  readonly code: SandboxErrorCode;
  /** Frozen and non-enumerable; the request and environment are not copied into it. */
  readonly execution?: SandboxExecutionObservation;
  constructor(code: SandboxErrorCode, message: string, options?: ErrorOptions);
}
export declare const SANDBOX_IMAGE: string;
