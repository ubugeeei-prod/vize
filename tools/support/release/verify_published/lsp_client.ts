// A minimal LSP client for post-publish checks: start the installed `vize lsp`,
// open files from disk and watch the diagnostics the server publishes.
import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

export type Diagnostics = Map<string, unknown[]>;

export interface LspSession {
  /** Absolute path of the installed `vize` bin script. */
  bin: string;
  /** Workspace folder the client opens. */
  root: string;
  /** Absolute paths of the documents to open. */
  files: string[];
  /** Expected `serverInfo.version`; skipped when absent. */
  version?: string;
  /** Called after every publish: null passes, a string fails, undefined keeps waiting. */
  settle: (latest: Diagnostics) => string | null | undefined;
}

export function uriOf(path: string): string {
  return pathToFileURL(path).href;
}

/** Returns null once `settle` passes, or a failure description. */
export function watchDiagnostics(session: LspSession): Promise<string | null> {
  return new Promise((resolve) => {
    const server = spawn(process.execPath, [session.bin, "lsp"], {
      cwd: session.root,
      stdio: ["pipe", "pipe", "pipe"],
    });
    const latest: Diagnostics = new Map();
    // The server logs every request; keep only the tail for failures.
    let log = "";
    server.stderr.on("data", (chunk: Buffer) => {
      log = (log + chunk.toString()).slice(-4000);
    });
    const done = (result: string | null) => {
      clearTimeout(timer);
      server.removeAllListeners("exit");
      server.kill();
      resolve(result === null ? null : `${result}\n${log}`);
    };
    const timer = setTimeout(
      () => done(`diagnostics did not settle within 60s: ${JSON.stringify([...latest])}`),
      60_000,
    );
    const send = (message: object) => {
      const body = JSON.stringify({ jsonrpc: "2.0", ...message });
      server.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
    };
    let buffer = Buffer.alloc(0);
    server.stdout.on("data", (chunk: Buffer) => {
      buffer = Buffer.concat([buffer, chunk]);
      for (;;) {
        const header = buffer.indexOf("\r\n\r\n");
        if (header < 0) return;
        const length = Number(
          /Content-Length: (\d+)/i.exec(buffer.subarray(0, header).toString())?.[1],
        );
        if (buffer.length < header + 4 + length) return;
        const message = JSON.parse(buffer.subarray(header + 4, header + 4 + length).toString());
        buffer = buffer.subarray(header + 4 + length);
        if (message.id === 1) {
          const version = message.result?.serverInfo?.version;
          if (session.version !== undefined && version !== session.version)
            return done(`serverInfo.version is ${version}`);
          send({ method: "initialized", params: {} });
          for (const file of session.files)
            send({
              method: "textDocument/didOpen",
              params: {
                textDocument: {
                  uri: uriOf(file),
                  languageId: "vue",
                  version: 1,
                  text: readFileSync(file, "utf8"),
                },
              },
            });
        }
        if (message.method === "textDocument/publishDiagnostics") {
          latest.set(message.params.uri, message.params.diagnostics ?? []);
          const result = session.settle(latest);
          if (result !== undefined) return done(result);
        }
      }
    });
    server.on("exit", (code) => done(`vize lsp exited with ${code}`));
    const rootUri = uriOf(session.root);
    send({
      id: 1,
      method: "initialize",
      params: {
        processId: process.pid,
        rootUri,
        workspaceFolders: [{ uri: rootUri, name: "root" }],
        capabilities: {},
      },
    });
  });
}
