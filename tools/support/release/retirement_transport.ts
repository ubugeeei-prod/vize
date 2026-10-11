import { setTimeout as pause } from "node:timers/promises";
import type { AbsenceTarget } from "./retirement_evidence.ts";

const temporaryCodes = new Set([
  "ECONNRESET",
  "ETIMEDOUT",
  "EAI_AGAIN",
  "UND_ERR_CONNECT_TIMEOUT",
  "UND_ERR_SOCKET",
]);

function details(error: unknown) {
  let complete = true;
  const seen = new Set<unknown>(),
    codes: string[] = [],
    messages: string[] = [];
  const visit = (value: unknown, depth: number) => {
    if (seen.has(value) || depth > 4 || seen.size >= 16) {
      complete = false;
      return;
    }
    seen.add(value);
    if (!(value instanceof Error)) {
      complete = false;
      messages.push(String(value).slice(0, 512));
      return;
    }
    const code = "code" in value && typeof value.code === "string" ? value.code : undefined;
    if (code) codes.push(code);
    messages.push(
      `${value.name.slice(0, 64)}: ${value.message.slice(0, 512)}${code ? ` [${code.slice(0, 80)}]` : ""}`,
    );
    if (!code && value.cause === undefined && !(value instanceof AggregateError)) complete = false;
    if (value.cause !== undefined) visit(value.cause, depth + 1);
    if (value instanceof AggregateError) {
      if (value.errors.length > 8 || value.errors.length === 0) complete = false;
      for (const child of value.errors.slice(0, 8)) visit(child, depth + 1);
    }
  };
  visit(error, 0);
  return { codes, messages, complete };
}

/** Retry only rejected transport reads within the caller's unchanged abort deadline. */
export function retirementReader(target: AbsenceTarget, fetchImpl: typeof globalThis.fetch) {
  return async (...args: Parameters<typeof globalThis.fetch>): Promise<Response> => {
    const failures: string[][] = [];
    const signal = args[1]?.signal;
    const refusal = (error: unknown, attempt: number) =>
      new Error(
        `retirement transport failed: ${target.channel} ${target.name} at ${target.url}; attempts=${attempt}; failures=${JSON.stringify(failures)}`,
        { cause: error },
      );
    for (let attempt = 1; ; attempt++) {
      try {
        signal?.throwIfAborted();
        return await fetchImpl(...args);
      } catch (error) {
        const failure = details(error);
        failures.push(failure.messages);
        if (
          attempt >= 3 ||
          signal?.aborted ||
          !failure.complete ||
          failure.codes.length === 0 ||
          !failure.codes.every((code) => temporaryCodes.has(code))
        ) {
          throw refusal(error, attempt);
        }
        try {
          await pause(250 * attempt, undefined, { signal: signal ?? undefined });
        } catch (error) {
          failures.push(details(error).messages);
          throw refusal(error, attempt);
        }
      }
    }
  };
}
