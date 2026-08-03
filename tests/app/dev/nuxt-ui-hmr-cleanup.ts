import * as fs from "node:fs";

export interface SourceRestoreGuard {
  detach(): void;
  markRestored(): void;
}

/** Restore a temporarily edited source file on normal exit or cancellation. */
export function installSourceRestore(
  sourcePath: string,
  originalSource: string,
): SourceRestoreGuard {
  let restored = false;
  const restore = () => {
    if (restored) return;
    try {
      fs.writeFileSync(sourcePath, originalSource);
      restored = true;
    } catch {
      // Best effort during abrupt shutdown.
    }
  };
  const signals: NodeJS.Signals[] = ["SIGINT", "SIGTERM"];
  const signalHandlers = new Map<NodeJS.Signals, () => void>();
  const detachSignals = () => {
    for (const [signal, handler] of signalHandlers) process.off(signal, handler);
  };
  const detach = () => {
    process.off("exit", restore);
    detachSignals();
  };

  process.on("exit", restore);
  for (const signal of signals) {
    const handler = () => {
      restore();
      detachSignals();
      if (restored) process.off("exit", restore);
      // Re-raise the exact signal after restoring so termination semantics stay intact.
      process.kill(process.pid, signal);
    };
    signalHandlers.set(signal, handler);
    process.on(signal, handler);
  }

  return {
    detach,
    markRestored() {
      restored = true;
    },
  };
}
