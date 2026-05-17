/**
 * Stream composables matching Ink's stdin/stdout/stderr helpers.
 */

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

export function useStdin(): UseStdinReturn {
  const stdin = process.stdin;

  return {
    stdin,
    setRawMode: (isRawMode: boolean) => {
      stdin.setRawMode?.(isRawMode);
    },
    isRawModeSupported: typeof stdin.setRawMode === "function",
    internal_exitOnCtrlC: true,
  };
}

export function useStdout(): UseStdoutReturn {
  const stdout = process.stdout;

  return {
    stdout,
    write: (data: string) => {
      stdout.write(data);
    },
  };
}

export function useStderr(): UseStderrReturn {
  const stderr = process.stderr;

  return {
    stderr,
    write: (data: string) => {
      stderr.write(data);
    },
  };
}
