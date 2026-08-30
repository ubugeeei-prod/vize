/** Execute every notification before surfacing one or more consumer errors. */
export function notifyAll(notifications: readonly (() => void)[]): void {
  const errors: unknown[] = [];
  for (const notify of notifications) {
    try {
      notify();
    } catch (error) {
      errors.push(error);
    }
  }
  if (errors.length === 1) throw errors[0];
  if (errors.length > 1) throw new AggregateError(errors, "Press callbacks failed");
}

/** Capture one consumer failure while allowing the lifecycle to settle. */
export function captureError(errors: unknown[], callback: () => void): void {
  try {
    callback();
  } catch (error) {
    errors.push(error);
  }
}

/** Surface captured failures after every required transition has run. */
export function surfaceErrors(errors: readonly unknown[]): void {
  if (errors.length === 0) return;
  notifyAll(
    errors.map((error) => () => {
      throw error;
    }),
  );
}
