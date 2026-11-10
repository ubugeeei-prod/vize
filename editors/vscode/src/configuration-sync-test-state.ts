const pending = new Set<Promise<void>>();

export function trackConfigurationSyncForHostTest(sync: Promise<void>): Promise<void> {
  if (process.env.VIZE_TEST_ENABLE_HOST_COMMANDS !== "1") return sync;
  pending.add(sync);
  const settle = () => pending.delete(sync);
  void sync.then(settle, settle);
  return sync;
}

/** Settles once no settings sync runs and no debounced one is scheduled. */
export async function waitForConfigurationIdle(scheduled: () => boolean): Promise<void> {
  while (pending.size || scheduled()) {
    await (pending.size
      ? Promise.allSettled(pending)
      : new Promise((resolve) => setTimeout(resolve, 25)));
  }
}
