import { Worker } from "node:worker_threads";

export function createArgosClient(python: string) {
  const worker = new Worker(new URL("./argos-worker.ts", import.meta.url), {
    env: { ...process.env, PYTHON_BIN: python, REQ_TIMEOUT: "0" },
  });
  const pending = new Map<
    number,
    { resolve: (text: string) => void; reject: (error: Error) => void }
  >();
  let nextId = 0;
  let stopped: Error | undefined;

  function stop(error: Error): void {
    stopped ??= error;
    for (const request of pending.values()) request.reject(stopped);
    pending.clear();
  }

  worker.on("message", (response: { id: number; error?: string; translated: string }) => {
    const request = pending.get(response.id);
    if (!request) return;
    pending.delete(response.id);
    if (response.error !== undefined) request.reject(new Error(response.error));
    else request.resolve(response.translated);
  });
  worker.on("error", stop);
  worker.on("exit", (code) => stop(new Error(`Argos translation worker exited with code ${code}`)));

  return {
    translate(text: string, locale: string): Promise<string> {
      if (stopped) return Promise.reject(stopped);
      return new Promise<string>((resolve, reject) => {
        const id = nextId++;
        pending.set(id, { resolve, reject });
        worker.postMessage({ id, text, locale });
      });
    },
    close() {
      if (stopped) return;
      stopped = new Error("Argos translation client is closed");
      worker.postMessage(null);
    },
  };
}
