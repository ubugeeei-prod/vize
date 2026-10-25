import type { PluginTracer } from "@parcel/types";

/** Synchronous native boundaries appear as individual operations in Parcel traces. */
export function nativeOperation<T>(
  tracer: PluginTracer,
  name: string,
  filename: string,
  action: () => T,
): T {
  if (!tracer.enabled) return action();
  const measurement = tracer.createMeasurement(`napi:${name}`, "native", filename);
  try {
    return action();
  } finally {
    measurement?.end();
  }
}
