/**
 * Stream composables matching Ink's stdin/stdout/stderr helpers.
 */

import { inject, type InjectionKey } from "@vue/runtime-core";

const BRACKETED_PASTE_ENABLE = "\x1B[?2004h";
const BRACKETED_PASTE_DISABLE = "\x1B[?2004l";

export const STREAMS_KEY: InjectionKey<StreamsContext> = Symbol("fresco-streams");

export interface StreamsContextOptions {
  stdin?: NodeJS.ReadStream;
  stdout?: NodeJS.WriteStream;
  stderr?: NodeJS.WriteStream;
  exitOnCtrlC?: boolean;
  interactive?: boolean;
}

export interface StreamsContext {
  stdin: NodeJS.ReadStream;
  stdout: NodeJS.WriteStream;
  stderr: NodeJS.WriteStream;
  setRawMode: (isRawMode: boolean) => void;
  isRawModeSupported: boolean;
  setBracketedPasteMode: (isEnabled: boolean) => void;
  internal_exitOnCtrlC: boolean;
}

export interface UseStdinReturn {
  stdin: NodeJS.ReadStream;
  setRawMode: (isRawMode: boolean) => void;
  isRawModeSupported: boolean;
  internal_exitOnCtrlC: boolean;
}

export interface UseStdoutReturn {
  stdout: NodeJS.WriteStream;
  write: (data: string) => void;
}

export interface UseStderrReturn {
  stderr: NodeJS.WriteStream;
  write: (data: string) => void;
}

export function createStreamsContext(options: StreamsContextOptions = {}): StreamsContext {
  const stdin = options.stdin ?? process.stdin;
  const stdout = options.stdout ?? process.stdout;
  const stderr = options.stderr ?? process.stderr;
  const isInteractive = options.interactive ?? true;
  let rawModeDepth = 0;
  let pendingRawModeDisable = false;
  let bracketedPasteDepth = 0;

  return {
    stdin,
    stdout,
    stderr,
    setRawMode: (isRawMode: boolean) => {
      if (typeof stdin.setRawMode !== "function") return;

      stdin.setEncoding?.("utf8");

      if (isRawMode) {
        pendingRawModeDisable = false;
        if (rawModeDepth === 0) {
          stdin.ref?.();
          stdin.setRawMode(true);
        }
        rawModeDepth += 1;
        return;
      }

      if (rawModeDepth === 0) return;
      rawModeDepth -= 1;
      if (rawModeDepth > 0) return;

      pendingRawModeDisable = true;
      queueMicrotask(() => {
        if (!pendingRawModeDisable || rawModeDepth > 0) return;
        pendingRawModeDisable = false;
        stdin.setRawMode?.(false);
        stdin.unref?.();
      });
    },
    isRawModeSupported: typeof stdin.setRawMode === "function",
    setBracketedPasteMode: (isEnabled: boolean) => {
      if (!isInteractive) return;

      if (isEnabled) {
        bracketedPasteDepth += 1;
        if (bracketedPasteDepth === 1) stdout.write(BRACKETED_PASTE_ENABLE);
        return;
      }

      if (bracketedPasteDepth === 0) return;
      bracketedPasteDepth -= 1;
      if (bracketedPasteDepth === 0) stdout.write(BRACKETED_PASTE_DISABLE);
    },
    internal_exitOnCtrlC: options.exitOnCtrlC ?? true,
  };
}

export function useStreamsContext(): StreamsContext {
  return inject(STREAMS_KEY) ?? createStreamsContext();
}

export function useStdin(): UseStdinReturn {
  const streams = useStreamsContext();

  return {
    stdin: streams.stdin,
    setRawMode: streams.setRawMode,
    isRawModeSupported: streams.isRawModeSupported,
    internal_exitOnCtrlC: streams.internal_exitOnCtrlC,
  };
}

export function useStdout(): UseStdoutReturn {
  const { stdout } = useStreamsContext();

  return {
    stdout,
    write: (data: string) => {
      stdout.write(data);
    },
  };
}

export function useStderr(): UseStderrReturn {
  const { stderr } = useStreamsContext();

  return {
    stderr,
    write: (data: string) => {
      stderr.write(data);
    },
  };
}
