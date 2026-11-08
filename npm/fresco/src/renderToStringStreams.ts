type Listener = (...args: unknown[]) => void;

/** Create a detached writable stream for non-interactive Fresco rendering. */
export function createNoopWriteStream(columns: number, rows: number): NodeJS.WriteStream {
  const stream = {
    isTTY: false,
    columns,
    rows,
    write: () => true,
    on: (_event: string, _listener: Listener) => stream,
    off: (_event: string, _listener: Listener) => stream,
    once: (_event: string, _listener: Listener) => stream,
    removeListener: (_event: string, _listener: Listener) => stream,
  };

  return stream as unknown as NodeJS.WriteStream;
}

/** Create a detached readable stream for non-interactive Fresco rendering. */
export function createNoopReadStream(): NodeJS.ReadStream {
  const stream = {
    isTTY: false,
    isPaused: () => true,
    pause: () => stream,
    resume: () => stream,
    ref: () => stream,
    unref: () => stream,
    setEncoding: () => stream,
    setRawMode: () => stream,
    on: (_event: string, _listener: Listener) => stream,
    off: (_event: string, _listener: Listener) => stream,
    once: (_event: string, _listener: Listener) => stream,
    removeListener: (_event: string, _listener: Listener) => stream,
  };

  return stream as unknown as NodeJS.ReadStream;
}
