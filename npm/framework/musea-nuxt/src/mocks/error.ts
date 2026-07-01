import { getErrorRef, setError } from "../context.js";

export interface NuxtError extends Error {
  data?: unknown;
  fatal?: boolean;
  statusCode?: number;
  statusMessage?: string;
}

export function createError(input: string | Partial<NuxtError>): NuxtError {
  const message =
    typeof input === "string" ? input : (input.message ?? input.statusMessage ?? "Error");
  const error = new Error(message) as NuxtError;
  if (typeof input === "object") {
    Object.assign(error, input);
  }
  return error;
}

export function showError(input: string | Partial<NuxtError>): NuxtError {
  const error = createError(input);
  setError(error);
  return error;
}

export function clearError(_options?: { redirect?: string }): Promise<void> {
  setError(null);
  return Promise.resolve();
}

export function useError() {
  return getErrorRef();
}
