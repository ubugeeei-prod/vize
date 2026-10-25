import type { PluginTracer } from "@parcel/types";
import { randomUUID } from "node:crypto";

/** Synchronous native boundaries appear as individual operations in Parcel traces. */
export function nativeOperation<T>(
  tracer: PluginTracer,
  name: string,
  filename: string,
  action: () => T,
): T {
  if (!tracer.enabled) return action();
  // Parcel can deliver one worker event to several ReporterRunner listeners.
  // A per-call ID lets consumers retain every raw event without counting copies.
  const measurement = tracer.createMeasurement(`napi:${name}`, "native", filename, {
    callId: randomUUID(),
  });
  try {
    return action();
  } finally {
    measurement?.end();
  }
}
