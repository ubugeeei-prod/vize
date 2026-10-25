import { spawn } from "child_process";
import { serverInvocation } from "./server-command.js";

/** Validate the LSP initialize contract rather than guessing from release numbers. */
export async function inspectWorkspaceServerProtocol(
  serverPath: string,
  timeoutMs = 5000,
): Promise<void> {
  const invocation = serverInvocation(serverPath);
  await new Promise<void>((resolve, reject) => {
    const child = spawn(invocation.command, [...invocation.args, "lsp"], {
      env: invocation.env,
      stdio: "pipe",
      windowsHide: true,
    });
    let input = Buffer.alloc(0);
    let stderr = "";
    let failure: Error | undefined;
    let initialized = false;
    let settled = false;
    const timer = setTimeout(() => finish(new Error("LSP initialize timed out")), timeoutMs);
    let reapTimer: ReturnType<typeof setTimeout> | undefined;
    function send(packet: unknown): void {
      const body = JSON.stringify(packet);
      child.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
    }
    function finish(error?: Error): void {
      if (settled) return;
      settled = true;
      failure = error;
      clearTimeout(timer);
      if (!error) {
        send({ jsonrpc: "2.0", method: "exit" });
        child.stdin.end();
      } else {
        child.kill();
      }
      // Reap even a server that ignores shutdown, EOF, or SIGTERM. Completion
      // waits for close, so no probe process survives into the real client.
      reapTimer = setTimeout(() => {
        failure ??= new Error("LSP server did not exit after shutdown");
        child.kill("SIGKILL");
      }, 250);
    }
    child.on("spawn", () => {
      send({
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
          processId: process.pid,
          rootUri: null,
          workspaceFolders: null,
          capabilities: {},
          // The compatibility probe must not load or warm a project twice.
          initializationOptions: { lint: false, typecheck: false, editor: false, ecosystem: false },
        },
      });
    });
    child.stdin.on("error", (error) => finish(error));
    child.stderr.on("data", (chunk: Buffer) => {
      stderr = (stderr + chunk.toString()).slice(-8192);
    });
    child.stdout.on("data", (chunk: Buffer) => {
      if (settled) return;
      input = Buffer.concat([input, chunk]);
      if (input.length > 1024 * 1024) return finish(new Error("LSP response exceeds 1 MiB"));
      try {
        while (true) {
          const end = input.indexOf("\r\n\r\n");
          if (end < 0) return;
          const length = /^Content-Length:\s*(\d+)\s*$/im.exec(input.subarray(0, end).toString());
          if (!length) throw new Error("Invalid LSP Content-Length");
          const count = Number(length[1]);
          if (count > 1024 * 1024) throw new Error("LSP response exceeds 1 MiB");
          if (input.length < end + 4 + count) return;
          const packet = JSON.parse(input.subarray(end + 4, end + 4 + count).toString());
          input = input.subarray(end + 4 + count);
          if (packet.id === 1) {
            const capabilities: unknown = packet.result?.capabilities;
            if (
              packet.jsonrpc !== "2.0" ||
              packet.error ||
              !capabilities ||
              typeof capabilities !== "object" ||
              Array.isArray(capabilities)
            ) {
              throw new Error("Server does not support the LSP initialize contract");
            }
            initialized = true;
            send({ jsonrpc: "2.0", method: "initialized", params: {} });
            send({ jsonrpc: "2.0", id: 2, method: "shutdown" });
          } else if (packet.id === 2 && initialized) {
            if (packet.error) throw new Error("Server rejected LSP shutdown");
            finish();
            return;
          }
        }
      } catch (error) {
        finish(error instanceof Error ? error : new Error(String(error)));
      }
    });
    child.on("error", (error) => finish(error));
    child.on("close", (code) => {
      clearTimeout(timer);
      clearTimeout(reapTimer);
      if (!settled) failure = new Error(`LSP probe exited before initialization (${code})`);
      else if (code !== 0) failure ??= new Error(`LSP probe exited unsuccessfully (${code})`);
      if (failure) reject(new Error(`${failure.message}${stderr ? `: ${stderr}` : ""}`));
      else resolve();
    });
  });
}
