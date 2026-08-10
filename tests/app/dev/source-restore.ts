import * as fs from "node:fs";

export interface SourceRestoreGuard {
  detach(): void;
  markRestored(): void;
  restore(): void;
}

export interface SourceRestoreEntry {
  sourcePath: string;
  originalSource: string | null;
}

/** Restore temporarily edited source files on normal exit or cancellation. */
export function installSourceRestores(entries: readonly SourceRestoreEntry[]): SourceRestoreGuard {
  if (entries.length === 0) throw new Error("source restore requires at least one file");
  const uniquePaths = new Set(entries.map((entry) => entry.sourcePath));
  if (uniquePaths.size !== entries.length) throw new Error("source restore paths must be unique");

  let restored = false;
  const restoreEntries = () => {
    if (restored) return;
    for (const entry of entries) {
      if (entry.originalSource === null) fs.rmSync(entry.sourcePath, { force: true });
      else fs.writeFileSync(entry.sourcePath, entry.originalSource);
    }
    restored = true;
  };
  const restoreBestEffort = () => {
    try {
      restoreEntries();
    } catch {
      // Best effort during abrupt shutdown. A normal test finally block verifies bytes exactly.
    }
  };
  const signals: NodeJS.Signals[] = ["SIGINT", "SIGTERM"];
  const signalHandlers = new Map<NodeJS.Signals, () => void>();
  const detachSignals = () => {
    for (const [signal, handler] of signalHandlers) process.off(signal, handler);
  };
  const detach = () => {
    process.off("exit", restoreBestEffort);
    detachSignals();
  };

  process.on("exit", restoreBestEffort);
  for (const signal of signals) {
    const handler = () => {
      restoreBestEffort();
      detachSignals();
      if (restored) process.off("exit", restoreBestEffort);
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
    restore() {
      restoreEntries();
    },
  };
}

export function installSourceRestore(
  sourcePath: string,
  originalSource: string,
): SourceRestoreGuard {
  return installSourceRestores([{ sourcePath, originalSource }]);
}
