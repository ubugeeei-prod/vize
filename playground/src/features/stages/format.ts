// Display formatting for measured compiler work.

/**
 * Wall time for display. The browser clock is quantized (5 µs when the page
 * is cross-origin isolated), so sub-microsecond values read as "under 1 µs".
 */
export function formatNanos(nanos: number): string {
  if (nanos < 1_000) return "<1 µs";
  if (nanos < 1_000_000) return `${Math.round(nanos / 1_000)} µs`;
  return `${(nanos / 1_000_000).toFixed(2)} ms`;
}
