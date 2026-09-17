const pending = new Set<Promise<void>>();

export function trackAutoInsertForHostTest(task: Promise<void>): Promise<void> {
  if (process.env.VIZE_TEST_ENABLE_HOST_COMMANDS !== "1") return task;
  pending.add(task);
  void task.then(
    () => pending.delete(task),
    () => pending.delete(task),
  );
  return task;
}

export async function waitForAutoInsertIdle(): Promise<void> {
  while (pending.size) await Promise.all(pending);
}
