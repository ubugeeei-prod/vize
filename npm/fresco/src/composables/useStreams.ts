/**
 * Stream composables matching Ink's stdin/stdout/stderr helpers.
 */

import { inject, type InjectionKey } from "vue";

const BRACKETED_PASTE_ENABLE = "\x1B[?2004h";
const BRACKETED_PASTE_DISABLE = "\x1B[?2004l";

/** Injection key for application streams and the terminal-mode controller. */
export const STREAMS_KEY: InjectionKey<StreamsContext> = Symbol("fresco-streams");

/** Host streams, output writers, and interactive terminal-mode policy. */
export interface StreamsContextOptions {
  /** Input stream whose terminal modes the context manages. @default process.stdin */
  stdin?: NodeJS.ReadStream;
  /** Output stream used for terminal output and mode control. @default process.stdout */
  stdout?: NodeJS.WriteStream;
  /** Error output stream. @default process.stderr */
  stderr?: NodeJS.WriteStream;
  /** Application exit policy exposed to input composables. @default true */
  exitOnCtrlC?: boolean;
  /** Whether raw-mode and bracketed-paste mode changes are permitted. @default true */
  interactive?: boolean;
  /** Writer for the configured output stream. @default data => stdout.write(data) */
  writeToStdout?: (data: string) => void;
  /** Writer for the configured error stream. @default data => stderr.write(data) */
  writeToStderr?: (data: string) => void;
}

/** Shared streams with reference-counted raw-mode and bracketed-paste controls. */
export interface StreamsContext {
  stdin: NodeJS.ReadStream;
  stdout: NodeJS.WriteStream;
  stderr: NodeJS.WriteStream;
  setRawMode: (isRawMode: boolean) => void;
  isRawModeSupported: boolean;
  setBracketedPasteMode: (isEnabled: boolean) => void;
  writeToStdout: (data: string) => void;
  writeToStderr: (data: string) => void;
  internal_exitOnCtrlC: boolean;
}

/** Input stream, raw-mode capability, and application exit policy. */
export interface UseStdinReturn {
  stdin: NodeJS.ReadStream;
  setRawMode: (isRawMode: boolean) => void;
  isRawModeSupported: boolean;
  internal_exitOnCtrlC: boolean;
}

/** Output stream and its configured writer. */
export interface UseStdoutReturn {
  stdout: NodeJS.WriteStream;
  write: (data: string) => void;
}

/** Error stream and its configured writer. */
export interface UseStderrReturn {
  stderr: NodeJS.WriteStream;
  write: (data: string) => void;
}

/** Create stream controls that balance nested mode requests and defer final raw-mode release to a microtask. */
export function createStreamsContext(options: StreamsContextOptions = {}): StreamsContext {
  const stdin = options.stdin ?? process.stdin;
  const stdout = options.stdout ?? process.stdout;
  const stderr = options.stderr ?? process.stderr;
  const isInteractive = options.interactive ?? true;
  const writeToStdout = options.writeToStdout ?? ((data: string) => stdout.write(data));
  const writeToStderr = options.writeToStderr ?? ((data: string) => stderr.write(data));
  let rawModeDepth = 0;
  let pendingRawModeDisable = false;
  let bracketedPasteDepth = 0;

  return {
    stdin,
    stdout,
    stderr,
    setRawMode: (isRawMode: boolean) => {
      if (!isInteractive || typeof stdin.setRawMode !== "function") return;

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
    isRawModeSupported: isInteractive && typeof stdin.setRawMode === "function",
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
    writeToStdout,
    writeToStderr,
    internal_exitOnCtrlC: options.exitOnCtrlC ?? true,
  };
}

/** Read the injected stream context, or create one for process streams outside an application. */
export function useStreamsContext(): StreamsContext {
  return inject(STREAMS_KEY) ?? createStreamsContext();
}

/** Read the application input stream and raw-mode controls. */
export function useStdin(): UseStdinReturn {
  const streams = useStreamsContext();

  return {
    stdin: streams.stdin,
    setRawMode: streams.setRawMode,
    isRawModeSupported: streams.isRawModeSupported,
    internal_exitOnCtrlC: streams.internal_exitOnCtrlC,
  };
}

/** Read the application output stream and configured output writer. */
export function useStdout(): UseStdoutReturn {
  const { stdout, writeToStdout } = useStreamsContext();

  return {
    stdout,
    write: writeToStdout,
  };
}

/** Read the application error stream and configured error writer. */
export function useStderr(): UseStderrReturn {
  const { stderr, writeToStderr } = useStreamsContext();

  return {
    stderr,
    write: writeToStderr,
  };
}
