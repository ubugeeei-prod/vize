import type { LspSession } from "../session.ts";

// Existing waitForNotification also consumes backlog. A close witness must be live.
export async function closePublication(session: LspSession, uri: string) {
  const observed: Array<{ method: string; params: unknown }> = [];
  let timeout: NodeJS.Timeout | undefined;
  let accept: (params: unknown) => void;
  const pending = new Promise<unknown>((resolve, reject) => {
    accept = resolve;
    timeout = setTimeout(() => reject(new Error("didClose live publication timed out")), 30000);
  });
  const observer = (method: string, params: unknown) => {
    observed.push({ method, params });
    const packet = params as { uri?: string; version?: number } | undefined;
    if (
      method === "textDocument/publishDiagnostics" &&
      packet?.uri === uri &&
      packet.version === undefined
    )
      accept(params); // First live scoped packet; diagnostics are asserted whole by the caller.
  };
  session.notificationObservers.push(observer);
  try {
    session.notify("textDocument/didClose", { textDocument: { uri } });
    return { actual: await pending, observed };
  } finally {
    if (timeout) clearTimeout(timeout);
    const index = session.notificationObservers.indexOf(observer);
    if (index >= 0) session.notificationObservers.splice(index, 1);
  }
}
